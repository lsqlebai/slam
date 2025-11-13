// 导入必要的测试依赖
use axum::{
    body::{to_bytes, Body},
    http::{Request, Response, StatusCode},
};
use reqwest::header;
use reqwest::multipart;
use serde_json;
use serde_json::json;
use serde_json::Value;
use std::env;
use tower::Service;

// 导入项目模块
use slam_server::app;
use slam_server::app::routes;
use slam_server::app::AppConfig;

/// 通用的响应打印函数
/// 打印指定接口的响应状态和响应体
async fn print_response(endpoint_name: &str, response: Response<Body>) -> (StatusCode, Vec<u8>) {
    // 打印响应状态
    println!("{}响应状态: {:?}", endpoint_name, response.status());

    // 保存状态码，因为into_body会消费响应
    let status = response.status();

    // 读取响应体
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);

    // 打印响应体
    println!("{}响应体: {}", endpoint_name, body_str);

    // 返回状态码和响应体，供后续使用
    (status, body.to_vec())
}

// 测试函数 - 获取状态接口
#[tokio::test]
async fn test_status_endpoint() {
    let mut app = app::create_app(AppConfig::default());

    // 创建请求 - 注意路径修正为 /api/status
    let request = Request::builder()
        .uri(routes::API_STATUS)
        .method("GET")
        .body(Body::empty())
        .unwrap();

    // 发送请求
    let response = app.call(request).await.unwrap();

    // 使用通用打印函数处理响应
    let (status, body) = print_response("状态接口", response).await;

    // 验证响应状态码
    assert_eq!(status, StatusCode::OK);

    // 解析JSON响应
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(response_json.get("status").is_some());
    assert!(response_json.get("message").is_some());
    assert!(response_json.get("timestamp").is_some());
}

#[tokio::test]
async fn get_api_key_from_env_example() {
    println!("执行get_api_key_from_env函数...");
    // 从环境变量读取 AI API 密钥
    let api_key = env::var("AI_API_KEY").ok().filter(|k| !k.trim().is_empty());
    match api_key {
        Some(key) => println!("成功获取到API Key: {}", key),
        None => println!("未获取到API Key，环境变量中可能没有设置"),
    }
    // 测试总是通过，因为我们只想看到输出
    assert!(true);
}

#[allow(dead_code)]
#[allow(dead_code)]
async fn ai_request_example() {
    /// Heuristic extractor for the assistant message text across several common response shapes.
    fn extract_text_from_response(v: &Value) -> Option<String> {
        // OpenAI-like: choices[0].message.content (string)
        if let Some(s) = v
            .get("choices")?
            .get(0)?
            .get("message")?
            .get("content")
            .and_then(|c| c.as_str())
        {
            return Some(s.to_string());
        }

        // Content as array of blocks (e.g., [{type: "output_text", text: "..."}, ...])
        if let Some(arr) = v
            .get("choices")?
            .get(0)?
            .get("message")?
            .get("content")
            .and_then(|c| c.as_array())
        {
            let mut pieces = Vec::new();
            for item in arr {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    pieces.push(text);
                }
            }
            if !pieces.is_empty() {
                return Some(pieces.join("\n"));
            }
        }

        // Fallback: top-level `content` string (rare)
        if let Some(s) = v.get("content").and_then(|c| c.as_str()) {
            return Some(s.to_string());
        }

        None
    }

    // 1) Read API key from environment

    let api_key = env::var("AI_API_KEY").expect("has keys");

    // 2) Prepare HTTP client
    let client = reqwest::Client::builder()
        .user_agent("ark-rust-example/0.1")
        .build()
        .unwrap();

    // 3) Build the request JSON payload (mirrors your curl body)
    let body = json!({
        "model": "doubao-seed-1-6-251015",
        "max_completion_tokens": 65535,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": { "url": "https://ark-project.tos-cn-beijing.ivolces.com/images/view.jpeg" }
                    },
                    {
                        "type": "text",
                        "text": "图片主要讲了什么?"
                    }
                ]
            }
        ],
        "reasoning_effort": "medium"
    });

    // 4) Send the POST request
    let url = "https://ark.cn-beijing.volces.com/api/v3/chat/completions";

    let resp = client
        .post(url)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .unwrap();

    // 5) Handle non-2xx status codes explicitly
    let status = resp.status();
    let text = resp.text().await.unwrap();
    if !status.is_success() {
        panic!("Request failed: {}\n{}", status, text);
    }

    // 6) Pretty-print the full JSON response
    let v: Value = serde_json::from_str(&text).unwrap();
    println!(
        "\n=== Full JSON response ===\n{}\n",
        serde_json::to_string_pretty(&v).unwrap()
    );

    // 7) Try to extract the assistant's message text (best-effort across common shapes)
    if let Some(extracted) = extract_text_from_response(&v) {
        println!("\n=== Extracted content ===\n{}\n", extracted);
    } else {
        println!("\n(No recognizable message text field found; see full JSON above.)\n");
    }
}

