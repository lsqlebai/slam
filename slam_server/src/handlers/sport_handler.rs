use axum::extract::State;
use axum_extra::extract::Multipart;
use std::io::Cursor;
use axum::extract::Json;
use std::sync::Arc;
use utoipa::ToSchema;
use super::jwt::Context;

use crate::app::{AppState, routes};
use crate::model::sport::Sport;
use crate::service::sport_service::{StatKind, StatsParam, StatSummary};
use axum::extract::Query;
use serde::Deserialize;
use super::response::HandlerResponse;
use axum::response::IntoResponse;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ActionResponse { pub success: bool }

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ImportResponse { pub success: bool, pub inserted: usize }

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct DeleteRequest { pub id: i32 }

#[utoipa::path(
    post,
    path = routes::API_SPORT_INSERT,
    request_body = Sport,
    responses(
        (status = 200, description = "Insert sport", body = ActionResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
#[axum::debug_handler]
pub async fn insert_sport_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    Json(sport): Json<Sport>,
) -> axum::response::Response {
    if let Err(e) = sport.validate_type_consistency() {
        return HandlerResponse::<ActionResponse>::Error(e).into_response();
    }
    match app.sport_service.insert(sport, &ctx).await {
        Ok(_) => HandlerResponse::<ActionResponse>::Success(ActionResponse { success: true }).into_response(),
        Err(e) => HandlerResponse::<ActionResponse>::Error(e.message).into_response(),
    }
}

#[utoipa::path(
    get,
    path = routes::API_SPORT_LIST,
    responses(
        (status = 200, description = "List sports", body = [Sport]),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
#[axum::debug_handler]
pub async fn list_sport_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    Query(q): Query<ListQuery>,
) -> axum::response::Response {
    let page = q.page.unwrap_or(0);
    let size = q.size.unwrap_or(20);
    match app.sport_service.list(page, size, &ctx).await {
        Ok(v) => HandlerResponse::<Vec<Sport>>::Success(v).into_response(),
        Err(e) => HandlerResponse::<Vec<Sport>>::Error(e.message).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ListQuery { pub page: Option<i32>, pub size: Option<i32> }

#[derive(Deserialize)]
pub struct StatsQuery { pub kind: String, pub year: i32, pub month: Option<u32>, pub week: Option<u32> }

#[utoipa::path(
    get,
    path = routes::API_SPORT_STATS,
    responses(
        (status = 200, description = "Stats", body = StatSummary),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
#[axum::debug_handler]
pub async fn stats_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    Query(q): Query<StatsQuery>,
) -> axum::response::Response {
    let kind = match q.kind.to_lowercase().as_str() {
        "year" => StatKind::Year,
        "month" => StatKind::Month,
        "week" => StatKind::Week,
        "total" => StatKind::Total,
        _ => return HandlerResponse::<StatSummary>::Error("invalid kind".to_string()).into_response(),
    };
    match app.sport_service.stats(StatsParam { kind, year: q.year, month: q.month, week: q.week },&ctx).await {
        Ok(v) => HandlerResponse::<StatSummary>::Success(v).into_response(),
        Err(e) => HandlerResponse::<StatSummary>::Error(e.message).into_response(),
    }
}
#[utoipa::path(
    post,
    path = routes::API_SPORT_UPDATE,
    request_body = Sport,
    responses(
        (status = 200, description = "Update sport", body = ActionResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
#[axum::debug_handler]
pub async fn update_sport_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    Json(sport): Json<Sport>,
) -> axum::response::Response {
    if sport.id <= 0 { return HandlerResponse::<ActionResponse>::Error("invalid id".to_string()).into_response(); }
    if let Err(e) = sport.validate_type_consistency() {
        return HandlerResponse::<ActionResponse>::Error(e).into_response();
    }
    match app.sport_service.update(sport, &ctx).await {
        Ok(_) => HandlerResponse::<ActionResponse>::Success(ActionResponse { success: true }).into_response(),
        Err(e) => HandlerResponse::<ActionResponse>::Error(e.message).into_response(),
    }
}

#[utoipa::path(
    post,
    path = routes::API_SPORT_IMPORT,
    responses(
        (status = 200, description = "Import sports", body = ImportResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
#[axum::debug_handler]
pub async fn import_sport_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    mut multipart: Multipart,
) -> axum::response::Response {
    let mut vendor = String::new();
    let mut csv_data: Option<Vec<u8>> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("vendor") => {
                if let Ok(text) = field.text().await { vendor = text; }
            }
            Some("file") | Some("csv") => {
                if let Ok(bytes) = field.bytes().await { csv_data = Some(bytes.to_vec()); }
            }
            _ => {}
        }
    }
    if vendor.is_empty() || csv_data.is_none() {
        return HandlerResponse::<ImportResponse>::Error("missing vendor or file".to_string()).into_response();
    }
    let bytes = csv_data.unwrap();
    let cursor = Cursor::new(bytes);
    let reader = csv::ReaderBuilder::new().has_headers(true).from_reader(cursor);
    match app.sport_service.import(vendor, reader, &ctx).await {
        Ok(n) => HandlerResponse::<ImportResponse>::Success(ImportResponse { success: n > 0, inserted: n }).into_response(),
        Err(e) => HandlerResponse::<ImportResponse>::Error(e.message).into_response(),
    }
}

#[utoipa::path(
    post,
    path = routes::API_SPORT_DELETE,
    request_body = DeleteRequest,
    responses(
        (status = 200, description = "Delete sport", body = ActionResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
#[axum::debug_handler]
pub async fn delete_sport_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    Json(req): Json<DeleteRequest>,
) -> axum::response::Response {
    if req.id <= 0 { return HandlerResponse::<ActionResponse>::Error("invalid id".to_string()).into_response(); }
    match app.sport_service.delete(req.id, &ctx).await {
        Ok(_) => HandlerResponse::<ActionResponse>::Success(ActionResponse { success: true }).into_response(),
        Err(e) => HandlerResponse::<ActionResponse>::Error(e.message).into_response(),
    }
}
