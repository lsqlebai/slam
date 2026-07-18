use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use reqwest::multipart;
use slam_server::app::{self, AppConfig, routes};
use slam_server::dao::Repository;
use slam_server::dao::idl::AiJobDao;
use slam_server::model::ai_job::{AiJobRecord, JOB_QUEUED};
use slam_server::model::sport::SAMPLE_XML_SWIMMING;
use slam_server::service::ai_job_service::AIJobService;
use slam_server::service::image_service::ImageService;
use slam_server::service::llm::{ChatCompletionRequest, LLM, LLMError};
use tempfile::TempDir;
use tokio::sync::Notify;
use tower::Service;

struct MockLlm {
    responses: Mutex<VecDeque<Result<String, LLMError>>>,
}

impl MockLlm {
    fn new(responses: Vec<Result<String, LLMError>>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
        }
    }
}

#[async_trait]
impl LLM for MockLlm {
    async fn chat(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match self.responses.lock().unwrap().pop_front() {
            Some(Ok(value)) => Ok(value),
            Some(Err(error)) => Err(Box::new(error)),
            None => Err(Box::new(LLMError::InternalError(
                "mock response exhausted".to_string(),
            ))),
        }
    }
}

fn isolated_config(temp: &TempDir, max_attempts: i32) -> AppConfig {
    let mut config = AppConfig::default();
    config.db.path = temp.path().join("sport.db").to_string_lossy().to_string();
    config.ai.job_dir = temp.path().join("ai-jobs").to_string_lossy().to_string();
    config.ai.worker_concurrency = 1;
    config.ai.max_attempts = max_attempts;
    config.ai.retry_delays_seconds = vec![0, 0];
    config
}

async fn response_json(response: axum::response::Response) -> (StatusCode, serde_json::Value) {
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    let value = serde_json::from_slice(&bytes).unwrap_or_else(
        |_| serde_json::json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }),
    );
    (status, value)
}

async fn register(app: &mut axum::Router, name: &str) -> String {
    let request = Request::builder()
        .uri(routes::API_USER_REGISTER)
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "name": name,
                "password": "p@ssw0rd",
                "nickname": name
            })
            .to_string(),
        ))
        .unwrap();
    let response = app.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

async fn create_job(app: &mut axum::Router, cookie: &str) -> serde_json::Value {
    let image = std::fs::read("tests/test_img/test1.jpg").unwrap();
    let form = multipart::Form::new().part(
        "image",
        multipart::Part::bytes(image)
            .file_name("sport.jpg")
            .mime_str("image/jpeg")
            .unwrap(),
    );
    let boundary = form.boundary().to_string();
    let request = Request::builder()
        .uri(routes::API_AI_JOBS)
        .method("POST")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("cookie", cookie)
        .body(Body::from_stream(form.into_stream()))
        .unwrap();
    let (status, body) = response_json(app.call(request).await.unwrap()).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    body
}

async fn get_job(
    app: &mut axum::Router,
    cookie: &str,
    id: &str,
) -> (StatusCode, serde_json::Value) {
    let request = Request::builder()
        .uri(routes::API_AI_JOB.replace(":id", id))
        .method("GET")
        .header("cookie", cookie)
        .body(Body::empty())
        .unwrap();
    response_json(app.call(request).await.unwrap()).await
}

