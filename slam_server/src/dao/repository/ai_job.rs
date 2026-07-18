use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};

use super::Repository;
use crate::dao::idl::AiJobDao;
use crate::model::ai_job::{
    AiJobAsset, AiJobRecord, JOB_FAILED, JOB_QUEUED, JOB_READY, JOB_RUNNING,
};

fn job_from_row(row: &sea_orm::QueryResult) -> Result<AiJobRecord, String> {
    Ok(AiJobRecord {
        id: row.try_get("", "id").map_err(|e| e.to_string())?,
        uid: row.try_get("", "uid").map_err(|e| e.to_string())?,
        status: row.try_get("", "status").map_err(|e| e.to_string())?,
        result_json: row.try_get("", "result_json").map_err(|e| e.to_string())?,
        error_code: row.try_get("", "error_code").map_err(|e| e.to_string())?,
        error_message: row
            .try_get("", "error_message")
            .map_err(|e| e.to_string())?,
        attempts: row.try_get("", "attempts").map_err(|e| e.to_string())?,
        next_attempt_at: row
            .try_get("", "next_attempt_at")
            .map_err(|e| e.to_string())?,
        lease_until: row.try_get("", "lease_until").map_err(|e| e.to_string())?,
        submitted_sport_id: row
            .try_get("", "submitted_sport_id")
            .map_err(|e| e.to_string())?,
        created_at: row.try_get("", "created_at").map_err(|e| e.to_string())?,
        started_at: row.try_get("", "started_at").map_err(|e| e.to_string())?,
        finished_at: row.try_get("", "finished_at").map_err(|e| e.to_string())?,
        submitted_at: row.try_get("", "submitted_at").map_err(|e| e.to_string())?,
    })
}

fn asset_from_row(row: &sea_orm::QueryResult) -> Result<AiJobAsset, String> {
    Ok(AiJobAsset {
        id: row.try_get("", "id").map_err(|e| e.to_string())?,
        uid: row.try_get("", "uid").map_err(|e| e.to_string())?,
        job_id: row.try_get("", "job_id").map_err(|e| e.to_string())?,
        original_path: row
            .try_get("", "original_path")
            .map_err(|e| e.to_string())?,
        thumbnail_path: row
            .try_get("", "thumbnail_path")
            .map_err(|e| e.to_string())?,
        mime: row.try_get("", "mime").map_err(|e| e.to_string())?,
        position: row.try_get("", "position").map_err(|e| e.to_string())?,
        created_at: row.try_get("", "created_at").map_err(|e| e.to_string())?,
        deleted_at: row.try_get("", "deleted_at").map_err(|e| e.to_string())?,
    })
}

const JOB_COLUMNS: &str = "id, uid, status, result_json, error_code, error_message, attempts, next_attempt_at, lease_until, submitted_sport_id, created_at, started_at, finished_at, submitted_at";
const ASSET_COLUMNS: &str =
    "id, uid, job_id, original_path, thumbnail_path, mime, position, created_at, deleted_at";