#[tokio::test]
async fn test_image_endpoint() {
    let mut app = app::create_app(AppConfig::default());

    // Read image data from local test.jpg file
    let image_data = std::fs::read("tests/test_img/test1.jpg").expect("Failed to read test.jpg file");
// Read image data from local test.jpg file
    let image_data2 = std::fs::read("tests/test_img/test2.jpg").expect("Failed to read test2.jpg file");
    let form = multipart::Form::new()
        .part(
            "image",
            multipart::Part::bytes(image_data)
                .file_name("sport.jpg")
                .mime_str("image/jpeg")
                .unwrap(),
        )
        .part(
            "image",
            multipart::Part::bytes(image_data2)
                .file_name("track.jpg")
                .mime_str("image/jpeg")
                .unwrap(),
        );

    let boundary = form.boundary().to_string(); // <-- 公有 API
    let stream = form.into_stream();
    let body = Body::from_stream(stream);

    let request = Request::builder()
        .uri(routes::API_IMAGE_PARSE)
        .method("POST")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(body)
        .unwrap();

    // Send the request
    let response = app.call(request).await.unwrap();

    // Assert the response
    let (status, body) = print_response("Image Compression Endpoint", response).await;
    assert_eq!(status, StatusCode::OK);

    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(response_json.get("success").is_some());
}

#[allow(dead_code)]
async fn sqlite_insert_and_query_sport_from_sample_xml_example() {
    use slam_server::dao::sqlite_sport_dao::SqliteSportDao;
    use slam_server::dao::sport_dao::SportDao;
    use slam_server::model::sport::{Sport, SAMPLE_XML};
    use std::path::Path;
    let db_path = "tests/test.db";
    if Path::new(db_path).exists() {
        let _ = std::fs::remove_file(db_path);
    }

    let sport = Sport::parse_from_xml(SAMPLE_XML).expect("parse xml");
    let dao = SqliteSportDao::new(db_path).await.expect("dao new");
    dao.insert(sport.clone()).await.expect("dao insert");

    let all = dao.list().await.expect("dao list");
    assert_eq!(all.len(), 1);
    let got = &all[0];
    assert!(got.id > 0);
    assert_eq!(got.r#type, sport.r#type);
    assert_eq!(got.start_time, sport.start_time);
    assert_eq!(got.calories, sport.calories);
    assert_eq!(got.distance_meter, sport.distance_meter);
    assert_eq!(got.duration_second, sport.duration_second);
    assert_eq!(got.heart_rate_avg, sport.heart_rate_avg);
    assert_eq!(got.heart_rate_max, sport.heart_rate_max);
    assert_eq!(got.pace_average, sport.pace_average);
    assert_eq!(got.extra.main_stroke, sport.extra.main_stroke);
    assert_eq!(got.extra.stroke_avg, sport.extra.stroke_avg);
    assert_eq!(got.extra.swolf_avg, sport.extra.swolf_avg);
    assert_eq!(got.tracks.len(), sport.tracks.len());
    assert_eq!(got.tracks[0].distance_meter, sport.tracks[0].distance_meter);
    assert_eq!(got.tracks[0].duration_second, sport.tracks[0].duration_second);
    assert_eq!(got.tracks[0].pace_average, sport.tracks[0].pace_average);
    assert_eq!(got.tracks[0].extra.main_stroke, sport.tracks[0].extra.main_stroke);

    let range = dao
        .list_by_time_range(sport.start_time - 1, sport.start_time + 1)
        .await
        .expect("dao list_by_time_range");
    assert_eq!(range.len(), 1);
}

#[allow(dead_code)]
fn app_config_default_uses_yaml_or_default_example() {
    use std::path::Path;
    use std::fs;
    use slam_server::config::AppConfig as Cfg;
    let cfg = Cfg::default();
    assert!(!cfg.sqlite_db_path.trim().is_empty());
    let cfg_path = Path::new("config/app.yml");
    if cfg_path.exists() {
        let file = fs::File::open(cfg_path).unwrap();
        let expected: Cfg = serde_yaml::from_reader(file).unwrap();
        assert_eq!(cfg.sqlite_db_path, expected.sqlite_db_path);
    } else {
        assert_eq!(cfg.sqlite_db_path, "sport.db");
    }
}

#[allow(dead_code)]
fn app_config_new_with_missing_file_returns_defaults_example() {
    use slam_server::config::AppConfig as Cfg;
    let missing = "config/__nonexistent__.yml";
    assert!(!std::path::Path::new(missing).exists());
    let cfg = Cfg::new(missing);
    assert_eq!(cfg.sqlite_db_path, "sport.db");
}
