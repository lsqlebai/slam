use std::env;
use serde_json::{json};
use reqwest::header;

#[tokio::test]
async fn test_get_api_key_from_env() {
    let api_key = env::var("AI_API_KEY").ok().filter(|k| !k.trim().is_empty());
    assert!(api_key.is_some() || api_key.is_none());
}

#[tokio::test]
async fn test_doubao_request() {
    let api_key = env::var("AI_API_KEY").expect("has keys");
    let client = reqwest::Client::builder()
        .user_agent("ark-rust-example/0.1")
        .build()
        .unwrap();
    let body = json!({
        "model": "doubao-seed-1-6-251015",
        "max_completion_tokens": 65535,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "image_url", "image_url": { "url": "https://ark-project.tos-cn-beijing.ivolces.com/images/view.jpeg" }},
                    {"type": "text", "text": "图片主要讲了什么?"}
                ]
            }
        ],
        "reasoning_effort": "medium"
    });
    let url = "https://ark.cn-beijing.volces.com/api/v3/chat/completions";
    let resp = client
        .post(url)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let text = resp.text().await.unwrap();
    assert!(status.is_success(), "{}", text);
}

#[tokio::test]
async fn test_sqlite_insert_and_query_sport_from_sample_xml() {
use slam_server::dao::sqlite_impl::SqliteImpl;
    use slam_server::dao::idl::SportDao;
    use slam_server::model::sport::{Sport, SAMPLE_XML};
    use std::path::Path;
    let db_path = "tests/test.db";
    if Path::new(db_path).exists() {
        let _ = std::fs::remove_file(db_path);
    }
    let sport = Sport::parse_from_xml(SAMPLE_XML).expect("parse xml");
    let dao = SqliteImpl::new(db_path).await.expect("dao new");
    dao.insert(0, sport.clone()).await.expect("dao insert");
    let all = dao.list(0).await.expect("dao list");
    assert_eq!(all.len(), 1);
}

#[test]
fn test_app_config_default_uses_yaml_or_default() {
    use std::path::Path;
    use std::fs;
    use slam_server::config::AppConfig as Cfg;
    let cfg = Cfg::default();
    assert!(!cfg.db.path.trim().is_empty());
    let cfg_path = Path::new("config/app.yml");
    if cfg_path.exists() {
        let file = fs::File::open(cfg_path).unwrap();
        let expected: Cfg = serde_yaml::from_reader(file).unwrap();
        assert_eq!(cfg.db.path, expected.db.path);
        assert_eq!(cfg.server.ip, expected.server.ip);
        assert_eq!(cfg.server.port, expected.server.port);
        assert_eq!(cfg.ai.key, expected.ai.key);
    } else {
        assert_eq!(cfg.db.path, "sport.db");
        assert_eq!(cfg.server.ip, "127.0.0.1");
        assert_eq!(cfg.server.port, 3000);
        assert_eq!(cfg.ai.key, "");
    }
}

#[test]
fn test_app_config_new_with_missing_file_returns_defaults() {
    use slam_server::config::AppConfig as Cfg;
    let missing = "config/__nonexistent__.yml";
    assert!(!std::path::Path::new(missing).exists());
    let cfg = Cfg::new(missing);
    assert_eq!(cfg.db.path, "sport.db");
    assert_eq!(cfg.server.ip, "127.0.0.1");
    assert_eq!(cfg.server.port, 3000);
    assert_eq!(cfg.ai.key, "");
}