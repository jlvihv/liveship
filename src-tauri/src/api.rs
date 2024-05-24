use crate::manager;
use crate::model::{ApiResponse, JsonValue, RecordingPlan, StartRecordRequest};
use log::info;

// 把前端构建后产生的资产文件夹打包进程序里
// static ASSETS_DIR: Dir<'static> = include_dir!("web/build");

/// 这个宏用来减少响应的样板代码
macro_rules! api_response {
    ($expr:expr) => {
        match $expr.await {
            Ok(result) => create_success_response(result),
            Err(e) => create_fail_response(&e.to_string()),
        }
    };
}

/// 这个宏用来从请求中提取字符串类型的参数
macro_rules! extract_string {
    ($req:expr, $key:expr) => {
        match $req.get($key) {
            Some(value) => match value.as_str() {
                Some(value) => value,
                None => return create_fail_response(&format!("{} must be a string", $key)),
            },
            None => return create_fail_response(&format!("{} is required", $key)),
        }
    };
}

/// 和录制相关的 api
pub mod record {

    use super::*;

    /// 开始录制
    #[tauri::command]
    pub async fn start(req: StartRecordRequest) -> ApiResponse {
        info!("start_record: {:?}", req);
        // 到底要不要用 req.stream 这个参数呢？
        api_response!(manager::record::start(
            &req.url,
            req.auto_record,
            req.stream,
            req.platform_kind,
            req.anchor_name
        ))
    }

    /// 停止录制
    pub async fn stop(req: JsonValue) -> ApiResponse {
        let url = extract_string!(req, "url");
        info!("stop_record: {:?}", url);
        api_response!(manager::record::stop(url))
    }

    /// 获取录制状态
    pub async fn status(req: JsonValue) -> ApiResponse {
        let url = extract_string!(req, "url");
        info!("get_record_status: {:?}", url);
        api_response!(manager::record::status(url))
    }
}

/// 和录制计划相关的 api
mod plan {
    use super::*;

    /// 获取所有录制计划
    pub async fn get_all() -> ApiResponse {
        info!("get_all_recording_plans");
        api_response!(manager::plan::get_all())
    }

    /// 新建录制计划
    pub async fn add(req: JsonValue) -> ApiResponse {
        info!("add plan: {:?}", req);
        let url = extract_string!(req, "url");
        let plan = RecordingPlan::new(url);
        api_response!(manager::plan::add(&plan))
    }

    /// 删除录制计划
    pub async fn delete(req: JsonValue) -> ApiResponse {
        let url = extract_string!(req, "url");

        info!("delete_recording_plan: {}", url);

        api_response!(manager::plan::delete(url))
    }

    /// 更新录制计划状态
    pub async fn update_status(enabled: String, req: JsonValue) -> ApiResponse {
        let url = extract_string!(req, "url");

        info!("update_recording_plan_status: {:?}, {}", url, enabled);

        let enabled = if enabled == "enable" {
            true
        } else if enabled == "disable" {
            false
        } else {
            return create_fail_response("invalid status, only accept 'enable' or 'disable'");
        };

        api_response!(manager::plan::update_status(url, enabled))
    }

    /// 获取上次轮询时间
    pub async fn last_polling_time() -> ApiResponse {
        info!("get_last_polling_time");
        api_response!(manager::plan::get_last_polling_time())
    }
}

/// 和录制历史相关的 api
mod history {
    use super::*;

    pub async fn get_all() -> ApiResponse {
        info!("get_history");
        api_response!(manager::history::get_all())
    }

    /// 删除一条历史
    pub async fn delete(req: JsonValue) -> ApiResponse {
        let url = extract_string!(req, "url");
        let start_time = match req.get("startTime") {
            Some(value) => match value.as_i64() {
                Some(value) => value,
                None => return create_fail_response("startTime must be a number"),
            },
            None => return create_fail_response("startTime is required"),
        };

        info!("delete_history: {}, {}", url, start_time);

        api_response!(manager::history::delete(url, start_time))
    }

    /// 在文件管理器中打开
    pub async fn open(req: JsonValue) -> ApiResponse {
        let path = extract_string!(req, "path");
        api_response!(manager::history::open(path))
    }
}

fn create_success_response(data: JsonValue) -> ApiResponse {
    ApiResponse {
        code: 0,
        message: "success".to_string(),
        data,
    }
}

fn create_fail_response(msg: &str) -> ApiResponse {
    ApiResponse {
        code: -1,
        message: msg.to_string(),
        data: JsonValue::Null,
    }
}
