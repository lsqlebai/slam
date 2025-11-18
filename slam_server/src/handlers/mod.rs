use axum::extract::Json;

use serde::Serialize;
use utoipa::ToSchema;
use crate::app::routes;
pub mod response;
pub mod jwt;
pub mod user_handler;
pub mod ai_handler;

// 定义响应数据结构
#[derive(Serialize, ToSchema)]
pub struct ApiResponse {
    message: String,
    status: String,
    timestamp: u64,
}

pub struct Context {
    pub uid: String,
}

/// Root 端点
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, body = String, description = "Root endpoint")
    )
)]
pub async fn root() -> &'static str {
    "Hello, World!"
}


// 获取服务状态的API
#[utoipa::path(
    get,
    path = routes::API_STATUS,
    responses(
        (status = 200, description = "Get service status", body = ApiResponse)
    )
)]
pub async fn get_status() -> Json<ApiResponse> {
    let response = ApiResponse {
        message: "服务正常运行中".to_string(),
        status: "ok".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    Json(response)
}

// 用户相关 handler 移至 user_handler / AI handler 移至 ai_handler

// 用户相关 handler 移至 user_handler 模块