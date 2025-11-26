use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub nickname: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone)]
pub struct UserInfo {
    pub nickname: String,
    pub avatar: String,
}
