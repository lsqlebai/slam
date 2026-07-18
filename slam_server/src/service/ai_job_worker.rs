use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
    let worker_count = count.max(1);
    tracing::info!(
        worker_count,
        max_attempts = max_attempts.max(1),
        retry_delays_seconds = ?retry_delays_seconds,
        "starting AI job workers"
    );
    for worker_id in 0..worker_count {
        let jobs = jobs.clone();
        let ai = ai.clone();
        let images = images.clone();
        let retry_delays_seconds = retry_delays_seconds.clone();
        tokio::spawn(async move {
            worker_loop(
                worker_id,
                max_attempts.max(1),
                retry_delays_seconds,
                jobs,
                ai,
                images,
            )
            .await;
        });
    }
}

async fn worker_loop(
    worker_id: usize,
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
                tracing::info!(
                    worker_id,
                    job_id = %job.id,
                    uid = job.uid,
                    attempt = job.attempts,
                    "AI job claimed"
                );
                process_job(
                    worker_id,
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
    worker_id: usize,
    max_attempts: i32,
    retry_delays_seconds: &[u64],
    jobs: &AIJobService,
    ai: &AIService,
    images: &ImageService,
    job: crate::model::ai_job::AiJobRecord,
) {
    let started = Instant::now();
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
                tracing::error!(worker_id, job_id = %job.id, error = %error, "failed to mark AI job ready");
            } else {
                tracing::info!(
                    worker_id,
                    job_id = %job.id,
                    uid = job.uid,
                    attempt = job.attempts,
                    elapsed_ms = started.elapsed().as_millis(),
                    "AI job completed"
                );
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
            tracing::warn!(
                worker_id,
                job_id = %job.id,
                uid = job.uid,
                attempt = job.attempts,
                error_code = error.code,
                error_message = %log_error_message(&error.message),
                retry_at,
                elapsed_ms = started.elapsed().as_millis(),
                "AI job processing failed"
            );
            if let Err(mark_error) = jobs
                .mark_error(&job.id, &error.code.to_string(), &error.message, retry_at)
                .await
            {
                tracing::error!(job_id = %job.id, error = %mark_error, "failed to mark AI job error");
            }
        }
    }
}

fn log_error_message(message: &str) -> String {
    message
        .chars()
        .filter(|character| !character.is_control())
        .take(500)
        .collect()
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