async fn wait_for_status(
    app: &mut axum::Router,
    cookie: &str,
    id: &str,
    expected: &str,
) -> serde_json::Value {
    for _ in 0..100 {
        let (status, job) = get_job(app, cookie, id).await;
        assert_eq!(status, StatusCode::OK, "{job}");
        if job["status"] == expected {
            return job;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("job {id} did not reach {expected}");
}

#[tokio::test]
async fn ai_job_success_submit_is_idempotent_and_assets_are_private() {
    let temp = TempDir::new().unwrap();
    let mock = Arc::new(MockLlm::new(vec![Ok(SAMPLE_XML_SWIMMING.to_string())]));
    let mut app = app::create_app_with_llm(isolated_config(&temp, 3), mock).await;
    let cookie = register(&mut app, "ai_job_owner").await;
    let other_cookie = register(&mut app, "ai_job_other").await;

    let created = create_job(&mut app, &cookie).await;
    let id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "queued");
    let asset_id = created["assets"][0]["id"].as_str().unwrap();

    let (other_status, _) = get_job(&mut app, &other_cookie, id).await;
    assert_eq!(other_status, StatusCode::NOT_FOUND);

    let unauthorized_asset = Request::builder()
        .uri(routes::API_AI_ASSET_THUMBNAIL.replace(":id", asset_id))
        .method("GET")
        .header("cookie", &other_cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.call(unauthorized_asset).await.unwrap().status(),
        StatusCode::NOT_FOUND
    );

    let ready = wait_for_status(&mut app, &cookie, id, "ready").await;
    assert_eq!(ready["result"]["type"], "Swimming");
    let mut sport = ready["result"].clone();
    sport["distance_meter"] = serde_json::json!(1250);
    sport["ai_job_id"] = serde_json::json!(id);

    let submit = |body: &serde_json::Value| {
        Request::builder()
            .uri(routes::API_SPORT_INSERT)
            .method("POST")
            .header("content-type", "application/json")
            .header("cookie", &cookie)
            .body(Body::from(body.to_string()))
            .unwrap()
    };
    let (status, first) = response_json(app.call(submit(&sport)).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    let sport_id = first["id"].as_i64().unwrap();

    let (status, second) = response_json(app.call(submit(&sport)).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{second}");
    assert_eq!(second["id"].as_i64().unwrap(), sport_id);

    for _ in 0..100 {
        let (_, submitted) = get_job(&mut app, &cookie, id).await;
        if submitted["status"] == "submitted" && submitted["assets"].as_array().unwrap().is_empty()
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let (_, submitted) = get_job(&mut app, &cookie, id).await;
    assert_eq!(submitted["status"], "submitted");
    assert!(submitted["assets"].as_array().unwrap().is_empty());

    let asset_after_submit = Request::builder()
        .uri(routes::API_AI_ASSET.replace(":id", asset_id))
        .method("GET")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.call(asset_after_submit).await.unwrap().status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn expired_running_job_is_recovered_after_restart_scan() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("sport.db");
    let repository = Repository::new(db_path.to_str().unwrap()).await.unwrap();
    let now = chrono::Utc::now().timestamp();
    repository
        .create_job(
            AiJobRecord {
                id: "recover-job".to_string(),
                uid: 7,
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
            },
            Vec::new(),
        )
        .await
        .unwrap();
    let claimed = repository
        .claim_next_job(now, now - 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(claimed.status, "running");

    repository.requeue_expired_jobs(now).await.unwrap();
    let recovered = repository.get_job(7, "recover-job").await.unwrap().unwrap();
    assert_eq!(recovered.status, "queued");
}

#[tokio::test]
async fn active_job_list_poll_recovers_expired_job_and_wakes_worker() {
    let temp = TempDir::new().unwrap();
    let repository = Arc::new(
        Repository::new(temp.path().join("sport.db").to_str().unwrap())
            .await
            .unwrap(),
    );
    let now = chrono::Utc::now().timestamp();
    repository
        .create_job(
            AiJobRecord {
                id: "poll-recover-job".to_string(),
                uid: 7,
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
            },
            Vec::new(),
        )
        .await
        .unwrap();
    repository
        .claim_next_job(now, now - 1)
        .await
        .unwrap()
        .unwrap();

    let notify = Arc::new(Notify::new());
    let service = AIJobService::new(
        repository.clone(),
        Arc::new(ImageService::new()),
        temp.path().join("ai-jobs"),
        notify.clone(),
    );
    let jobs = service.list(7, 0, 50).await.unwrap();

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, JOB_QUEUED);
    tokio::time::timeout(Duration::from_millis(50), notify.notified())
        .await
        .expect("poll should wake the worker");
}

#[tokio::test]
async fn failed_job_can_be_retried_from_job_api() {
    let temp = TempDir::new().unwrap();
    let mock = Arc::new(MockLlm::new(vec![
        Err(LLMError::APIFailure("temporary upstream error".to_string())),
        Ok(SAMPLE_XML_SWIMMING.to_string()),
    ]));
    let mut app = app::create_app_with_llm(isolated_config(&temp, 1), mock).await;
    let cookie = register(&mut app, "ai_job_retry").await;
    let created = create_job(&mut app, &cookie).await;
    let id = created["id"].as_str().unwrap();

    let failed = wait_for_status(&mut app, &cookie, id, "failed").await;
    assert_eq!(failed["attempts"], 1);
    assert!(
        failed["error_message"]
            .as_str()
            .unwrap()
            .contains("temporary")
    );

    let retry = Request::builder()
        .uri(routes::API_AI_JOB_RETRY.replace(":id", id))
        .method("POST")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    let (status, body) = response_json(app.call(retry).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let ready = wait_for_status(&mut app, &cookie, id, "ready").await;
    assert_eq!(ready["attempts"], 1);

    let retry_ready = Request::builder()
        .uri(routes::API_AI_JOB_RETRY.replace(":id", id))
        .method("POST")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.call(retry_ready).await.unwrap().status(),
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn retryable_failure_runs_automatically_until_ready() {
    let temp = TempDir::new().unwrap();
    let mock = Arc::new(MockLlm::new(vec![
        Err(LLMError::TimeoutError("mock timeout".to_string())),
        Ok(SAMPLE_XML_SWIMMING.to_string()),
    ]));
    let mut app = app::create_app_with_llm(isolated_config(&temp, 3), mock).await;
    let cookie = register(&mut app, "ai_job_auto_retry").await;
    let created = create_job(&mut app, &cookie).await;
    let id = created["id"].as_str().unwrap();

    let ready = wait_for_status(&mut app, &cookie, id, "ready").await;
    assert_eq!(ready["attempts"], 2);
}

#[tokio::test]
async fn ai_job_creation_requires_authentication_and_valid_image() {
    let temp = TempDir::new().unwrap();
    let mock = Arc::new(MockLlm::new(Vec::new()));
    let mut app = app::create_app_with_llm(isolated_config(&temp, 1), mock).await;

    let form = multipart::Form::new().part(
        "image",
        multipart::Part::bytes(b"not-an-image".to_vec())
            .file_name("bad.jpg")
            .mime_str("image/jpeg")
            .unwrap(),
    );
    let boundary = form.boundary().to_string();
    let unauthenticated = Request::builder()
        .uri(routes::API_AI_JOBS)
        .method("POST")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from_stream(form.into_stream()))
        .unwrap();
    assert_eq!(
        app.call(unauthenticated).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );

    let cookie = register(&mut app, "ai_job_invalid_image").await;
    let form = multipart::Form::new().part(
        "image",
        multipart::Part::bytes(b"not-an-image".to_vec())
            .file_name("bad.jpg")
            .mime_str("image/jpeg")
            .unwrap(),
    );
    let boundary = form.boundary().to_string();
    let invalid = Request::builder()
        .uri(routes::API_AI_JOBS)
        .method("POST")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("cookie", &cookie)
        .body(Body::from_stream(form.into_stream()))
        .unwrap();
    assert_eq!(
        app.call(invalid).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );

    let list = Request::builder()
        .uri(routes::API_AI_JOBS)
        .method("GET")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    let (status, jobs) = response_json(app.call(list).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(jobs, serde_json::json!([]));
}

#[tokio::test]
async fn authentication_failure_is_not_retried_automatically() {
    let temp = TempDir::new().unwrap();
    let mock = Arc::new(MockLlm::new(vec![Err(LLMError::LLMAuthenticationError(
        "鉴权失败: API Key 无效".to_string(),
    ))]));
    let mut app = app::create_app_with_llm(isolated_config(&temp, 3), mock).await;
    let cookie = register(&mut app, "ai_job_auth_error").await;
    let created = create_job(&mut app, &cookie).await;
    let id = created["id"].as_str().unwrap();

    let failed = wait_for_status(&mut app, &cookie, id, "failed").await;
    assert_eq!(failed["attempts"], 1);
    assert!(failed["next_attempt_at"].is_null());
}
