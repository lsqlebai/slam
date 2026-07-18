use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::sport::Sport;

pub const JOB_QUEUED: &str = "queued";
pub const JOB_RUNNING: &str = "running";
pub const JOB_READY: &str = "ready";
pub const JOB_FAILED: &str = "failed";
pub const JOB_SUBMITTED: &str = "submitted";

#[derive(Debug, Clone)]
pub struct AiJobRecord {
    pub id: String,
    pub uid: i32,
    pub status: String,
    pub result_json: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub attempts: i32,
    pub next_attempt_at: Option<i64>,
    pub lease_until: Option<i64>,
    pub submitted_sport_id: Option<i32>,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub submitted_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiJobAsset {
    pub id: String,
    #[serde(skip_serializing)]
    pub uid: i32,
    #[serde(skip_serializing)]
    pub job_id: String,
    #[serde(skip_serializing)]
    pub original_path: String,
    #[serde(skip_serializing)]
    pub thumbnail_path: String,
    pub mime: String,
    pub position: i32,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiJobView {
    pub id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Sport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub attempts: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_attempt_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_sport_id: Option<i32>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_at: Option<i64>,
    pub assets: Vec<AiJobAsset>,
}

#[derive(Debug, Clone)]
pub struct AiJobSubmission {
    pub sport_id: i32,
    pub asset_paths: Vec<String>,
}
