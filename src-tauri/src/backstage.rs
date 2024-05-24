use crate::{
    kv,
    manager::{self, TASKS},
    model::RecordStatus,
};
use std::time::Duration;

// 在新线程中初始化
pub fn init_with_new_thread() {
    std::thread::spawn(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(init());
    });
}

// 后台任务
pub async fn init() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("后台检查任务已运行");
    check_recording_histories().await;
    tokio::spawn(check_tasks_loop());
    tokio::spawn(check_plans_loop());
    // 无限循环阻塞
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

// 查看数据库中，状态为正在录制的任务，是否真的还在录制，如果不是，就更新数据库记录
pub async fn check_recording_histories() {
    let histories = kv::history::get_all().unwrap_or_else(|e| {
        eprintln!("get_recording_histories error: {}", e);
        vec![]
    });
    if histories.is_empty() {
        return;
    }
    for history in histories {
        if history.status != RecordStatus::Recording {
            continue;
        }
        // 如果任务已经在 TASKS 中，就跳过
        if TASKS.contains_key(&history.url) {
            continue;
        }
        // 否则，更新数据库记录
        kv::history::end(&history.url).unwrap_or_else(|e| {
            eprintln!("end_recording_history error: {}", e);
        });
    }
}

/// 在循环中，每隔 1 秒查看正在运行的任务的状态
pub async fn check_tasks_loop() {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _ = check_tasks()
            .await
            .map_err(|e| eprintln!("check_tasks error: {}", e));
    }
}

/// 在循环中，轮询查看录制计划中的直播是否开始
pub async fn check_plans_loop() {
    loop {
        check_plans().await;
        // 获取轮询间隔
        let interval = kv::config::get()
            .map(|config| config.live_info_check_interval)
            .unwrap_or(10);
        tokio::time::sleep(Duration::from_secs(interval as u64)).await;
    }
}

// 查看 TASKS 中的录制任务是否还在运行，如果不是，就从 TASKS 中移除，并更新数据库记录
pub async fn check_tasks() -> anyhow::Result<()> {
    if TASKS.is_empty() {
        return Ok(());
    }
    let mut need_remove = vec![];
    for mut task in TASKS.iter_mut() {
        let child = task.value_mut();
        match child.try_wait() {
            Ok(Some(_)) => {
                need_remove.push(task.key().clone());
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("check_tasks error: {}", e);
            }
        }
    }
    for url in need_remove {
        let task = TASKS.remove(&url);
        let Some((_url, mut child)) = task else {
            eprintln!("要移除任务时发现找不到它了：{}", url);
            continue;
        };
        if let Some(status) = child.try_wait()? {
            let output = match child
                .wait_with_output()
                .map_err(|e| eprintln!("无法获取输出 for url: {} ：{:?}", url, e))
            {
                Ok(output) => output,
                Err(_) => continue,
            };
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let error_message = format!(
                "status: {:?}; stdout: {}; stderr: {}",
                status, stdout, stderr
            );
            eprintln!("任务已退出：{}", error_message);
        }
        // 更新数据库记录
        kv::history::end(&url).unwrap_or_else(|e| {
            eprintln!("end_recording_history error: {}", e);
        });
    }
    Ok(())
}

pub async fn check_plans() {
    // 首先记录当前检查时间
    kv::plan::mark_polling_time().unwrap_or_else(|e| {
        eprintln!("mark_polling_time error: {}", e);
    });
    // 从数据库中获取所有录制计划
    let plans = kv::plan::get_enabled().unwrap_or_else(|e| {
        eprintln!("get_enabled_recording_plans error: {}", e);
        vec![]
    });
    if plans.is_empty() {
        return;
    }
    for plan in plans {
        // 如果任务已经在 TASKS 中，就跳过
        if TASKS.contains_key(&plan.url) {
            continue;
        }
        // 创建录制任务
        let _ = manager::inner::start_record_default(&plan.url)
            .await
            .map_err(|e| {
                eprintln!("check_plans start record error: {}", e);
            });
    }
}
