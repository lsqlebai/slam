use axum::extract::State;
use axum_extra::extract::Multipart;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;
use utoipa::ToSchema;
use super::jwt::Context;
use crate::app::{AppState, routes};
use super::response::HandlerResponse;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct AIResponseText(pub crate::service::ai_service::AIResponse<crate::model::sport::Sport>);

#[utoipa::path(
    post,
    path = routes::API_IMAGE_PARSE,
    responses(
        (status = 200, description = "Sports image parse", body = AIResponseText),
        (status = 400, description = "Bad request", body = crate::service::ai_service::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::service::ai_service::ErrorResponse),
        (status = 422, description = "Unprocessable entity", body = crate::service::ai_service::ErrorResponse),
        (status = 502, description = "Bad gateway", body = crate::service::ai_service::ErrorResponse),
        (status = 504, description = "Gateway timeout", body = crate::service::ai_service::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::service::ai_service::ErrorResponse)
    )
)]

#[axum::debug_handler]
pub async fn sports_image_recognition_handler(
    State(app): State<Arc<AppState>>,
    _ctx: Context,
    mut multipart: Multipart,
) -> axum::response::Response {
    let mut all_base64: Vec<String> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            match field.bytes().await {
                Ok(data) => {
                    println!("field data len: {}", data.len());
                    match app.image_service.process_image(data.into()) {
                        Ok(resp) => {
                            all_base64.extend(resp.base64_data);
                        }
                        Err(e) => {
                            let err = crate::service::ai_service::ErrorResponse { code: "422".to_string(), message: "图片处理失败".to_string(), details: Some(e.to_string()) };
                            let resp = crate::service::ai_service::AIResponse::<crate::model::sport::Sport> { success: false, data: None, error: Some(err), request_id: crate::service::common::generate_request_id() };
                            return (StatusCode::UNPROCESSABLE_ENTITY, Json(resp)).into_response();
                        }
                    }
                }
                Err(e) => {
                    let err = crate::service::ai_service::ErrorResponse { code: "400".to_string(), message: "multipart读取失败".to_string(), details: Some(e.to_string()) };
                    let resp = crate::service::ai_service::AIResponse::<crate::model::sport::Sport> { success: false, data: None, error: Some(err), request_id: crate::service::common::generate_request_id() };
                    return (StatusCode::BAD_REQUEST, Json(resp)).into_response();
                }
            }
        }
    }

    if all_base64.is_empty() {
        let err = crate::service::ai_service::ErrorResponse { code: "400".to_string(), message: "缺少'image'字段".to_string(), details: None };
        let resp = crate::service::ai_service::AIResponse::<crate::model::sport::Sport> { success: false, data: None, error: Some(err), request_id: crate::service::common::generate_request_id() };
        return (StatusCode::BAD_REQUEST, Json(resp)).into_response();
    }

    match app.ai_service.sports_image_recognition(all_base64).await {
        Ok(result) => HandlerResponse::Success(AIResponseText(result)).into_response(),
        Err(e) => {
            let status = match e.code {
                400 => StatusCode::BAD_REQUEST,
                401 => StatusCode::UNAUTHORIZED,
                404 => StatusCode::NOT_FOUND,
                422 => StatusCode::UNPROCESSABLE_ENTITY,
                502 => StatusCode::BAD_GATEWAY,
                504 => StatusCode::GATEWAY_TIMEOUT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            let err = crate::service::ai_service::ErrorResponse { code: format!("{}", e.code), message: e.message, details: None };
            let resp = crate::service::ai_service::AIResponse::<crate::model::sport::Sport> { success: false, data: None, error: Some(err), request_id: crate::service::common::generate_request_id() };
            (status, Json(resp)).into_response()
        }
    }
}
