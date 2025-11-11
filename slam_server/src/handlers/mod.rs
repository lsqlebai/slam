use axum::extract::{Json, State};
use axum_extra::extract::Multipart;

use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;
use crate::service::ai_service::AIService;
use crate::service::image_service::ImageService;
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
    path = routes::API_IMAGE_PARSE,
    request_body = TextGenerationRequest,
    responses(
        (status = 200, description = "Generate text", body = AIResponseText),
        (status = 500, description = "Internal server error", body = String)
    )
)]

pub async fn generate_text_handler(
    State(ai_service): State<Arc<AIService>>,
    mut multipart: Multipart,
) -> HandlerResponse<AIResponseText> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            if let Ok(data) = field.bytes().await {
                // 调用服务处理图片
                return match ai_service.generate_text(data.into()).await {
                    Ok(result) => {
                        // 打印base64数据长度
                        //println!("Base64 data length: {}", result.base64_data.len());
                        HandlerResponse::Success(AIResponseText(result))
                    },
                    Err(e) => HandlerResponse::Error(e.message),
                };
            }
        }
    }

    HandlerResponse::Error("在multipart请求中未找到 'image' 字段".to_string())
}



/// 图片上传和压缩处理函数
#[utoipa::path(
    post,
    path = routes::API_IMAGE_PARSE,
    request_body(content = ImageUploadRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "图片处理成功", body = crate::service::image_service::ImageProcessResponse),
        (status = 400, description = "请求参数错误", body = String),
        (status = 500, description = "服务器内部错误", body = String)
    )
)]

pub async fn compress_image_handler(
    State(image_service): State<Arc<ImageService>>,
    mut multipart: Multipart,
) -> HandlerResponse<crate::service::image_service::ImageProcessResponse> {

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            if let Ok(data) = field.bytes().await {
                // 调用服务处理图片
                return match image_service.process_image(data.into()) {
                    Ok(result) => {
                        // 打印base64数据长度
                        println!("Base64 data length: {}", result.base64_data.len());
                        HandlerResponse::Success(result)
                    },
                    Err(e) => HandlerResponse::Error(e.message),
                };
            }
        }
    }

    HandlerResponse::Error("在multipart请求中未找到 'image' 字段".to_string())
}