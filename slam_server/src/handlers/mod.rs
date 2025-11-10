use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use crate::service::ai_service::{AIService, TextGenerationRequest};
use self::response::HandlerResponse;
use crate::app::routes;

pub mod response;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AIResponseText(crate::service::ai_service::AIResponse<crate::service::ai_service::TextGenerationResponse>);



// 定义响应数据结构
#[derive(Serialize, ToSchema)]
pub struct ApiResponse {
    message: String,
    status: String,
    timestamp: u64,
}

// 定义请求数据结构
#[derive(Deserialize, ToSchema)]
pub struct PostRequest {
    name: String,
    message: String,
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

// AI文本生成处理函数
#[utoipa::path(
    post,
    path = routes::API_AI_GENERATE_TEXT,
    request_body = TextGenerationRequest,
    responses(
        (status = 200, description = "Generate text", body = AIResponseText),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn generate_text_handler(
    State(ai_service): State<Arc<AIService>>,
    Json(payload): Json<TextGenerationRequest>,
) -> HandlerResponse<AIResponseText> {
    match ai_service.generate_text(payload).await {
        Ok(result) => HandlerResponse::Success(AIResponseText(result)),
        Err(e) => HandlerResponse::Error(e.to_string()),
    }
}

/*
// AI图像生成处理函数
#[utoipa::path(
    post,
    path = routes::API_AI_GENERATE_IMAGE,
    request_body = ImageGenerationRequest,
    responses(
        (status = 200, description = "Generate image", body = AIResponseImage),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn generate_image_handler(
    State(ai_service): State<Arc<AIService>>,
    Json(payload): Json<ImageGenerationRequest>,
) -> HandlerResponse<AIResponseImage> {
    match ai_service.generate_image(&payload.prompt, payload.n, payload.size.as_deref(), None).await {
        Ok(result) => HandlerResponse::Success(AIResponseImage(result)),
        Err(e) => HandlerResponse::Error(e.to_string()),
    }
}
*/