use crate::model::ai_job::{AiJobAsset, AiJobRecord, AiJobSubmission};
use crate::model::sport::Sport;
use crate::model::user::{User, UserInfo};
use async_trait::async_trait;

#[async_trait]
pub trait SportDao {
    async fn insert(&self, uid: i32, sport: Sport) -> Result<(), String>;
    async fn insert_many(&self, uid: i32, sports: Vec<Sport>) -> Result<usize, String>;
    async fn list(&self, uid: i32, page: i32, size: i32) -> Result<Vec<Sport>, String>;
    async fn list_by_time_range(
        &self,
        uid: i32,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<Sport>, String>;
    async fn update(&self, uid: i32, sport: Sport) -> Result<(), String>;
    async fn remove(&self, uid: i32, id: i32) -> Result<(), String>;
    async fn get_by_id(&self, uid: i32, id: i32) -> Result<Option<Sport>, String>;
    async fn get_first(&self, uid: i32) -> Result<Option<Sport>, String>;
    async fn insert_from_ai_job(
        &self,
        uid: i32,
        sport: Sport,
        job_id: &str,
    ) -> Result<AiJobSubmission, String>;
}

#[async_trait]
pub trait AiJobDao {
    async fn create_job(&self, job: AiJobRecord, assets: Vec<AiJobAsset>) -> Result<(), String>;
    async fn list_jobs(&self, uid: i32, page: i32, size: i32) -> Result<Vec<AiJobRecord>, String>;
    async fn get_job(&self, uid: i32, id: &str) -> Result<Option<AiJobRecord>, String>;
    async fn list_assets(&self, uid: i32, job_id: &str) -> Result<Vec<AiJobAsset>, String>;
    async fn get_asset(&self, uid: i32, asset_id: &str) -> Result<Option<AiJobAsset>, String>;
    async fn count_active_jobs(&self, uid: i32) -> Result<i64, String>;
    async fn claim_next_job(
        &self,
        now: i64,
        lease_until: i64,
    ) -> Result<Option<AiJobRecord>, String>;
    async fn requeue_expired_jobs(&self, now: i64) -> Result<(), String>;
    async fn mark_job_ready(&self, id: &str, result_json: &str, now: i64) -> Result<(), String>;
    async fn mark_job_error(
        &self,
        id: &str,
        code: &str,
        message: &str,
        retry_at: Option<i64>,
        now: i64,
    ) -> Result<(), String>;
    async fn retry_job(&self, uid: i32, id: &str, now: i64) -> Result<bool, String>;
    async fn delete_job(&self, uid: i32, id: &str) -> Result<bool, String>;
    async fn list_assets_for_cleanup(&self, limit: i32) -> Result<Vec<AiJobAsset>, String>;
    async fn mark_asset_deleted(&self, id: &str, now: i64) -> Result<(), String>;
}

#[async_trait]
pub trait UserDao {
    async fn insert(&self, user: User) -> Result<i32, String>;
    async fn get_by_id(&self, id: i32) -> Result<Option<UserInfo>, String>;
    async fn login(&self, name: &str, password: &str) -> Result<Option<User>, String>;
    async fn set_avatar(&self, uid: i32, base64: String) -> Result<(), String>;
}
