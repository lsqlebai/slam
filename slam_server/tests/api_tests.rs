// 导入必要的测试依赖
use axum::{
    body::{to_bytes, Body},
    http::{Request, Response, StatusCode},
};
use reqwest::multipart;
use serde_json;
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
async fn test_user_register_and_login() {
    let mut app = app::create_app(AppConfig::default());

    // 随机用户名，避免与现有数据冲突
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let username = format!("test_user_{}", unique);
    let password = "p@ssw0rd";

    // 注册请求
    let register_body = serde_json::json!({
        "name": username,
        "password": password
    });
    let register_req = Request::builder()
        .uri(routes::API_USER_REGISTER)
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(register_body.to_string()))
        .unwrap();

    let register_resp = app.call(register_req).await.unwrap();
    let register_cookie = register_resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let (register_status, register_bytes) = print_response("用户注册", register_resp).await;
    assert_eq!(register_status, StatusCode::OK);

    let reg_json: serde_json::Value = serde_json::from_slice(&register_bytes).unwrap();
    assert_eq!(reg_json.get("success").unwrap().as_bool().unwrap(), true);
    assert!(register_cookie.is_some());
    assert!(register_cookie.unwrap().starts_with("slam="));

    // 登录请求
    let login_body = serde_json::json!({
        "name": username,
        "password": password
    });
    let login_req = Request::builder()
        .uri(routes::API_USER_LOGIN)
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(login_body.to_string()))
        .unwrap();

    let login_resp = app.call(login_req).await.unwrap();
    let login_cookie = login_resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let (login_status, login_bytes) = print_response("用户登录", login_resp).await;
    assert_eq!(login_status, StatusCode::OK);

    let login_json: serde_json::Value = serde_json::from_slice(&login_bytes).unwrap();
    assert_eq!(login_json.get("success").unwrap().as_bool().unwrap(), true);
    assert!(login_cookie.is_some());
    assert!(login_cookie.unwrap().starts_with("slam="));
}

#[tokio::test]
async fn test_user_login_wrong_password() {
    let mut app = app::create_app(AppConfig::default());

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let username = format!("test_user_wrong_{}", unique);
    let password = "correct_password";

    let register_body = serde_json::json!({
        "name": username,
        "password": password
    });
    let register_req = Request::builder()
        .uri(routes::API_USER_REGISTER)
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(register_body.to_string()))
        .unwrap();
    let register_resp = app.call(register_req).await.unwrap();
    let (register_status, _) = print_response("用户注册(用于登录失败场景)", register_resp).await;
    assert_eq!(register_status, StatusCode::OK);

    let login_body = serde_json::json!({
        "name": username,
        "password": "wrong_password"
    });
    let login_req = Request::builder()
        .uri(routes::API_USER_LOGIN)
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(login_body.to_string()))
        .unwrap();
    let login_resp = app.call(login_req).await.unwrap();
    let (login_status, login_bytes) = print_response("用户登录(错误密码)", login_resp).await;
    assert_ne!(login_status, StatusCode::OK);
    let err_json: serde_json::Value = serde_json::from_slice(&login_bytes).unwrap();
    assert_eq!(err_json.get("error").unwrap().as_str().unwrap(), "用户名或密码错误");
}
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

#[tokio::test]
#[ignore]
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

#[tokio::test]
async fn test_image_endpoint_unauthenticated() {
    let mut app = app::create_app(AppConfig::default());

    let form = multipart::Form::new();
    let boundary = form.boundary().to_string();
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

    let response = app.call(request).await.unwrap();
    let (status, body_bytes) = print_response("未登录运动识别", response).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let response_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(response_json.get("error").unwrap().as_str().unwrap(), "未登录或token无效");
}



