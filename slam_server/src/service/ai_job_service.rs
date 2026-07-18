use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::Notify;
use uuid::Uuid;

use crate::dao::idl::AiJobDao;
use crate::model::ai_job::{
    AiJobAsset, AiJobRecord, AiJobView, JOB_FAILED, JOB_QUEUED, JOB_RUNNING, JOB_SUBMITTED,
};
use crate::model::sport::Sport;
use crate::service::common::ServiceError;
use crate::service::image_service::ImageService;

pub struct JobUpload {
    pub bytes: Vec<u8>,
    pub mime: String,
}

pub struct AIJobService {
    dao: Arc<dyn AiJobDao + Send + Sync>,
    image_service: Arc<ImageService>,
    storage_dir: PathBuf,
    notify: Arc<Notify>,
}

impl AIJobService {
    pub fn new(
        dao: Arc<dyn AiJobDao + Send + Sync>,
        image_service: Arc<ImageService>,
        storage_dir: impl Into<PathBuf>,
        notify: Arc<Notify>,
    ) -> Self {
        Self {
            dao,
            image_service,
            storage_dir: storage_dir.into(),
            notify,
        }
    }

    pub fn notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }

    pub async fn create_job(
        &self,
        uid: i32,
        uploads: Vec<JobUpload>,
    ) -> Result<AiJobView, ServiceError> {
        if uploads.is_empty() {
            return Err(ServiceError {
                code: 400,
                message: "缺少'image'字段".to_string(),
            });
        }
        if self
            .dao
            .count_active_jobs(uid)
            .await
            .map_err(internal_error)?
            >= 3
        {
            return Err(ServiceError {
                code: 429,
                message: "最多同时存在3个排队或识别中的任务".to_string(),
            });
        }

        let now = now_timestamp();
        let job_id = Uuid::new_v4().to_string();
        let temp_dir = self.storage_dir.join(format!(".{job_id}.tmp"));
        let final_dir = self.storage_dir.join(&job_id);
        fs::create_dir_all(&self.storage_dir).map_err(file_error)?;
        fs::create_dir(&temp_dir).map_err(file_error)?;

        let mut assets = Vec::with_capacity(uploads.len());
        let write_result = (|| -> Result<(), ServiceError> {
            for (position, upload) in uploads.iter().enumerate() {
                let thumbnail = self.image_service.create_thumbnail(&upload.bytes)?;
                let asset_id = Uuid::new_v4().to_string();
                let original_path = temp_dir.join(format!("{asset_id}.image"));
                let thumbnail_path = temp_dir.join(format!("{asset_id}.thumb.jpg"));
                fs::write(&original_path, &upload.bytes).map_err(file_error)?;
                fs::write(&thumbnail_path, thumbnail).map_err(file_error)?;
                assets.push(AiJobAsset {
                    id: asset_id,
                    uid,
                    job_id: job_id.clone(),
                    original_path: original_path.to_string_lossy().to_string(),
                    thumbnail_path: thumbnail_path.to_string_lossy().to_string(),
                    mime: normalize_mime(&upload.mime),
                    position: position as i32,
                    created_at: now,
                    deleted_at: None,
                });
            }
            fs::rename(&temp_dir, &final_dir).map_err(file_error)?;
            for asset in &mut assets {
                asset.original_path = replace_parent(&asset.original_path, &final_dir);
                asset.thumbnail_path = replace_parent(&asset.thumbnail_path, &final_dir);
            }
            Ok(())
        })();

        if let Err(error) = write_result {
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(error);
        }

        let job = AiJobRecord {
            id: job_id.clone(),
            uid,
            status: JOB_QUEUED.to_string(),
            result_json: None,
            error_code: None,
            error_message: None,
            attempts: 0,
            next_attempt_at: None,
            lease_until: None,
            submitted_sport_id: None,
            created_at: now,
            started_at: None,
            finished_at: None,
            submitted_at: None,
        };
        if let Err(error) = self.dao.create_job(job.clone(), assets.clone()).await {
            let _ = fs::remove_dir_all(&final_dir);
            return Err(internal_error(error));
        }
        tracing::info!(
            job_id = %job_id,
            uid,
            image_count = assets.len(),
            "AI job created"
        );
        self.notify.notify_one();
        Ok(to_view(job, assets))
    }

    pub async fn list(
        &self,
        uid: i32,
        page: i32,
        size: i32,
    ) -> Result<Vec<AiJobView>, ServiceError> {
        let mut jobs = self
            .dao
            .list_jobs(uid, page, size)
            .await
            .map_err(internal_error)?;
        let has_active = jobs
            .iter()
            .any(|job| job.status == JOB_QUEUED || job.status == JOB_RUNNING);
        if has_active {
            self.dao
                .requeue_expired_jobs(now_timestamp())
                .await
                .map_err(internal_error)?;
            self.notify.notify_one();
            jobs = self
                .dao
                .list_jobs(uid, page, size)
                .await
                .map_err(internal_error)?;
        }
        let mut result = Vec::with_capacity(jobs.len());
        for job in jobs {
            let assets = self
                .dao
                .list_assets(uid, &job.id)
                .await
                .map_err(internal_error)?;
            result.push(to_view(job, assets));
        }
        Ok(result)
    }

    pub async fn get(&self, uid: i32, id: &str) -> Result<AiJobView, ServiceError> {
        let job = self
            .dao
            .get_job(uid, id)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| ServiceError {
                code: 404,
                message: "AI任务不存在".to_string(),
            })?;
        let assets = self
            .dao
            .list_assets(uid, id)
            .await
            .map_err(internal_error)?;
        Ok(to_view(job, assets))
    }

    pub async fn retry(&self, uid: i32, id: &str) -> Result<(), ServiceError> {
        let job = self
            .dao
            .get_job(uid, id)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| ServiceError {
                code: 404,
                message: "AI任务不存在".to_string(),
            })?;
        if job.status != JOB_FAILED {
            return Err(ServiceError {
                code: 409,
                message: "只有失败的AI任务可以重试".to_string(),
            });
        }
        let now = now_timestamp();
        if !self
            .dao
            .retry_job(uid, id, now)
            .await
            .map_err(internal_error)?
        {
            return Err(ServiceError {
                code: 409,
                message: "AI任务状态已变化，请刷新后重试".to_string(),
            });
        }
        tracing::info!(job_id = %id, uid, "AI job manually queued for retry");
        self.notify.notify_one();
        Ok(())
    }

    pub async fn delete(&self, uid: i32, id: &str) -> Result<(), ServiceError> {
        let job = self
            .dao
            .get_job(uid, id)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| ServiceError {
                code: 404,
                message: "AI任务不存在".to_string(),
            })?;
        if job.status == JOB_RUNNING {
            return Err(ServiceError {
                code: 409,
                message: "识别中的AI任务暂时不能删除，请稍后重试".to_string(),
            });
        }
        if job.status == JOB_SUBMITTED {
            return Err(ServiceError {
                code: 409,
                message: "已提交的AI任务不能删除，请删除对应运动记录".to_string(),
            });
        }

        let assets = self
            .dao
            .list_assets(uid, id)
            .await
            .map_err(internal_error)?;
        if !self.dao.delete_job(uid, id).await.map_err(internal_error)? {
            return Err(ServiceError {
                code: 409,
                message: "AI任务状态已变化，请刷新后重试".to_string(),
            });
        }

        let asset_count = assets.len();
        for asset in assets {
            let original_deleted = remove_if_present(&asset.original_path);
            let thumbnail_deleted = remove_if_present(&asset.thumbnail_path);
            if !original_deleted || !thumbnail_deleted {
                tracing::warn!(
                    job_id = %id,
                    asset_id = %asset.id,
                    original_deleted,
                    thumbnail_deleted,
                    "AI job deleted but an asset file could not be removed"
                );
            }
            if let Some(parent) = Path::new(&asset.original_path).parent() {
                let _ = fs::remove_dir(parent);
            }
        }
        tracing::info!(
            job_id = %id,
            uid,
            status = %job.status,
            asset_count,
            "AI job deleted"
        );
        Ok(())
    }

    pub async fn read_asset(
        &self,
        uid: i32,
        asset_id: &str,
        thumbnail: bool,
    ) -> Result<(Vec<u8>, String), ServiceError> {
        let asset = self
            .dao
            .get_asset(uid, asset_id)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| ServiceError {
                code: 404,
                message: "图片不存在".to_string(),
            })?;
        let path = if thumbnail {
            &asset.thumbnail_path
        } else {
            &asset.original_path
        };
        let bytes = fs::read(path).map_err(|_| ServiceError {
            code: 404,
            message: "图片文件不存在".to_string(),
        })?;
        Ok((
            bytes,
            if thumbnail {
                "image/jpeg".to_string()
            } else {
                asset.mime
            },
        ))
    }

    pub async fn get_assets_for_worker(
        &self,
        job: &AiJobRecord,
    ) -> Result<Vec<AiJobAsset>, String> {
        self.dao.list_assets(job.uid, &job.id).await
    }

    pub async fn claim(&self, lease_seconds: i64) -> Result<Option<AiJobRecord>, String> {
        let now = now_timestamp();
        self.dao.claim_next_job(now, now + lease_seconds).await
    }

    pub async fn recover_expired(&self) -> Result<(), String> {
        self.dao.requeue_expired_jobs(now_timestamp()).await
    }

    pub async fn mark_ready(&self, id: &str, sport: &Sport) -> Result<(), String> {
        let json = serde_json::to_string(sport).map_err(|e| e.to_string())?;
        self.dao.mark_job_ready(id, &json, now_timestamp()).await
    }

    pub async fn mark_error(
        &self,
        id: &str,
        code: &str,
        message: &str,
        retry_at: Option<i64>,
    ) -> Result<(), String> {
        self.dao
            .mark_job_error(id, code, message, retry_at, now_timestamp())
            .await
    }

    pub async fn cleanup_submitted_assets(&self) -> Result<(), String> {
        let assets = self.dao.list_assets_for_cleanup(20).await?;
        for asset in assets {
            let original_deleted = remove_if_present(&asset.original_path);
            let thumbnail_deleted = remove_if_present(&asset.thumbnail_path);
            if original_deleted && thumbnail_deleted {
                self.dao
                    .mark_asset_deleted(&asset.id, now_timestamp())
                    .await?;
                if let Some(parent) = Path::new(&asset.original_path).parent() {
                    let _ = fs::remove_dir(parent);
                }
            }
        }
        Ok(())
    }

    pub fn cleanup_paths(paths: Vec<String>) {
        let mut parents = std::collections::HashSet::new();
        for path in paths {
            if let Some(parent) = Path::new(&path).parent() {
                parents.insert(parent.to_path_buf());
            }
            let _ = fs::remove_file(path);
        }
        for parent in parents {
            let _ = fs::remove_dir(parent);
        }
    }
}

