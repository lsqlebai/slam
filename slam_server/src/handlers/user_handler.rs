use axum::extract::{Json, State};
use axum::response::IntoResponse;
use axum::http::{HeaderValue, header::SET_COOKIE};
// no request extractor here for OpenAPI, router closures will decide browser detection
use std::sync::Arc;
use utoipa::ToSchema;
use crate::app::{AppState, routes};
use crate::handlers::jwt::Context;
use super::response::HandlerResponse;
use axum_extra::extract::Multipart;

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
pub struct UserInfoResponse { pub nickname: String, pub avatar: String }

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
        Ok(u) => HandlerResponse::<UserInfoResponse>::Success(UserInfoResponse { nickname: u.nickname, avatar: u.avatar }).into_response(),
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

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct AvatarUploadResponse { pub success: bool, pub avatar: String }


#[utoipa::path(
    post,
    path = routes::API_USER_AVATAR_UPLOAD,
    responses(
        (status = 200, description = "Upload avatar", body = AvatarUploadResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal error", body = String)
    )
)]
#[axum::debug_handler]
pub async fn user_avatar_upload_handler(
    State(app): State<Arc<AppState>>,
    ctx: Context,
    mut mp: Multipart,
) -> axum::response::Response {
    let mut data: Option<Vec<u8>> = None;
    let mut b64_text: Option<String> = None;
    while let Some(field) = mp.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("");
        if name == "file" {
            data = Some(field.bytes().await.unwrap_or_default().to_vec());
        } else if name == "avatar" || name == "base64" {
            b64_text = Some(field.text().await.unwrap_or_default());
        }
    }
    if let Some(bytes) = data {
        match app.image_service.process_image(bytes) {
            Ok(resp) => {
                let b64 = resp.base64_data.into_iter().next().unwrap_or_default();
                match app.user_service.set_avatar(ctx.uid, b64.clone()).await {
                    Ok(()) => HandlerResponse::<AvatarUploadResponse>::Success(AvatarUploadResponse { success: true, avatar: b64 }).into_response(),
                    Err(e) => HandlerResponse::<AvatarUploadResponse>::Error(e.message).into_response(),
                }
            }
            Err(e) => HandlerResponse::<AvatarUploadResponse>::Error(e.message).into_response(),
        }
    } else if let Some(txt) = b64_text {
        let b64 = txt;
        match app.user_service.set_avatar(ctx.uid, b64.clone()).await {
            Ok(()) => HandlerResponse::<AvatarUploadResponse>::Success(AvatarUploadResponse { success: true, avatar: b64 }).into_response(),
            Err(e) => HandlerResponse::<AvatarUploadResponse>::Error(e.message).into_response(),
        }
    } else {
        HandlerResponse::<AvatarUploadResponse>::Error("缺少文件或base64参数".to_string()).into_response()
    }
}
