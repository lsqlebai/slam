use axum::extract::State;
use axum_extra::extract::Multipart;
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
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]

#[axum::debug_handler]
pub async fn sports_image_recognition_handler(
    State(app): State<Arc<AppState>>,
    _ctx: Context,
    mut multipart: Multipart,
) -> HandlerResponse<AIResponseText> {
    let mut all_base64: Vec<String> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            if let Ok(data) = field.bytes().await {
                if let Ok(resp) = app.image_service.process_image(data.into()) {
                    all_base64.extend(resp.base64_data);
                }
            }
        }
    }

    if all_base64.is_empty() {
        return HandlerResponse::Error("在multipart请求中未找到 'image' 字段".to_string());
    }

    match app.ai_service.sports_image_recognition(all_base64).await {
        Ok(result) => HandlerResponse::Success(AIResponseText(result)),
        Err(e) => HandlerResponse::Error(e.message),
    }
}