fn to_view(job: AiJobRecord, assets: Vec<AiJobAsset>) -> AiJobView {
    let result = job
        .result_json
        .as_deref()
        .and_then(|json| serde_json::from_str(json).ok());
    AiJobView {
        id: job.id,
        status: job.status,
        result,
        error_code: job.error_code,
        error_message: job.error_message,
        attempts: job.attempts,
        next_attempt_at: job.next_attempt_at,
        submitted_sport_id: job.submitted_sport_id,
        created_at: job.created_at,
        started_at: job.started_at,
        finished_at: job.finished_at,
        submitted_at: job.submitted_at,
        assets,
    }
}

fn replace_parent(path: &str, parent: &Path) -> String {
    let name = Path::new(path)
        .file_name()
        .expect("generated asset path has a file name");
    parent.join(name).to_string_lossy().to_string()
}

fn normalize_mime(value: &str) -> String {
    match value {
        "image/png" | "image/webp" | "image/gif" => value.to_string(),
        _ => "image/jpeg".to_string(),
    }
}

fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn internal_error(message: String) -> ServiceError {
    ServiceError { code: 500, message }
}

fn file_error(error: std::io::Error) -> ServiceError {
    ServiceError {
        code: 500,
        message: format!("保存AI任务图片失败: {error}"),
    }
}

fn remove_if_present(path: &str) -> bool {
    match fs::remove_file(path) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(_) => false,
    }
}

pub fn is_failed(job: &AiJobView) -> bool {
    job.status == JOB_FAILED
}
