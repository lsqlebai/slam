use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use reqwest::multipart;
use slam_server::app::{self, AppConfig, routes};
use slam_server::dao::Repository;
use slam_server::dao::idl::AiJobDao;
use slam_server::model::ai_job::{
    AiJobRecord, JOB_FAILED, JOB_QUEUED, JOB_READY, JOB_RUNNING, JOB_SUBMITTED,
};
use slam_server::model::sport::{SAMPLE_XML_SWIMMING, Sport};
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
    create_job_with_image_count(app, cookie, 1).await
}

async fn create_job_with_image_count(
    app: &mut axum::Router,
    cookie: &str,
    image_count: usize,
) -> serde_json::Value {
    let image = std::fs::read("tests/test_img/test1.jpg").unwrap();
    let mut form = multipart::Form::new();
    for index in 0..image_count {
        form = form.part(
            "image",
            multipart::Part::bytes(image.clone())
                .file_name(format!("sport-{index}.jpg"))
                .mime_str("image/jpeg")
                .unwrap(),
        );
    }
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

async fn list_jobs(
    app: &mut axum::Router,
    cookie: &str,
    query: &str,
) -> (StatusCode, serde_json::Value) {
    let request = Request::builder()
        .uri(format!("{}{query}", routes::API_AI_JOBS))
        .method("GET")
        .header("cookie", cookie)
        .body(Body::empty())
        .unwrap();
    response_json(app.call(request).await.unwrap()).await
}

async fn seed_job(repository: &Repository, uid: i32, id: &str, status: &str, created_at: i64) {
    repository
        .create_job(
            AiJobRecord {
                id: id.to_string(),
                uid,
                status: status.to_string(),
                result_json: (status == JOB_READY).then(|| {
                    serde_json::to_string(&Sport::parse_from_xml(SAMPLE_XML_SWIMMING).unwrap())
                        .unwrap()
                }),
                error_code: (status == JOB_FAILED).then(|| "mock_error".to_string()),
                error_message: (status == JOB_FAILED).then(|| "mock failure".to_string()),
                attempts: 0,
                next_attempt_at: None,
                lease_until: None,
                submitted_sport_id: None,
                created_at,
                started_at: None,
                finished_at: None,
                submitted_at: None,
            },
            Vec::new(),
        )
        .await
        .unwrap();
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

async fn delete_job(
    app: &mut axum::Router,
    cookie: Option<&str>,
    id: &str,
) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder()
        .uri(routes::API_AI_JOB.replace(":id", id))
        .method("DELETE");
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    let request = builder.body(Body::empty()).unwrap();
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
    let unauthenticated_asset = Request::builder()
        .uri(routes::API_AI_ASSET.replace(":id", asset_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.call(unauthenticated_asset).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    let missing_asset = Request::builder()
        .uri(routes::API_AI_ASSET.replace(":id", "missing-asset"))
        .method("GET")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.call(missing_asset).await.unwrap().status(),
        StatusCode::NOT_FOUND
    );

    let original_asset = Request::builder()
        .uri(routes::API_AI_ASSET.replace(":id", asset_id))
        .method("GET")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    let original_response = app.call(original_asset).await.unwrap();
    assert_eq!(original_response.status(), StatusCode::OK);
    assert_eq!(
        original_response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap(),
        "image/jpeg"
    );
    assert_eq!(
        original_response
            .headers()
            .get(header::CACHE_CONTROL)
            .unwrap(),
        "private, max-age=300"
    );
    let original_bytes = to_bytes(original_response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(
        original_bytes.as_ref(),
        std::fs::read("tests/test_img/test1.jpg").unwrap()
    );

    let thumbnail_asset = Request::builder()
        .uri(routes::API_AI_ASSET_THUMBNAIL.replace(":id", asset_id))
        .method("GET")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    let thumbnail_response = app.call(thumbnail_asset).await.unwrap();
    assert_eq!(thumbnail_response.status(), StatusCode::OK);
    assert_eq!(
        thumbnail_response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap(),
        "image/jpeg"
    );
    let thumbnail_bytes = to_bytes(thumbnail_response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    assert!(!thumbnail_bytes.is_empty());
    assert_eq!(&thumbnail_bytes[..2], &[0xff, 0xd8]);

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
    let config = isolated_config(&temp, 1);
    let repository = Repository::new(&config.db.path).await.unwrap();
    let now = chrono::Utc::now().timestamp();
    seed_job(&repository, 1, "recover-job", JOB_QUEUED, now).await;
    let claimed = repository
        .claim_next_job(now, now - 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(claimed.status, "running");

    let mock = Arc::new(MockLlm::new(Vec::new()));
    let mut app = app::create_app_with_llm(config, mock).await;
    let cookie = register(&mut app, "restart_recovery_owner").await;
    let recovered = wait_for_status(&mut app, &cookie, "recover-job", "failed").await;
    assert_eq!(recovered["attempts"], 2);
    assert!(
        recovered["error_message"]
            .as_str()
            .unwrap()
            .contains("没有可用图片")
    );
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
async fn ai_job_list_api_paginates_orders_and_isolates_users() {
    let temp = TempDir::new().unwrap();
    let config = isolated_config(&temp, 1);
    let mock = Arc::new(MockLlm::new(Vec::new()));
    let mut app = app::create_app_with_llm(config.clone(), mock).await;
    let owner_cookie = register(&mut app, "list_owner").await;
    let other_cookie = register(&mut app, "list_other").await;
    let repository = Repository::new(&config.db.path).await.unwrap();

    seed_job(&repository, 1, "owner-old", JOB_FAILED, 100).await;
    seed_job(&repository, 1, "owner-middle", JOB_FAILED, 200).await;
    seed_job(&repository, 1, "owner-new", JOB_FAILED, 300).await;
    seed_job(&repository, 1, "owner-submitted", JOB_SUBMITTED, 400).await;
    seed_job(&repository, 2, "other-newest", JOB_FAILED, 500).await;

    let unauthenticated = Request::builder()
        .uri(routes::API_AI_JOBS)
        .method("GET")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.call(unauthenticated).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );

    let (status, first_page) = list_jobs(&mut app, &owner_cookie, "?page=0&size=2").await;
    assert_eq!(status, StatusCode::OK, "{first_page}");
    assert_eq!(first_page.as_array().unwrap().len(), 2);
    assert_eq!(first_page[0]["id"], "owner-new");
    assert_eq!(first_page[1]["id"], "owner-middle");

    let (status, second_page) = list_jobs(&mut app, &owner_cookie, "?page=1&size=2").await;
    assert_eq!(status, StatusCode::OK, "{second_page}");
    assert_eq!(second_page.as_array().unwrap().len(), 1);
    assert_eq!(second_page[0]["id"], "owner-old");

    let (status, other_jobs) = list_jobs(&mut app, &other_cookie, "?page=0&size=50").await;
    assert_eq!(status, StatusCode::OK, "{other_jobs}");
    assert_eq!(other_jobs.as_array().unwrap().len(), 1);
    assert_eq!(other_jobs[0]["id"], "other-newest");
}

#[tokio::test]
async fn active_job_list_api_recovers_expired_job_and_wakes_worker() {
    let temp = TempDir::new().unwrap();
    let config = isolated_config(&temp, 1);
    let mock = Arc::new(MockLlm::new(Vec::new()));
    let mut app = app::create_app_with_llm(config.clone(), mock).await;
    let cookie = register(&mut app, "poll_api_owner").await;
    let repository = Repository::new(&config.db.path).await.unwrap();
    let now = chrono::Utc::now().timestamp();
    seed_job(&repository, 1, "poll-api-job", JOB_QUEUED, now).await;
    repository
        .claim_next_job(now, now - 1)
        .await
        .unwrap()
        .unwrap();

    let (status, jobs) = list_jobs(&mut app, &cookie, "?page=0&size=50").await;
    assert_eq!(status, StatusCode::OK, "{jobs}");
    assert_eq!(jobs.as_array().unwrap().len(), 1);
    assert_eq!(jobs[0]["id"], "poll-api-job");
    assert!(
        jobs[0]["status"] != "running" || jobs[0]["attempts"] == 2,
        "expired running lease was returned without recovery: {jobs}"
    );

    let failed = wait_for_status(&mut app, &cookie, "poll-api-job", "failed").await;
    assert_eq!(failed["attempts"], 2);
}

#[tokio::test]
async fn ai_job_detail_and_retry_enforce_authentication_ownership_and_existence() {
    let temp = TempDir::new().unwrap();
    let config = isolated_config(&temp, 1);
    let mock = Arc::new(MockLlm::new(Vec::new()));
    let mut app = app::create_app_with_llm(config.clone(), mock).await;
    let owner_cookie = register(&mut app, "permission_owner").await;
    let other_cookie = register(&mut app, "permission_other").await;
    let repository = Repository::new(&config.db.path).await.unwrap();
    seed_job(&repository, 1, "owner-failed", JOB_FAILED, 100).await;

    let unauthenticated_detail = Request::builder()
        .uri(routes::API_AI_JOB.replace(":id", "owner-failed"))
        .method("GET")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.call(unauthenticated_detail).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );

    let (status, _) = get_job(&mut app, &other_cookie, "owner-failed").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = get_job(&mut app, &owner_cookie, "missing-job").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let retry_request = |cookie: Option<&str>, id: &str| {
        let mut builder = Request::builder()
            .uri(routes::API_AI_JOB_RETRY.replace(":id", id))
            .method("POST");
        if let Some(cookie) = cookie {
            builder = builder.header("cookie", cookie);
        }
        builder.body(Body::empty()).unwrap()
    };
    assert_eq!(
        app.call(retry_request(None, "owner-failed"))
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        app.call(retry_request(Some(&other_cookie), "owner-failed"))
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.call(retry_request(Some(&owner_cookie), "missing-job"))
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.call(retry_request(Some(&owner_cookie), "owner-failed"))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn ai_job_delete_enforces_ownership_status_and_removes_draft_assets() {
    let temp = TempDir::new().unwrap();
    let config = isolated_config(&temp, 1);
    let mock = Arc::new(MockLlm::new(vec![Ok(SAMPLE_XML_SWIMMING.to_string())]));
    let mut app = app::create_app_with_llm(config.clone(), mock).await;
    let owner_cookie = register(&mut app, "delete_owner").await;
    let other_cookie = register(&mut app, "delete_other").await;
    let repository = Repository::new(&config.db.path).await.unwrap();

    let created = create_job(&mut app, &owner_cookie).await;
    let draft_id = created["id"].as_str().unwrap();
    wait_for_status(&mut app, &owner_cookie, draft_id, JOB_READY).await;
    let asset_id = created["assets"][0]["id"].as_str().unwrap();
    let asset = repository
        .get_asset(1, asset_id)
        .await
        .unwrap()
        .expect("draft asset should exist");
    assert!(std::path::Path::new(&asset.original_path).exists());
    assert!(std::path::Path::new(&asset.thumbnail_path).exists());

    assert_eq!(
        delete_job(&mut app, None, draft_id).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        delete_job(&mut app, Some(&other_cookie), draft_id).await.0,
        StatusCode::NOT_FOUND
    );

    seed_job(&repository, 1, "running-delete", JOB_RUNNING, 100).await;
    seed_job(&repository, 1, "submitted-delete", JOB_SUBMITTED, 101).await;
    assert_eq!(
        delete_job(&mut app, Some(&owner_cookie), "running-delete")
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        delete_job(&mut app, Some(&owner_cookie), "submitted-delete")
            .await
            .0,
        StatusCode::CONFLICT
    );

    let (status, body) = delete_job(&mut app, Some(&owner_cookie), draft_id).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, serde_json::json!({ "success": true }));
    assert_eq!(
        get_job(&mut app, &owner_cookie, draft_id).await.0,
        StatusCode::NOT_FOUND
    );
    assert!(repository.get_asset(1, asset_id).await.unwrap().is_none());
    assert!(!std::path::Path::new(&asset.original_path).exists());
    assert!(!std::path::Path::new(&asset.thumbnail_path).exists());

    let deleted_asset = Request::builder()
        .uri(routes::API_AI_ASSET_THUMBNAIL.replace(":id", asset_id))
        .method("GET")
        .header("cookie", &owner_cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.call(deleted_asset).await.unwrap().status(),
        StatusCode::NOT_FOUND
    );

    seed_job(&repository, 1, "failed-delete", JOB_FAILED, 102).await;
    assert_eq!(
        delete_job(&mut app, Some(&owner_cookie), "failed-delete")
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        delete_job(&mut app, Some(&owner_cookie), "failed-delete")
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn queued_ai_job_can_be_deleted_before_a_worker_claims_it() {
    let temp = TempDir::new().unwrap();
    let config = isolated_config(&temp, 1);
    let repository = Arc::new(Repository::new(&config.db.path).await.unwrap());
    seed_job(&repository, 7, "queued-delete", JOB_QUEUED, 100).await;
    let service = AIJobService::new(
        repository.clone(),
        Arc::new(ImageService::new()),
        &config.ai.job_dir,
        Arc::new(Notify::new()),
    );

    service.delete(7, "queued-delete").await.unwrap();

    assert!(
        repository
            .get_job(7, "queued-delete")
            .await
            .unwrap()
            .is_none()
    );
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
async fn ai_job_creation_accepts_multiple_images_and_rejects_empty_or_oversized_uploads() {
    let temp = TempDir::new().unwrap();
    let mock = Arc::new(MockLlm::new(vec![Ok(SAMPLE_XML_SWIMMING.to_string())]));
    let mut app = app::create_app_with_llm(isolated_config(&temp, 1), mock).await;
    let cookie = register(&mut app, "ai_job_upload_boundaries").await;

    let created = create_job_with_image_count(&mut app, &cookie, 2).await;
    assert_eq!(created["assets"].as_array().unwrap().len(), 2);
    assert_eq!(created["assets"][0]["position"], 0);
    assert_eq!(created["assets"][1]["position"], 1);
    let id = created["id"].as_str().unwrap();
    wait_for_status(&mut app, &cookie, id, "ready").await;

    let empty_form = multipart::Form::new();
    let boundary = empty_form.boundary().to_string();
    let empty_upload = Request::builder()
        .uri(routes::API_AI_JOBS)
        .method("POST")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("cookie", &cookie)
        .body(Body::from_stream(empty_form.into_stream()))
        .unwrap();
    assert_eq!(
        app.call(empty_upload).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );

    let oversized_form = multipart::Form::new().part(
        "image",
        multipart::Part::bytes(vec![0_u8; 50 * 1024 * 1024 + 1])
            .file_name("oversized.jpg")
            .mime_str("image/jpeg")
            .unwrap(),
    );
    let boundary = oversized_form.boundary().to_string();
    let oversized_upload = Request::builder()
        .uri(routes::API_AI_JOBS)
        .method("POST")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("cookie", &cookie)
        .body(Body::from_stream(oversized_form.into_stream()))
        .unwrap();
    assert_eq!(
        app.call(oversized_upload).await.unwrap().status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn ai_job_creation_limits_each_user_to_three_active_jobs() {
    let temp = TempDir::new().unwrap();
    let config = isolated_config(&temp, 1);
    let mock = Arc::new(MockLlm::new(Vec::new()));
    let mut app = app::create_app_with_llm(config.clone(), mock).await;
    let cookie = register(&mut app, "active_job_limit").await;
    let repository = Repository::new(&config.db.path).await.unwrap();
    for index in 0..3 {
        seed_job(
            &repository,
            1,
            &format!("active-{index}"),
            JOB_RUNNING,
            100 + index,
        )
        .await;
    }

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
        .header("cookie", &cookie)
        .body(Body::from_stream(form.into_stream()))
        .unwrap();
    assert_eq!(
        app.call(request).await.unwrap().status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn sport_submission_rejects_missing_foreign_and_non_ready_ai_jobs_atomically() {
    let temp = TempDir::new().unwrap();
    let config = isolated_config(&temp, 1);
    let mock = Arc::new(MockLlm::new(Vec::new()));
    let mut app = app::create_app_with_llm(config.clone(), mock).await;
    let owner_cookie = register(&mut app, "submit_owner").await;
    register(&mut app, "submit_other").await;
    let repository = Repository::new(&config.db.path).await.unwrap();
    seed_job(&repository, 1, "owner-failed", JOB_FAILED, 100).await;
    seed_job(&repository, 2, "other-ready", JOB_READY, 200).await;

    let sport = serde_json::to_value(Sport::parse_from_xml(SAMPLE_XML_SWIMMING).unwrap()).unwrap();
    let submit = |job_id: &str| {
        let mut body = sport.clone();
        body["ai_job_id"] = serde_json::json!(job_id);
        Request::builder()
            .uri(routes::API_SPORT_INSERT)
            .method("POST")
            .header("content-type", "application/json")
            .header("cookie", &owner_cookie)
            .body(Body::from(body.to_string()))
            .unwrap()
    };

    assert_eq!(
        app.call(submit("missing-job")).await.unwrap().status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.call(submit("other-ready")).await.unwrap().status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.call(submit("owner-failed")).await.unwrap().status(),
        StatusCode::CONFLICT
    );

    let sports = Request::builder()
        .uri(format!("{}?page=0&size=20", routes::API_SPORT_LIST))
        .method("GET")
        .header("cookie", &owner_cookie)
        .body(Body::empty())
        .unwrap();
    let (status, body) = response_json(app.call(sports).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, serde_json::json!([]));
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
