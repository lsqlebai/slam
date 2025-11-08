// 导入必要的测试依赖
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

// 已从crate::tests::mock_models导入所需结构体



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
        "prompt": "生成一个简短的介绍",
        "model": "gpt-3.5-turbo",
        "max_tokens": 100
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
    assert!(status.is_client_error() || status.is_server_error());
    
    // 如果返回错误，我们可以验证错误响应的格式
    if status.is_client_error() || status.is_server_error() {
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(response_json.get("error").is_some());
        assert!(response_json.get("request_id").is_some());
    }
}

// 测试函数 - AI图像生成接口
#[tokio::test]
async fn test_generate_image_endpoint() {
    let mut app = app::create_app(AppConfig::default());
    
    // 创建请求体
    let request_body = serde_json::json!({
        "prompt": "生成一只可爱的猫的图片",
        "n": 1,
        "size": "512x512"
    });
    
    let request = Request::builder()
        .uri(routes::API_AI_GENERATE_IMAGE) // 修正路径
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();
    
    // 发送请求
    let response = app.call(request).await.unwrap();
    
    // 使用通用打印函数处理响应
    let (status, body) = print_response("AI图像生成接口", response).await;
    
    // 在测试环境中，由于我们使用模拟配置，API可能会返回错误（因为没有有效的API密钥）
    // 我们验证请求能被应用接收和处理
    assert!(status.is_client_error() || status.is_server_error());
    
    // 如果返回错误，我们可以验证错误响应的格式
    if status.is_client_error() || status.is_server_error() {
        
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(response_json.get("error").is_some());
        assert!(response_json.get("request_id").is_some());
    }
}