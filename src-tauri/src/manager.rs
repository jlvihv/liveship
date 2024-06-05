use crate::model::AppConfig;
use crate::model::LiveInfo;
use crate::model::{PlatformKind, Stream};
use crate::{
    ffmpeg, kv,
    model::{RecordStatus, RecordingHistory, RecordingPlan},
    utils,
};
use chrono::Utc;
use dashmap::DashMap;
use ffmpeg_sidecar::version::ffmpeg_version_with_path;
use once_cell::sync::Lazy;
use std::result::Result;
use std::{path::PathBuf, process::Child};

// 用一个 dashmap 用来保存已经开始的录制任务
pub static TASKS: Lazy<DashMap<String, Child>> = Lazy::new(|| DashMap::new());

/// 内部方法，不对外暴露
pub mod inner {
    use super::*;

    /// 为 api 提供的录制方法，使用 api 传入的 stream 信息，不再获取直播间信息
    pub async fn start_record_with_stream(
        stream: Stream,
        live_info: LiveInfo,
    ) -> anyhow::Result<()> {
        // 如果已经在录制了，就不再录制，返回错误
        if inner::get_record_status(&live_info.url).await? == RecordStatus::Recording {
            return Err(anyhow::anyhow!("Already recording"));
        }
        let (path, filename) =
            utils::generate_path_and_filename(&live_info.platform_kind, &live_info.anchor_name)
                .await?;
        // 如果路径不存在，则创建
        if !std::path::Path::new(&path).exists() {
            std::fs::create_dir_all(&path)?;
        }

        // 使用系统提供的函数拼接路径和文件名
        let path = PathBuf::from(path);
        let full_filename = path.join(filename);
        let full_filename = full_filename
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Could not convert path to string: {:?}", path))?;

        inner::record_with_ffmpeg(&live_info.url, &stream.url, full_filename).await?;

        // 记录录制历史
        let mut history = RecordingHistory::new(&live_info.url, full_filename);
        history.live_info = Some(live_info);
        kv::history::add(&history).unwrap_or_else(|e| {
            eprintln!("Could not add recording history: {}", e);
        });

        Ok(())
    }

    pub(super) async fn record_with_ffmpeg(
        url: &str,
        stream_url: &str,
        full_filename: &str,
    ) -> anyhow::Result<()> {
        let ffmpeg_path = kv::config::get()?.ffmpeg_path;
        let mut child = match ffmpeg::record(&ffmpeg_path, stream_url, full_filename) {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Could not start recording: {}", e);
                return Err(e);
            }
        };

        if let Some(status) = child.try_wait()? {
            let output = child.wait_with_output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let error_message = format!(
                "status: {:?}\nstdout: {}\nstderr: {}",
                status, stdout, stderr
            );
            eprintln!("可能不是预期的结果，但程序已退出：{}", error_message);
            return Err(anyhow::anyhow!(error_message));
        }

        // 将 child 插入到 TASKS 中
        TASKS.insert(url.into(), child);
        Ok(())
    }

    /// 获取录制状态，就是看对应 task 的 Child 是否还在运行
    pub(super) async fn get_record_status(url: &str) -> anyhow::Result<RecordStatus> {
        // 任务状态都维护在数据库里，避免直接操作 TASKS
        // 从数据库获取状态
        if kv::recording::is_exist(url)? {
            Ok(RecordStatus::Recording)
        } else {
            Ok(RecordStatus::NotRecording)
        }
    }
}

pub mod record {
    use super::*;

    /// 开始录制，就是调用 ffmpeg::record 方法
    #[tauri::command]
    pub async fn start_record(
        auto_record: bool,
        stream: Stream,
        live_info: LiveInfo,
    ) -> Result<RecordStatus, String> {
        // 如果要自动录制，加入录制计划表
        if auto_record {
            let mut plan = RecordingPlan::new(
                &live_info.url,
                stream.protocol.clone(),
                stream.resolution.clone(),
            );
            plan.live_info = Some(live_info.clone());
            kv::plan::add(&plan).map_err(|e| format!("Could not add recording plan: {}", e))?;
        }
        inner::start_record_with_stream(stream, live_info)
            .await
            .map_err(|e| {
                eprintln!("Could not start recording: {}", e);
                e.to_string()
            })?;
        Ok(RecordStatus::Recording)
    }