#[async_trait]
impl AiJobDao for Repository {
    async fn create_job(&self, job: AiJobRecord, assets: Vec<AiJobAsset>) -> Result<(), String> {
        self.conn
            .transaction(|txn| {
                Box::pin(async move {
                    txn.execute(Statement::from_sql_and_values(
                        DbBackend::Sqlite,
                        "INSERT INTO ai_jobs (id, uid, status, attempts, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?)",
                        vec![
                            job.id.clone().into(),
                            job.uid.into(),
                            job.status.clone().into(),
                            job.created_at.into(),
                            job.created_at.into(),
                        ],
                    ))
                    .await?;
                    for asset in assets {
                        txn.execute(Statement::from_sql_and_values(
                            DbBackend::Sqlite,
                            "INSERT INTO ai_job_assets (id, uid, job_id, original_path, thumbnail_path, mime, position, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                            vec![
                                asset.id.into(),
                                asset.uid.into(),
                                asset.job_id.into(),
                                asset.original_path.into(),
                                asset.thumbnail_path.into(),
                                asset.mime.into(),
                                asset.position.into(),
                                asset.created_at.into(),
                            ],
                        ))
                        .await?;
                    }
                    Ok::<_, sea_orm::DbErr>(())
                })
            })
            .await
            .map_err(|e| format!("创建AI任务失败: {e}"))
    }

    async fn list_jobs(&self, uid: i32, page: i32, size: i32) -> Result<Vec<AiJobRecord>, String> {
        let safe_page = page.max(0);
        let safe_size = size.clamp(1, 100);
        let sql = format!(
            "SELECT {JOB_COLUMNS} FROM ai_jobs WHERE uid = ? AND status != 'submitted' ORDER BY created_at DESC LIMIT ? OFFSET ?"
        );
        let rows = self
            .conn
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                sql,
                vec![uid.into(), safe_size.into(), (safe_page * safe_size).into()],
            ))
            .await
            .map_err(|e| format!("查询AI任务失败: {e}"))?;
        rows.iter().map(job_from_row).collect()
    }

    async fn get_job(&self, uid: i32, id: &str) -> Result<Option<AiJobRecord>, String> {
        let sql = format!("SELECT {JOB_COLUMNS} FROM ai_jobs WHERE uid = ? AND id = ?");
        let row = self
            .conn
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                sql,
                vec![uid.into(), id.into()],
            ))
            .await
            .map_err(|e| format!("查询AI任务失败: {e}"))?;
        row.as_ref().map(job_from_row).transpose()
    }

    async fn list_assets(&self, uid: i32, job_id: &str) -> Result<Vec<AiJobAsset>, String> {
        let sql = format!(
            "SELECT {ASSET_COLUMNS} FROM ai_job_assets WHERE uid = ? AND job_id = ? AND deleted_at IS NULL ORDER BY position"
        );
        let rows = self
            .conn
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                sql,
                vec![uid.into(), job_id.into()],
            ))
            .await
            .map_err(|e| format!("查询AI任务图片失败: {e}"))?;
        rows.iter().map(asset_from_row).collect()
    }

    async fn get_asset(&self, uid: i32, asset_id: &str) -> Result<Option<AiJobAsset>, String> {
        let sql = format!(
            "SELECT {ASSET_COLUMNS} FROM ai_job_assets WHERE uid = ? AND id = ? AND deleted_at IS NULL"
        );
        let row = self
            .conn
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                sql,
                vec![uid.into(), asset_id.into()],
            ))
            .await
            .map_err(|e| format!("查询AI任务图片失败: {e}"))?;
        row.as_ref().map(asset_from_row).transpose()
    }

    async fn count_active_jobs(&self, uid: i32) -> Result<i64, String> {
        let row = self
            .conn
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "SELECT COUNT(*) AS count FROM ai_jobs WHERE uid = ? AND status IN ('queued', 'running')",
                vec![uid.into()],
            ))
            .await
            .map_err(|e| format!("查询AI任务数量失败: {e}"))?
            .ok_or_else(|| "查询AI任务数量失败".to_string())?;
        row.try_get("", "count").map_err(|e| e.to_string())
    }

    async fn claim_next_job(
        &self,
        now: i64,
        lease_until: i64,
    ) -> Result<Option<AiJobRecord>, String> {
        for _ in 0..3 {
            let sql = format!(
                "SELECT {JOB_COLUMNS} FROM ai_jobs WHERE status = ? AND (next_attempt_at IS NULL OR next_attempt_at <= ?) ORDER BY created_at LIMIT 1"
            );
            let Some(row) = self
                .conn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Sqlite,
                    sql,
                    vec![JOB_QUEUED.into(), now.into()],
                ))
                .await
                .map_err(|e| format!("领取AI任务失败: {e}"))?
            else {
                return Ok(None);
            };
            let job = job_from_row(&row)?;
            let result = self
                .conn
                .execute(Statement::from_sql_and_values(
                    DbBackend::Sqlite,
                    "UPDATE ai_jobs SET status = ?, attempts = attempts + 1, started_at = COALESCE(started_at, ?), lease_until = ?, next_attempt_at = NULL, updated_at = ? WHERE id = ? AND status = ?",
                    vec![
                        JOB_RUNNING.into(),
                        now.into(),
                        lease_until.into(),
                        now.into(),
                        job.id.clone().into(),
                        JOB_QUEUED.into(),
                    ],
                ))
                .await
                .map_err(|e| format!("领取AI任务失败: {e}"))?;
            if result.rows_affected() == 1 {
                return Ok(Some(AiJobRecord {
                    status: JOB_RUNNING.to_string(),
                    attempts: job.attempts + 1,
                    lease_until: Some(lease_until),
                    next_attempt_at: None,
                    started_at: job.started_at.or(Some(now)),
                    ..job
                }));
            }
        }
        Ok(None)
    }

    async fn requeue_expired_jobs(&self, now: i64) -> Result<(), String> {
        self.conn
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "UPDATE ai_jobs SET status = ?, next_attempt_at = ?, lease_until = NULL, updated_at = ? WHERE status = ? AND lease_until < ?",
                vec![
                    JOB_QUEUED.into(),
                    now.into(),
                    now.into(),
                    JOB_RUNNING.into(),
                    now.into(),
                ],
            ))
            .await
            .map_err(|e| format!("恢复AI任务失败: {e}"))?;
        Ok(())
    }

    async fn mark_job_ready(&self, id: &str, result_json: &str, now: i64) -> Result<(), String> {
        self.conn
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "UPDATE ai_jobs SET status = ?, result_json = ?, error_code = NULL, error_message = NULL, lease_until = NULL, finished_at = ?, updated_at = ? WHERE id = ? AND status = ?",
                vec![
                    JOB_READY.into(),
                    result_json.into(),
                    now.into(),
                    now.into(),
                    id.into(),
                    JOB_RUNNING.into(),
                ],
            ))
            .await
            .map_err(|e| format!("更新AI任务失败: {e}"))?;
        Ok(())
    }

    async fn mark_job_error(
        &self,
        id: &str,
        code: &str,
        message: &str,
        retry_at: Option<i64>,
        now: i64,
    ) -> Result<(), String> {
        let status = if retry_at.is_some() {
            JOB_QUEUED
        } else {
            JOB_FAILED
        };
        self.conn
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "UPDATE ai_jobs SET status = ?, error_code = ?, error_message = ?, next_attempt_at = ?, lease_until = NULL, finished_at = CASE WHEN ? IS NULL THEN ? ELSE finished_at END, updated_at = ? WHERE id = ?",
                vec![
                    status.into(),
                    code.into(),
                    message.into(),
                    retry_at.into(),
                    retry_at.into(),
                    now.into(),
                    now.into(),
                    id.into(),
                ],
            ))
            .await
            .map_err(|e| format!("更新AI任务错误失败: {e}"))?;
        Ok(())
    }

    async fn retry_job(&self, uid: i32, id: &str, now: i64) -> Result<bool, String> {
        let result = self
            .conn
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "UPDATE ai_jobs SET status = ?, attempts = 0, next_attempt_at = ?, error_code = NULL, error_message = NULL, started_at = NULL, finished_at = NULL, updated_at = ? WHERE uid = ? AND id = ? AND status = ?",
                vec![
                    JOB_QUEUED.into(),
                    now.into(),
                    now.into(),
                    uid.into(),
                    id.into(),
                    JOB_FAILED.into(),
                ],
            ))
            .await
            .map_err(|e| format!("重试AI任务失败: {e}"))?;
        Ok(result.rows_affected() == 1)
    }

    async fn delete_job(&self, uid: i32, id: &str) -> Result<bool, String> {
        let id = id.to_string();
        self.conn
            .transaction(|txn| {
                let id = id.clone();
                Box::pin(async move {
                    let result = txn
                        .execute(Statement::from_sql_and_values(
                            DbBackend::Sqlite,
                            "DELETE FROM ai_jobs WHERE uid = ? AND id = ? AND status IN ('queued', 'ready', 'failed')",
                            vec![uid.into(), id.clone().into()],
                        ))
                        .await?;
                    if result.rows_affected() == 0 {
                        return Ok::<bool, sea_orm::DbErr>(false);
                    }
                    txn.execute(Statement::from_sql_and_values(
                        DbBackend::Sqlite,
                        "DELETE FROM ai_job_assets WHERE uid = ? AND job_id = ?",
                        vec![uid.into(), id.into()],
                    ))
                    .await?;
                    Ok::<bool, sea_orm::DbErr>(true)
                })
            })
            .await
            .map_err(|e| format!("删除AI任务失败: {e}"))
    }

    async fn list_assets_for_cleanup(&self, limit: i32) -> Result<Vec<AiJobAsset>, String> {
        let sql = format!(
            "SELECT a.{columns} FROM ai_job_assets a JOIN ai_jobs j ON j.id = a.job_id WHERE j.status = 'submitted' AND a.deleted_at IS NULL ORDER BY j.submitted_at LIMIT ?",
            columns = ASSET_COLUMNS
                .split(", ")
                .map(|column| format!("{column}"))
                .collect::<Vec<_>>()
                .join(", a.")
        );
        let rows = self
            .conn
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                sql,
                vec![limit.clamp(1, 100).into()],
            ))
            .await
            .map_err(|e| format!("查询待清理AI图片失败: {e}"))?;
        rows.iter().map(asset_from_row).collect()
    }

    async fn mark_asset_deleted(&self, id: &str, now: i64) -> Result<(), String> {
        self.conn
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "UPDATE ai_job_assets SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL",
                vec![now.into(), id.into()],
            ))
            .await
            .map_err(|e| format!("更新AI图片清理状态失败: {e}"))?;
        Ok(())
    }
}
