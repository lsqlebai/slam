// 导入必要的测试依赖
use std::env;
use reqwest::header;
use serde_json::json;
use serde_json::Value;
use axum::{body::{Body, to_bytes}, http::{Request, Response, StatusCode}};
use serde_json;
use tower::Service;

// 导入项目模块
use slam_server::app;
use slam_server::app::AppConfig;
use slam_server::app::routes;

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

// 测试函数 - AI文本生成接口
#[tokio::test]
async fn test_generate_text_endpoint() {
    let mut app = app::create_app(AppConfig::default());
    
    // 创建请求体
    let request_body = serde_json::json!({
        "messages": "生成一个简短的介绍",
    });
    
    let request = Request::builder()
        .uri(routes::API_AI_GENERATE_TEXT) // 修正路径
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();
    
    // 发送请求
    let response = app.call(request).await.unwrap();
    
    // 使用通用打印函数处理响应
    let (status, body) = print_response("AI文本生成接口", response).await;
    
    // 在测试环境中，由于我们使用模拟配置，API可能会返回错误（因为没有有效的API密钥）
    // 我们验证请求能被应用接收和处理
    assert_eq!(status, StatusCode::OK);
    
    // 如果返回错误，我们可以验证错误响应的格式
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(response_json.get("error").is_some());
    
}

#[tokio::test]
async fn test_get_api_key_from_env() {
    println!("执行get_api_key_from_env函数...");
    // 从环境变量读取 AI API 密钥
    let api_key = env::var("AI_API_KEY").ok().filter(|k| !k.trim().is_empty());
    match api_key {
        Some(key) => println!("成功获取到API Key: {}", key),
        None => println!("未获取到API Key，环境变量中可能没有设置")
    }
    // 测试总是通过，因为我们只想看到输出
    assert!(true);
}

#[tokio::test]
#[ignore] // This test calls an external API and requires a valid API key.
async fn test_ai_request() {
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
    
    let resp = client.post(url)
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
    println!("\n=== Full JSON response ===\n{}\n", serde_json::to_string_pretty(&v).unwrap());

    // 7) Try to extract the assistant's message text (best-effort across common shapes)
    if let Some(extracted) = extract_text_from_response(&v) {
        println!("\n=== Extracted content ===\n{}\n", extracted);
    } else {
        println!("\n(No recognizable message text field found; see full JSON above.)\n");
    }

}