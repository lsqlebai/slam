use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};
use axum::http::{HeaderMap, StatusCode};
use axum::body::Body;
use axum::extract::{FromRequest, FromRequestParts};
use async_trait::async_trait;
use std::sync::Arc;
use crate::app::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub uid: i32,
    pub iat: usize,
    pub exp: usize,
}

pub struct Jwt {
    ttl_seconds: u64,
    secret: String,
}

impl Jwt {
    pub fn new(ttl_seconds: u64, secret: String) -> Self { Self { ttl_seconds, secret } }

    pub fn create_token(&self, uid: i32) -> Result<String, String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs() as usize;
        let exp = now + self.ttl_seconds as usize;
        let claims = Claims { uid, iat: now, exp };
        let header = Header::new(Algorithm::HS256);
        encode(&header, &claims, &EncodingKey::from_secret(self.secret.as_bytes())).map_err(|e| e.to_string())
    }

pub fn verify_token(&self, token: &str) -> Result<Claims, String> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        let data = decode::<Claims>(token, &DecodingKey::from_secret(self.secret.as_bytes()), &validation)
            .map_err(|e| e.to_string())?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs() as usize;
        if data.claims.exp <= now { return Err("token已过期".to_string()); }
        Ok(data.claims)
    }

    pub fn create_context_from_cookie(&self, headers: &HeaderMap) -> Result<Context, String> {
        let cookie_header = headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("");
        let token = cookie_header
            .split(';')
            .map(|s| s.trim())
            .find_map(|pair| pair.strip_prefix("slam=").map(|v| v.to_string()));
        match token {
            Some(t) => self.verify_token(&t).map(|c| Context { uid: c.uid }),
            None => Err("未登录或token无效".to_string()),
        }
    }
}

#[derive(Clone)]
pub struct Context {
    pub uid: i32,
}

#[async_trait]
impl FromRequest<Arc<AppState>> for Context {
    type Rejection = (StatusCode, String);
    async fn from_request(req: axum::http::Request<Body>, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        match state.jwt.create_context_from_cookie(req.headers()) {
            Ok(ctx) => Ok(ctx),
            Err(e) => Err((StatusCode::UNAUTHORIZED, e)),
        }
    }
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for Context {
    type Rejection = (StatusCode, String);
    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        match state.jwt.create_context_from_cookie(&parts.headers) {
            Ok(ctx) => Ok(ctx),
            Err(e) => Err((StatusCode::UNAUTHORIZED, e)),
        }
    }
}