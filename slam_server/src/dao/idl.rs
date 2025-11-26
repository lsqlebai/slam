use async_trait::async_trait;
use crate::model::sport::Sport;
use crate::model::user::{User, UserInfo};

#[async_trait]
pub trait SportDao {
    async fn insert(&self, uid: i32, sport: Sport) -> Result<(), String>;
    async fn insert_many(&self, uid: i32, sports: Vec<Sport>) -> Result<usize, String>;
    async fn list(&self, uid: i32, page: i32, size: i32) -> Result<Vec<Sport>, String>;
    async fn list_by_time_range(&self, uid: i32, start_time: i64, end_time: i64) -> Result<Vec<Sport>, String>;
    async fn update(&self, uid: i32, sport: Sport) -> Result<(), String>;
    async fn remove(&self, uid: i32, id: i32) -> Result<(), String>;
    async fn get_by_id(&self, uid: i32, id: i32) -> Result<Option<Sport>, String>;
    async fn get_first(&self, uid: i32) -> Result<Option<Sport>, String>;
}

#[async_trait]
pub trait UserDao {
    async fn insert(&self, user: User) -> Result<i32, String>;
    async fn get_by_id(&self, id: i32) -> Result<Option<UserInfo>, String>;
    async fn login(&self, name: &str, password: &str) -> Result<Option<User>, String>;
    async fn set_avatar(&self, uid: i32, base64: String) -> Result<(), String>;
}
