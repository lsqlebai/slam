use axum::extract::State;
use axum::http::HeaderMap;
use axum::extract::Json;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::app::{AppState, routes};
use crate::model::sport::Sport;
use crate::service::sport_service::{StatKind, StatsParam, StatSummary};
use axum::extract::Query;
use serde::Deserialize;
use super::response::HandlerResponse;
use axum::response::IntoResponse;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ActionResponse { pub success: bool }

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
    headers: HeaderMap,
    Json(sport): Json<Sport>,
) -> axum::response::Response {
    let ctx = match app.jwt.create_context_from_cookie(&headers) { Ok(c) => c, Err(e) => return HandlerResponse::<ActionResponse>::Unauthorized(e).into_response() };
    match app.sport_service.insert(ctx.uid, sport).await {
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
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> axum::response::Response {
    let ctx = match app.jwt.create_context_from_cookie(&headers) { Ok(c) => c, Err(e) => return HandlerResponse::<Vec<Sport>>::Unauthorized(e).into_response() };
    let page = q.page.unwrap_or(0);
    let size = q.size.unwrap_or(20);
    match app.sport_service.list(ctx.uid, page, size).await {
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
    headers: HeaderMap,
    Query(q): Query<StatsQuery>,
) -> axum::response::Response {
    let ctx = match app.jwt.create_context_from_cookie(&headers) { Ok(c) => c, Err(e) => return HandlerResponse::<StatSummary>::Unauthorized(e).into_response() };
    let kind = match q.kind.to_lowercase().as_str() {
        "year" => StatKind::Year,
        "month" => StatKind::Month,
        "week" => StatKind::Week,
        _ => return HandlerResponse::<StatSummary>::Error("invalid kind".to_string()).into_response(),
    };
    match app.sport_service.stats(ctx.uid, StatsParam { kind, year: q.year, month: q.month, week: q.week }).await {
        Ok(v) => HandlerResponse::<StatSummary>::Success(v).into_response(),
        Err(e) => HandlerResponse::<StatSummary>::Error(e.message).into_response(),
    }
}