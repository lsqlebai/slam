use async_trait::async_trait;
use crate::model::sport::Sport;

#[async_trait(?Send)]
pub trait SportDao {
    async fn insert(&self, sport: Sport) -> Result<(), String>;
    async fn list(&self) -> Result<Vec<Sport>, String>;
    async fn list_by_time_range(&self, start_time: i64, end_time: i64) -> Result<Vec<Sport>, String>;
}