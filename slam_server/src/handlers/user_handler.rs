use axum::extract::{Json, State};
use axum::response::IntoResponse;
use axum::http::{HeaderValue, header::SET_COOKIE};
// no request extractor here for OpenAPI, router closures will decide browser detection
use std::sync::Arc;
use utoipa::ToSchema;
use crate::app::{AppState, routes};
use crate::handlers::jwt::Context;
use super::response::HandlerResponse;

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct UserRegisterRequest {
    pub name: String,
    pub password: String,
    pub nickname: String,
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
    path = routes::API_USER_LOGOUT,
    responses(
        (status = 200, description = "User logout", body = UserActionResponse)
    )
)]
pub async fn user_logout_handler(
    State(_app): State<Arc<AppState>>,
    _ctx: Context,
) -> axum::response::Response {
    let mut resp = HandlerResponse::Success(UserActionResponse { success: true }).into_response();
    if let Ok(val) = HeaderValue::from_str("slam=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0") {
        resp.headers_mut().append(SET_COOKIE, val);
    }
    resp
}
#[utoipa::path(
    post,
    path = routes::API_USER_REGISTER,
    request_body = UserRegisterRequest,
    responses(
        (status = 200, description = "User registered", body = UserActionResponse),
        (status = 500, description = "Internal error", body = String)
    )
)]
pub async fn user_register_handler(
    State(app): State<Arc<AppState>>,
    Json(req): Json<UserRegisterRequest>,
) -> axum::response::Response {
    let user = crate::model::user::User { id: 0, name: req.name, password: req.password, nickname: req.nickname };
    match app.user_service.register(user).await {
        Ok(uid) => token_response(app.as_ref(), uid),
        Err(e) => HandlerResponse::<UserActionResponse>::Error(e.message).into_response(),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct UserInfoResponse { pub nickname: String }

#[utoipa::path(
    get,
    path = routes::API_USER_INFO,
    responses(
        (status = 200, description = "User info", body = UserInfoResponse),
        (status = 401, description = "Unauthorized", body = String)
    )
)]
pub async fn user_info_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
) -> axum::response::Response {
    match app.user_service.get_user(ctx.uid).await {
        Ok(u) => HandlerResponse::<UserInfoResponse>::Success(UserInfoResponse { nickname: u.nickname }).into_response(),
        Err(e) => HandlerResponse::<UserInfoResponse>::Error(e.message).into_response(),
    }
}

#[utoipa::path(
    post,
    path = routes::API_USER_LOGIN,
    request_body = UserLoginRequest,
    responses(
        (status = 200, description = "User login", body = UserActionResponse),
        (status = 500, description = "Internal error", body = String)
    )
)]
pub async fn user_login_handler(
    State(app): State<Arc<AppState>>,
    Json(req): Json<UserLoginRequest>,
) -> axum::response::Response {
    match app.user_service.login(req.name, req.password).await {
        Ok(uid) => token_response(app.as_ref(), uid),
        Err(e) => HandlerResponse::<UserActionResponse>::Error(e.message).into_response(),
    }
}
#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct UserLoginRequest {
    pub name: String,
    pub password: String,
}