    /// 停止录制，就是杀死对应 task 的 Child
    #[tauri::command]
    pub async fn stop_record(url: &str) -> Result<RecordStatus, String> {
        if let Some(mut task) = TASKS.get_mut(url) {
            // 杀掉任务
            task.value_mut().kill().map_err(|e| {
                eprintln!("Could not kill task: {}", e);
                e.to_string()
            })?;

            // 杀掉任务后，变成了僵尸进程，通过 wait 来回收资源
            task.value_mut().wait().map_err(|e| {
                eprintln!("Could not wait for task: {}", e);
                e.to_string()
            })?;
            println!("停止录制成功：{}", url);
        }
        // 删除对应的 task
        TASKS.remove(url);
        // 更新录制历史
        kv::history::end(url).map_err(|e| {
            eprintln!("Could not update recording history: {}", e);
            e.to_string()
        })?;
        Ok(RecordStatus::NotRecording)
    }

    /// 获取录制状态，返回 JsonValue，用于 API
    #[tauri::command]
    pub async fn record_status(url: &str) -> Result<RecordStatus, String> {
        let status = inner::get_record_status(url).await.map_err(|e| {
            eprintln!("Could not get record status: {}", e);
            e.to_string()
        })?;
        Ok(status)
    }

    // 退出所有录制，同时更新数据库
    pub fn stop_all_record() {
        // 先收集所有的 key，然后遍历 key，通过 remove 删除
        let keys: Vec<String> = TASKS.iter().map(|pair| pair.key().clone()).collect();
        for key in keys {
            if let Some((url, mut child)) = TASKS.remove(&key) {
                if let Err(e) = child.kill() {
                    eprintln!("Could not kill task: {}", e);
                }
                if let Err(e) = child.wait() {
                    eprintln!("Could not wait for task: {}", e);
                }
                println!("停止录制成功：{}", url);
                kv::history::end(&url).unwrap_or_else(|e| {
                    eprintln!("Could not update recording history: {}", e);
                });
            }
        }
    }
}

pub mod plan {
    use super::*;

    /// 获取所有录制计划
    #[tauri::command]
    pub async fn get_all_plans() -> Result<Vec<RecordingPlan>, String> {
        let plans = kv::plan::get_all().map_err(|e| {
            eprintln!("Could not get all recording plans: {}", e);
            e.to_string()
        })?;
        Ok(plans)
    }

    /// 新增录制计划
    #[tauri::command]
    pub async fn add_plan(plan: RecordingPlan) -> Result<(), String> {
        kv::plan::add(&plan).map_err(|e| {
            eprintln!("Could not add recording plan: {}", e);
            e.to_string()
        })?;
        Ok(())
    }

    #[tauri::command]
    pub async fn add_plan_with_url(url: String) -> Result<(), String> {
        let platform_kind = PlatformKind::from(url.clone());
        if platform_kind == PlatformKind::Unknown {
            return Err("Unknown platform".to_string());
        }
        let plan = RecordingPlan::new_with_url(&url);
        kv::plan::add(&plan).map_err(|e| {
            eprintln!("Could not add recording plan: {}", e);
            e.to_string()
        })?;
        Ok(())
    }

    /// 删除录制计划
    #[tauri::command]
    pub async fn delete_plan(url: &str) -> Result<(), String> {
        kv::plan::delete(url).map_err(|e| {
            eprintln!("Could not delete recording plan: {}", e);
            e.to_string()
        })?;
        Ok(())
    }

    /// 更新录制计划状态
    #[tauri::command]
    pub async fn update_plan_status(url: &str, enabled: bool) -> Result<(), String> {
        kv::plan::update_status(url, enabled).map_err(|e| {
            eprintln!("Could not update recording plan status: {}", e);
            e.to_string()
        })?;
        Ok(())
    }

    /// 获取上次轮询时间
    #[tauri::command]
    pub async fn get_last_polling_time() -> Result<i64, String> {
        let last_polling_time = kv::plan::get_last_polling_time().map_err(|e| {
            eprintln!("Could not get last polling time: {}", e);
            e.to_string()
        })?;
        Ok(last_polling_time)
    }

    // 获取有计划，但未在录制中的任务
    #[tauri::command]
    pub async fn get_plans_not_recording() -> Vec<RecordingPlan> {
        let plans = kv::plan::get_enabled().unwrap_or_else(|e| {
            eprintln!("get_enabled_recording_plans error: {}", e);
            vec![]
        });
        let mut result = vec![];
        for plan in plans {
            if !TASKS.contains_key(&plan.url) {
                result.push(plan);
            }
        }
        result
    }
}

