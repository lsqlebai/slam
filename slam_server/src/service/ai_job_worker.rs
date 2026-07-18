use std::fs;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::service::ai_job_service::AIJobService;
use crate::service::ai_service::AIService;
use crate::service::common::ServiceError;
use crate::service::image_service::ImageService;

pub fn start_workers(
    count: usize,
    max_attempts: i32,
    retry_delays_seconds: Vec<u64>,
    jobs: Arc<AIJobService>,
    ai: Arc<AIService>,
    images: Arc<ImageService>,
) {
    for _ in 0..count.max(1) {
        let jobs = jobs.clone();
        let ai = ai.clone();
        let images = images.clone();
        let retry_delays_seconds = retry_delays_seconds.clone();
        tokio::spawn(async move {
            worker_loop(max_attempts.max(1), retry_delays_seconds, jobs, ai, images).await;
        });
    }
}

async fn worker_loop(
    max_attempts: i32,
    retry_delays_seconds: Vec<u64>,
    jobs: Arc<AIJobService>,
    ai: Arc<AIService>,
    images: Arc<ImageService>,
) {
    let notify = jobs.notify();
    loop {
        if let Err(error) = jobs.cleanup_submitted_assets().await {
            tracing::error!(error = %error, "failed to clean submitted AI job assets");
        }
        if let Err(error) = jobs.recover_expired().await {
            tracing::error!(error = %error, "failed to recover expired AI jobs");
        }
        match jobs.claim(600).await {
            Ok(Some(job)) => {
                process_job(
                    max_attempts,
                    &retry_delays_seconds,
                    &jobs,
                    &ai,
                    &images,
                    job,
                )
                .await
            }
            Ok(None) => {
                tokio::select! {
                    _ = notify.notified() => {},
                    _ = tokio::time::sleep(Duration::from_secs(2)) => {},
                }
            }
            Err(error) => {
                tracing::error!(error = %error, "failed to claim AI job");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

async fn process_job(
    max_attempts: i32,
    retry_delays_seconds: &[u64],
    jobs: &AIJobService,
    ai: &AIService,
    images: &ImageService,
    job: crate::model::ai_job::AiJobRecord,
) {
    let result = async {
        let assets = jobs
            .get_assets_for_worker(&job)
            .await
            .map_err(internal_error)?;
        if assets.is_empty() {
            return Err(ServiceError {
                code: 400,
                message: "AI任务没有可用图片".to_string(),
            });
        }
        let mut base64 = Vec::new();
        for asset in assets {
            let bytes = fs::read(&asset.original_path).map_err(|_| ServiceError {
                code: 400,
                message: "AI任务原始图片不存在".to_string(),
            })?;
            let processed = images.process_image(bytes)?;
            base64.extend(processed.base64_data);
        }
        let response = ai.sports_image_recognition(base64).await?;
        response.data.ok_or_else(|| ServiceError {
            code: 500,
            message: "AI服务未返回运动数据".to_string(),
        })
    }
    .await;

    match result {
        Ok(sport) => {
            if let Err(error) = jobs.mark_ready(&job.id, &sport).await {
                tracing::error!(job_id = %job.id, error = %error, "failed to mark AI job ready");
            }
        }
        Err(error) => {
            let authentication_error = error.message.contains("鉴权")
                || error.message.contains("API Key")
                || error.message.contains("API key");
            let retryable = matches!(error.code, 422 | 502 | 504) && !authentication_error;
            let retry_at = if retryable && job.attempts < max_attempts {
                let index = (job.attempts - 1).max(0) as usize;
                let delay = retry_delays_seconds
                    .get(index)
                    .copied()
                    .or_else(|| retry_delays_seconds.last().copied())
                    .unwrap_or(60);
                Some(now_timestamp() + delay as i64)
            } else {
                None
            };
            if let Err(mark_error) = jobs
                .mark_error(&job.id, &error.code.to_string(), &error.message, retry_at)
                .await
            {
                tracing::error!(job_id = %job.id, error = %mark_error, "failed to mark AI job error");
            }
        }
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
