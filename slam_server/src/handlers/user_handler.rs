use axum::extract::{Json, State};
use axum::response::IntoResponse;
use axum::http::{HeaderValue, header::SET_COOKIE};
// no request extractor here for OpenAPI, router closures will decide browser detection
use std::sync::Arc;
use utoipa::ToSchema;
use crate::app::{AppState, routes};
use super::response::HandlerResponse;

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct UserAuthRequest {
    pub name: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct UserActionResponse {
    pub success: bool,
}

pub fn token_response(app: &AppState, uid: i32) -> axum::response::Response {
    match app.jwt.create_token(uid) {
        Ok(token) => {
            let mut resp = HandlerResponse::Success(UserActionResponse { success: true }).into_response();
            if let Ok(val) = HeaderValue::from_str(&format!("slam={}; Path=/; HttpOnly; SameSite=Lax", token)) {
                resp.headers_mut().append(SET_COOKIE, val);
            }
            resp
        }
        Err(e) => HandlerResponse::<UserActionResponse>::Error(e).into_response(),
    }
}

#[utoipa::path(
    post,
    path = routes::API_USER_REGISTER,
    request_body = UserAuthRequest,
    responses(
        (status = 200, description = "User registered", body = UserActionResponse),
        (status = 500, description = "Internal error", body = String)
    )
)]
pub async fn user_register_handler(
    State(app): State<Arc<AppState>>,
    Json(req): Json<UserAuthRequest>,
) -> axum::response::Response {
    match app.user_service.register(req.name, req.password).await {
        Ok(uid) => token_response(app.as_ref(), uid),
        Err(e) => HandlerResponse::<UserActionResponse>::Error(e.message).into_response(),
    }
}

#[utoipa::path(
    post,
    path = routes::API_USER_LOGIN,
    request_body = UserAuthRequest,
    responses(
        (status = 200, description = "User login", body = UserActionResponse),
        (status = 500, description = "Internal error", body = String)
    )
)]
pub async fn user_login_handler(
    State(app): State<Arc<AppState>>,
    Json(req): Json<UserAuthRequest>,
) -> axum::response::Response {
    match app.user_service.login(req.name, req.password).await {
        Ok(uid) => token_response(app.as_ref(), uid),
        Err(e) => HandlerResponse::<UserActionResponse>::Error(e.message).into_response(),
    }
}