pub mod history {
    use super::*;

    /// 获取所有录制历史
    #[tauri::command]
    pub async fn get_all_history() -> Result<Vec<RecordingHistory>, String> {
        let mut histories = kv::history::get_all()
            .map_err(|e| format!("Could not get all recording histories: {}", e))?;
        // 遍历列表，计算每个文件的尺寸，更新到 histories 中，仅在文件存在时才计算
        for history in &mut histories {
            if std::path::Path::new(&history.path).exists() {
                let file_size = std::fs::metadata(&history.path)
                    .map_err(|e| format!("Could not get file size: {}", e))?
                    .len();
                history.file_size = file_size;
            } else {
                // 当开始录制于 20 秒前，但文件不存在，我们认为文件已经被删除
                if Utc::now().timestamp_millis() - history.start_time > 20000 {
                    history.deleted = true;
                }
            }
        }
        Ok(histories)
    }

    /// 删除一条历史记录
    #[tauri::command]
    pub async fn delete_history(
        url: &str,
        start_time: i64,
        delete_file: bool,
    ) -> Result<(), String> {
        kv::history::delete(url, start_time, delete_file)
            .map_err(|e| format!("Could not delete recording history: {}", e))?;
        Ok(())
    }

    /// 在文件管理器中打开文件夹
    #[tauri::command]
    pub async fn open_in_folder(path: &str) -> Result<(), String> {
        Ok(showfile::show_path_in_file_manager(path))
    }
}

pub mod config {
    use super::*;

    /// 获取配置
    #[tauri::command]
    pub fn get_config() -> Result<AppConfig, String> {
        let config = kv::config::get().map_err(|e| format!("Could not get config: {}", e))?;
        Ok(config)
    }

    /// 设置配置
    #[tauri::command]
    pub fn set_config(config: AppConfig) -> Result<(), String> {
        kv::config::set(&config).map_err(|e| format!("Could not set config: {}", e))?;
        Ok(())
    }
}

pub mod ffmpeg_api {
    use crate::{model::JsonMap, request};

    use super::*;

    /// 检查 ffmpeg
    #[tauri::command]
    pub fn check_ffmpeg_version(path: &str) -> Result<String, String> {
        let version =
            ffmpeg_version_with_path(path).map_err(|e| format!("Could not check ffmpeg: {}", e))?;
        Ok(version)
    }

    /// 检查 ffmpeg 可用性
    #[tauri::command]
    pub fn check_ffmpeg_availability() -> Result<String, String> {
        let config = kv::config::get().map_err(|e| format!("Could not get config: {}", e))?;
        let ffmpeg_path = config.ffmpeg_path;
        check_ffmpeg_version(&ffmpeg_path)
    }

    /// 自动下载 ffmpeg
    #[tauri::command]
    pub async fn download_ffmpeg() -> Result<String, String> {
        let path =
            ffmpeg::download_ffmpeg().map_err(|e| format!("Could not download ffmpeg: {}", e))?;
        // 更新配置
        let mut config = kv::config::get().map_err(|e| format!("Could not get config: {}", e))?;
        config.ffmpeg_path = path.clone();
        kv::config::set(&config).map_err(|e| format!("Could not set config: {}", e))?;
        Ok(path)
    }

    /// 请求 url，得到 text
    #[tauri::command]
    pub async fn request(url: String, headers: JsonMap) -> Result<String, String> {
        // 遍历 headers，转换成 HeaderMap
        let headers = headers
            .iter()
            .map(|(k, v)| (k.as_str().into(), v.as_str().unwrap_or("").into()))
            .collect::<Vec<(String, String)>>();
        let mut header_map = reqwest::header::HeaderMap::new();
        for (k, v) in headers {
            header_map.insert(
                reqwest::header::HeaderName::from_bytes(k.as_bytes()).map_err(|e| e.to_string())?,
                reqwest::header::HeaderValue::from_str(&v).map_err(|e| e.to_string())?,
            );
        }

        let resp = request::get_with_headers(&url, header_map)
            .await
            .map_err(|e| format!("Could not request: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Could not get text: {}", e))?;
        Ok(resp)
    }
}
