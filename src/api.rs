use crate::model::{ApiResponse, JsonValue, RecordingPlan, StartRecordRequest};
use axum::{
    extract::Path,
    routing::{get, post, put},
    Json, Router,
};
use include_dir::{include_dir, Dir};
use log::info;
use tower_http::cors::{Any, CorsLayer};
use tower_serve_static::ServeDir;

use crate::manager;

// 把前端构建后产生的资产文件夹打包进程序里
static ASSETS_DIR: Dir<'static> = include_dir!("web/build");

/// 这个宏用来减少响应的样板代码
macro_rules! api_response {
    ($expr:expr) => {
        match $expr.await {
            Ok(result) => Json(create_success_response(result)),
            Err(e) => Json(create_fail_response(&e.to_string())),
        }
    };
}

/// 这个宏用来从请求中提取字符串类型的参数
macro_rules! extract_string {
    ($req:expr, $key:expr) => {
        match $req.get($key) {
            Some(value) => match value.as_str() {
                Some(value) => value,
                None => return Json(create_fail_response(&format!("{} must be a string", $key))),
            },
            None => return Json(create_fail_response(&format!("{} is required", $key))),
        }
    };
}

pub async fn listen(port: u16) {
    let app = Router::new()
        .nest("/api/record", record::router())
        .nest("/api/plan", plan::router())
        .nest("/api/history", history::router())
        .nest("/api/config", config::router())
        .nest("/api/live", live::router())
        .nest("/api/ffmpeg", ffmpeg_api::router())
        .nest_service("/", ServeDir::new(&ASSETS_DIR))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!("Listening on http://127.0.0.1:{}", port);

    axum::serve(listener, app).await.unwrap();
}

/// 和录制相关的 api
mod record {
    use super::*;

    pub fn router() -> Router {
        Router::new()
            .route("/start", post(start))
            .route("/stop", post(stop))
            .route("/status", post(status))
    }

    /// 开始录制
    pub async fn start(Json(req): Json<StartRecordRequest>) -> Json<ApiResponse> {
        info!("start_record: {:?}", req);
        // 到底要不要用 req.stream 这个参数呢？
        api_response!(manager::record::start(
            &req.url,
            req.auto_record,
            // req.stream
        ))
    }

    /// 停止录制
    pub async fn stop(Json(req): Json<JsonValue>) -> Json<ApiResponse> {
        let url = extract_string!(req, "url");
        info!("stop_record: {:?}", url);
        api_response!(manager::record::stop(url))
    }

    /// 获取录制状态
    pub async fn status(Json(req): Json<JsonValue>) -> Json<ApiResponse> {
        let url = extract_string!(req, "url");
        info!("get_record_status: {:?}", url);
        api_response!(manager::record::status(url))
    }
}

/// 和录制计划相关的 api
mod plan {
    use super::*;

    pub fn router() -> Router {
        Router::new()
            .route("/:status", put(update_status))
            .route("/", get(get_all).post(add).delete(delete))
            .route("/lasttime", get(last_polling_time))
    }

    /// 获取所有录制计划
    pub async fn get_all() -> Json<ApiResponse> {
        info!("get_all_recording_plans");
        api_response!(manager::plan::get_all())
    }

    /// 新建录制计划
    pub async fn add(Json(payload): Json<RecordingPlan>) -> Json<ApiResponse> {
        info!("create_recording_plan: {:?}", payload);
        api_response!(manager::plan::add(&payload))
    }

    /// 删除录制计划
    pub async fn delete(Json(req): Json<JsonValue>) -> Json<ApiResponse> {
        let url = extract_string!(req, "url");

        info!("delete_recording_plan: {}", url);

        api_response!(manager::plan::delete(url))
    }

    /// 更新录制计划状态
    pub async fn update_status(
        Path(enabled): Path<String>,
        Json(req): Json<JsonValue>,
    ) -> Json<ApiResponse> {
        let url = extract_string!(req, "url");

        info!("update_recording_plan_status: {:?}, {}", url, enabled);

        let enabled = if enabled == "enable" {
            true
        } else if enabled == "disable" {
            false
        } else {
            return Json(create_fail_response(
                "invalid status, only accept 'enable' or 'disable'",
            ));
        };

        api_response!(manager::plan::update_status(url, enabled))
    }

    /// 获取上次轮询时间
    pub async fn last_polling_time() -> Json<ApiResponse> {
        info!("get_last_polling_time");
        api_response!(manager::plan::get_last_polling_time())
    }
}

/// 和录制历史相关的 api
mod history {
    use super::*;

    pub fn router() -> Router {
        Router::new()
            .route("/", get(get_all).delete(delete))
            .route("/open", post(open))
    }

    pub async fn get_all() -> Json<ApiResponse> {
        info!("get_history");
        api_response!(manager::history::get_all())
    }

    /// 删除一条历史
    pub async fn delete(Json(req): Json<JsonValue>) -> Json<ApiResponse> {
        let url = extract_string!(req, "url");
        let start_time = match req.get("startTime") {
            Some(value) => match value.as_i64() {
                Some(value) => value,
                None => return Json(create_fail_response("startTime must be a number")),
            },
            None => return Json(create_fail_response("startTime is required")),
        };

        info!("delete_history: {}, {}", url, start_time);

        api_response!(manager::history::delete(url, start_time))
    }

    /// 在文件管理器中打开
    pub async fn open(Json(req): Json<JsonValue>) -> Json<ApiResponse> {
        let path = extract_string!(req, "path");
        api_response!(manager::history::open(path))
    }
}

/// 和配置相关的 api
mod config {
    use super::*;

    pub fn router() -> Router {
        Router::new().route("/", get(get_config).post(set))
    }

    /// 获取配置
    pub async fn get_config() -> Json<ApiResponse> {
        info!("get_config");
        api_response!(manager::config::get())
    }

    /// 设置配置
    pub async fn set(Json(config): Json<JsonValue>) -> Json<ApiResponse> {
        info!("set_config: {:?}", config);
        api_response!(manager::config::set(config))
    }
}

/// 和直播相关的 api
mod live {
    use super::*;

    pub fn router() -> Router {
        Router::new().route("/info", post(info))
    }

    pub async fn info(Json(req): Json<JsonValue>) -> Json<ApiResponse> {
        let url = extract_string!(req, "url");
        info!("get_live_info: {:?}", url);
        api_response!(manager::live::info(url))
    }
}

/// 和 ffmpeg 相关的 api
mod ffmpeg_api {
    use super::*;

    pub fn router() -> Router {
        Router::new().route("/check", post(check))
    }

    /// 检查 ffmpeg
    pub async fn check(Json(req): Json<JsonValue>) -> Json<ApiResponse> {
        let path = extract_string!(req, "path");
        info!("check_ffmpeg: {}", path);
        api_response!(manager::ffmpeg_api::check(path))
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
