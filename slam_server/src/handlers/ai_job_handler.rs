use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::Multipart;
use serde::Deserialize;

use super::jwt::Context;
use crate::app::AppState;
use crate::service::ai_job_service::JobUpload;

#[derive(Deserialize)]
pub struct JobListQuery {
    pub page: Option<i32>,
    pub size: Option<i32>,
}

#[utoipa::path(
    post,
    path = "/api/ai/jobs",
    responses(
        (status = 202, description = "AI job created", body = crate::model::ai_job::AiJobView),
        (status = 400, description = "Invalid image"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Too many active jobs")
    )
)]
pub async fn create_ai_job_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    mut multipart: Multipart,
) -> Response {
    let mut uploads = Vec::new();
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                return (
                    error.status(),
                    Json(serde_json::json!({ "error": format!("multipart读取失败: {error}") })),
                )
                    .into_response();
            }
        };
        if field.name() != Some("image") {
            continue;
        }
        let mime = field
            .content_type()
            .map(str::to_string)
            .unwrap_or_else(|| "image/jpeg".to_string());
        match field.bytes().await {
            Ok(bytes) => uploads.push(JobUpload {
                bytes: bytes.to_vec(),
                mime,
            }),
            Err(error) => {
                return (
                    error.status(),
                    Json(serde_json::json!({ "error": format!("multipart读取失败: {error}") })),
                )
                    .into_response();
            }
        }
    }
    match app.ai_job_service.create_job(ctx.uid, uploads).await {
        Ok(job) => (StatusCode::ACCEPTED, Json(job)).into_response(),
        Err(error) => error_response(error.code, error.message),
    }
}

#[utoipa::path(
    get,
    path = "/api/ai/jobs",
    params(
        ("page" = Option<i32>, Query, description = "Page index"),
        ("size" = Option<i32>, Query, description = "Page size")
    ),
    responses((status = 200, description = "AI jobs", body = [crate::model::ai_job::AiJobView]))
)]
pub async fn list_ai_jobs_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    Query(query): Query<JobListQuery>,
) -> Response {
    match app
        .ai_job_service
        .list(ctx.uid, query.page.unwrap_or(0), query.size.unwrap_or(50))
        .await
    {
        Ok(jobs) => Json(jobs).into_response(),
        Err(error) => error_response(error.code, error.message),
    }
}

#[utoipa::path(
    get,
    path = "/api/ai/jobs/{id}",
    params(("id" = String, Path, description = "AI job id")),
    responses((status = 200, body = crate::model::ai_job::AiJobView), (status = 404, description = "Not found"))
)]
pub async fn get_ai_job_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    Path(id): Path<String>,
) -> Response {
    match app.ai_job_service.get(ctx.uid, &id).await {
        Ok(job) => Json(job).into_response(),
        Err(error) => error_response(error.code, error.message),
    }
}

#[utoipa::path(
    post,
    path = "/api/ai/jobs/{id}/retry",
    params(("id" = String, Path, description = "AI job id")),
    responses((status = 200, description = "Retry scheduled"), (status = 409, description = "Job is not failed"))
)]
pub async fn retry_ai_job_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    Path(id): Path<String>,
) -> Response {
    match app.ai_job_service.retry(ctx.uid, &id).await {
        Ok(()) => Json(serde_json::json!({ "success": true })).into_response(),
        Err(error) => error_response(error.code, error.message),
    }
}

#[utoipa::path(
    get,
    path = "/api/ai/assets/{id}/content",
    params(("id" = String, Path, description = "AI asset id")),
    responses((status = 200, description = "Original image"), (status = 404, description = "Not found"))
)]
pub async fn get_ai_asset_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    Path(id): Path<String>,
) -> Response {
    asset_response(&app, ctx.uid, &id, false).await
}

#[utoipa::path(
    get,
    path = "/api/ai/assets/{id}/thumbnail",
    params(("id" = String, Path, description = "AI asset id")),
    responses((status = 200, description = "Image thumbnail"), (status = 404, description = "Not found"))
)]
pub async fn get_ai_asset_thumbnail_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    Path(id): Path<String>,
) -> Response {
    asset_response(&app, ctx.uid, &id, true).await
}

async fn asset_response(app: &AppState, uid: i32, id: &str, thumbnail: bool) -> Response {
    match app.ai_job_service.read_asset(uid, id, thumbnail).await {
        Ok((bytes, mime)) => {
            let mut response = Response::new(Body::from(bytes));
            *response.status_mut() = StatusCode::OK;
            if let Ok(value) = HeaderValue::from_str(&mime) {
                response.headers_mut().insert(header::CONTENT_TYPE, value);
            }
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("private, max-age=300"),
            );
            response
        }
        Err(error) => error_response(error.code, error.message),
    }
}

fn error_response(code: u32, message: String) -> Response {
    let status = StatusCode::from_u16(code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}
