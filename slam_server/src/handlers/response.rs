use axum::{
    response::{IntoResponse, Response},
    Json,
    http::StatusCode,
};
use serde::Serialize;

// 定义一个可以包含成功或失败响应的枚举
pub enum HandlerResponse<T: Serialize> {
    Success(T),
    Error(String),
}

// 为HandlerResponse实现IntoResponse trait
impl<T: Serialize> IntoResponse for HandlerResponse<T> {
    fn into_response(self) -> Response {
        match self {
            HandlerResponse::Success(data) => (StatusCode::OK, Json(data)).into_response(),
            HandlerResponse::Error(err_msg) => {
                let error_response = serde_json::json!({
                    "error": err_msg,
                    "request_id": "unknown-request-id" // 在实际应用中，这里应该是一个真实的请求ID
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
            }
        }
    }
}