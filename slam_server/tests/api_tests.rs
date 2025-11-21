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
use chrono::{Utc, TimeZone, Datelike};

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
        "password": password,
        "nickname": "Nick"
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
        "password": password,
        "nickname": "WrongPWUser"
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

#[tokio::test]
async fn test_sport_insert_list_stats() {
    let mut app = app::create_app(AppConfig::default());

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let username = format!("test_sport_user_{}", unique);
    let password = "p@ssw0rd";

    let register_body = serde_json::json!({
        "name": username,
        "password": password,
        "nickname": "SportsUser"
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
        .map(|s| s.to_string())
        .expect("register set-cookie");
    let cookie_header = register_cookie.split(';').next().unwrap().to_string();

    let dt = Utc.with_ymd_and_hms(2025, 11, 17, 0, 0, 0).unwrap();
    let ts = dt.timestamp();
    let sport_body = serde_json::json!({
        "type": "Swimming",
        "start_time": ts,
        "calories": 123,
        "distance_meter": 1000,
        "duration_second": 600,
        "heart_rate_avg": 120,
        "heart_rate_max": 140,
        "pace_average": "3'59''"
    });
    let insert_req = Request::builder()
        .uri(routes::API_SPORT_INSERT)
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Cookie", cookie_header.clone())
        .body(Body::from(sport_body.to_string()))
        .unwrap();
    let insert_resp = app.call(insert_req).await.unwrap();
    let (insert_status, insert_bytes) = print_response("运动插入", insert_resp).await;
    assert_eq!(insert_status, StatusCode::OK);
    let insert_json: serde_json::Value = serde_json::from_slice(&insert_bytes).unwrap();
    assert_eq!(insert_json.get("success").unwrap().as_bool().unwrap(), true);

    let list_req = Request::builder()
        .uri(format!("{}?page=0&size=20", routes::API_SPORT_LIST))
        .method("GET")
        .header("Cookie", cookie_header.clone())
        .body(Body::empty())
        .unwrap();
    let list_resp = app.call(list_req).await.unwrap();
    let (list_status, list_bytes) = print_response("运动列表", list_resp).await;
    assert_eq!(list_status, StatusCode::OK);
    let list_json: serde_json::Value = serde_json::from_slice(&list_bytes).unwrap();
    assert!(list_json.is_array());
    let arr = list_json.as_array().unwrap();
    assert!(arr.len() >= 1);
    let first = &arr[0];
    assert_eq!(first.get("type").unwrap().as_str().unwrap(), "Swimming");
    assert_eq!(first.get("start_time").unwrap().as_i64().unwrap(), ts);

    let year = dt.year();
    let month = dt.month();
    let week = dt.iso_week().week();

    let stats_year_req = Request::builder()
        .uri(format!("{}?kind=year&year={}", routes::API_SPORT_STATS, year))
        .method("GET")
        .header("Cookie", cookie_header.clone())
        .body(Body::empty())
        .unwrap();
    let stats_year_resp = app.call(stats_year_req).await.unwrap();
    let (stats_year_status, stats_year_bytes) = print_response("年度统计", stats_year_resp).await;
    assert_eq!(stats_year_status, StatusCode::OK);
    let stats_year_json: serde_json::Value = serde_json::from_slice(&stats_year_bytes).unwrap();
    assert_eq!(stats_year_json.get("total_count").unwrap().as_i64().unwrap(), 1);
    assert_eq!(stats_year_json.get("total_calories").unwrap().as_i64().unwrap(), 123);
    assert_eq!(stats_year_json.get("total_duration_second").unwrap().as_i64().unwrap(), 600);
    assert!(stats_year_json.get("sports").unwrap().is_array());
    assert_eq!(stats_year_json.get("sports").unwrap().as_array().unwrap().len(), 1);
    let buckets_year = stats_year_json.get("buckets").unwrap().as_array().unwrap();
    assert_eq!(buckets_year.len(), 1);
    assert_eq!(buckets_year[0].get("date").unwrap().as_i64().unwrap(), month as i64);

    let stats_month_req = Request::builder()
        .uri(format!("{}?kind=month&year={}&month={}", routes::API_SPORT_STATS, year, month))
        .method("GET")
        .header("Cookie", cookie_header.clone())
        .body(Body::empty())
        .unwrap();
    let stats_month_resp = app.call(stats_month_req).await.unwrap();
    let (stats_month_status, stats_month_bytes) = print_response("月度统计", stats_month_resp).await;
    assert_eq!(stats_month_status, StatusCode::OK);
    let stats_month_json: serde_json::Value = serde_json::from_slice(&stats_month_bytes).unwrap();
    let buckets_month = stats_month_json.get("buckets").unwrap().as_array().unwrap();
    assert_eq!(buckets_month.len(), 1);
    assert_eq!(buckets_month[0].get("date").unwrap().as_i64().unwrap(), 17);

    let stats_week_req = Request::builder()
        .uri(format!("{}?kind=week&year={}&week={}", routes::API_SPORT_STATS, year, week))
        .method("GET")
        .header("Cookie", cookie_header.clone())
        .body(Body::empty())
        .unwrap();
    let stats_week_resp = app.call(stats_week_req).await.unwrap();
    let (stats_week_status, stats_week_bytes) = print_response("周度统计", stats_week_resp).await;
    assert_eq!(stats_week_status, StatusCode::OK);
    let stats_week_json: serde_json::Value = serde_json::from_slice(&stats_week_bytes).unwrap();
    let buckets_week = stats_week_json.get("buckets").unwrap().as_array().unwrap();
    assert_eq!(buckets_week.len(), 1);
    assert_eq!(buckets_week[0].get("date").unwrap().as_i64().unwrap(), 1);
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
    let body_str = String::from_utf8(body_bytes.clone()).unwrap();
    assert_eq!(body_str, "未登录或token无效");
}


#[tokio::test]
async fn test_user_info_endpoint() {
    let mut app = app::create_app(AppConfig::default());

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let username = format!("test_user_info_{}", unique);
    let password = "p@ssw0rd";

    let register_body = serde_json::json!({
        "name": username,
        "password": password,
        "nickname": "Tester"
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
        .map(|s| s.to_string())
        .expect("register set-cookie");
    let cookie_header = register_cookie.split(';').next().unwrap().to_string();

    let info_req = Request::builder()
        .uri(routes::API_USER_INFO)
        .method("GET")
        .header("Cookie", cookie_header)
        .body(Body::empty())
        .unwrap();
    let info_resp = app.call(info_req).await.unwrap();
    let (info_status, info_bytes) = print_response("用户信息", info_resp).await;
    assert_eq!(info_status, StatusCode::OK);
    let info_json: serde_json::Value = serde_json::from_slice(&info_bytes).unwrap();
    assert_eq!(info_json.get("nickname").unwrap().as_str().unwrap(), "Tester");
}


#[tokio::test]
async fn test_user_logout() {
    let mut app = app::create_app(AppConfig::default());

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let username = format!("test_logout_{}", unique);
    let password = "p@ssw0rd";

    let register_body = serde_json::json!({
        "name": username,
        "password": password,
        "nickname": "LogoutUser"
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
        .map(|s| s.to_string())
        .expect("register set-cookie");
    let cookie_header = register_cookie.split(';').next().unwrap().to_string();

    let info_req = Request::builder()
        .uri(routes::API_USER_INFO)
        .method("GET")
        .header("Cookie", cookie_header.clone())
        .body(Body::empty())
        .unwrap();
    let info_resp = app.call(info_req).await.unwrap();
    let (info_status, _) = print_response("用户信息(登录态)", info_resp).await;
    assert_eq!(info_status, StatusCode::OK);

    let logout_req = Request::builder()
        .uri(routes::API_USER_LOGOUT)
        .method("POST")
        .header("Cookie", cookie_header.clone())
        .body(Body::empty())
        .unwrap();
    let logout_resp = app.call(logout_req).await.unwrap();
    let logout_cookie = logout_resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .expect("logout set-cookie");
    let cleared_cookie_header = logout_cookie.split(';').next().unwrap().to_string();
    assert!(cleared_cookie_header.starts_with("slam="));

    let info_req2 = Request::builder()
        .uri(routes::API_USER_INFO)
        .method("GET")
        .header("Cookie", cleared_cookie_header)
        .body(Body::empty())
        .unwrap();
    let info_resp2 = app.call(info_req2).await.unwrap();
    let (info_status2, info_bytes2) = print_response("用户信息(注销后)", info_resp2).await;
    assert_eq!(info_status2, StatusCode::UNAUTHORIZED);
    let body_str2 = String::from_utf8(info_bytes2).unwrap();
    assert!(!body_str2.is_empty());
}


#[tokio::test]
async fn test_sport_update() {
    let mut app = app::create_app(AppConfig::default());

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let username = format!("test_sport_update_{}", unique);
    let password = "p@ssw0rd";

    let register_body = serde_json::json!({
        "name": username,
        "password": password,
        "nickname": "Updater"
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
        .map(|s| s.to_string())
        .expect("register set-cookie");
    let cookie_header = register_cookie.split(';').next().unwrap().to_string();

    let dt = chrono::Utc.with_ymd_and_hms(2025, 11, 18, 0, 0, 0).unwrap();
    let ts = dt.timestamp();
    let sport_body = serde_json::json!({
        "type": "Swimming",
        "start_time": ts,
        "calories": 100,
        "distance_meter": 500,
        "duration_second": 300,
        "heart_rate_avg": 110,
        "heart_rate_max": 130,
        "pace_average": "4'10''",
        "extra": {"main_stroke": "freestyle", "stroke_avg": 18, "swolf_avg": 78},
        "tracks": []
    });
    let insert_req = Request::builder()
        .uri(routes::API_SPORT_INSERT)
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Cookie", cookie_header.clone())
        .body(Body::from(sport_body.to_string()))
        .unwrap();
    let insert_resp = app.call(insert_req).await.unwrap();
    let (insert_status, _) = print_response("运动插入(更新前)", insert_resp).await;
    assert_eq!(insert_status, StatusCode::OK);

    let list_req = Request::builder()
        .uri(format!("{}?page=0&size=20", routes::API_SPORT_LIST))
        .method("GET")
        .header("Cookie", cookie_header.clone())
        .body(Body::empty())
        .unwrap();
    let list_resp = app.call(list_req).await.unwrap();
    let (list_status, list_bytes) = print_response("运动列表(更新前)", list_resp).await;
    assert_eq!(list_status, StatusCode::OK);
    let list_json: serde_json::Value = serde_json::from_slice(&list_bytes).unwrap();
    let arr = list_json.as_array().unwrap();
    assert!(arr.len() >= 1);
    let first = &arr[0];
    let sport_id = first.get("id").unwrap().as_i64().unwrap() as i32;

    let update_body = serde_json::json!({
        "id": sport_id,
        "type": "Swimming",
        "start_time": ts,
        "calories": 200,
        "distance_meter": 800,
        "duration_second": 450,
        "heart_rate_avg": 115,
        "heart_rate_max": 140,
        "pace_average": "4'00''",
        "extra": {"main_stroke": "freestyle", "stroke_avg": 20, "swolf_avg": 80},
        "tracks": []
    });
    let update_req = Request::builder()
        .uri(routes::API_SPORT_UPDATE)
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Cookie", cookie_header.clone())
        .body(Body::from(update_body.to_string()))
        .unwrap();
    let update_resp = app.call(update_req).await.unwrap();
    let (update_status, update_bytes) = print_response("运动更新", update_resp).await;
    assert_eq!(update_status, StatusCode::OK);
    let update_json: serde_json::Value = serde_json::from_slice(&update_bytes).unwrap();
    assert_eq!(update_json.get("success").unwrap().as_bool().unwrap(), true);

    let list_req2 = Request::builder()
        .uri(format!("{}?page=0&size=20", routes::API_SPORT_LIST))
        .method("GET")
        .header("Cookie", cookie_header.clone())
        .body(Body::empty())
        .unwrap();
    let list_resp2 = app.call(list_req2).await.unwrap();
    let (list_status2, list_bytes2) = print_response("运动列表(更新后)", list_resp2).await;
    assert_eq!(list_status2, StatusCode::OK);
    let list_json2: serde_json::Value = serde_json::from_slice(&list_bytes2).unwrap();
    let first2 = &list_json2.as_array().unwrap()[0];
    assert_eq!(first2.get("id").unwrap().as_i64().unwrap() as i32, sport_id);
    assert_eq!(first2.get("calories").unwrap().as_i64().unwrap(), 200);
    assert_eq!(first2.get("distance_meter").unwrap().as_i64().unwrap(), 800);
    assert_eq!(first2.get("duration_second").unwrap().as_i64().unwrap(), 450);
}
#[tokio::test]
async fn test_sport_import_csv() {
    let mut app = app::create_app(AppConfig::default());

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let username = format!("test_sport_import_{}", unique);
    let password = "p@ssw0rd";

    let register_body = serde_json::json!({
        "name": username,
        "password": password,
        "nickname": "Importer"
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
        .map(|s| s.to_string())
        .expect("register set-cookie");
    let cookie_header = register_cookie.split(';').next().unwrap().to_string();

    let csv = "Uid,Sid,Key,Time,Category,Value,UpdateTime\n49767842,609301467,pool_swimm,1731888000,swimming,{\"anaerobic_train_e\":1},1731888000\n49767842,609301467,pool_swimm,1731889000,swimming,{\"anaerobic_train_e\":1},1731889000";
    let form = multipart::Form::new()
        .text("vendor", "xiaomi")
        .part(
            "file",
            multipart::Part::text(csv)
                .file_name("sports.csv")
                .mime_str("text/csv")
                .unwrap(),
        );
    let boundary = form.boundary().to_string();
    let stream = form.into_stream();
    let body = Body::from_stream(stream);

    let import_req = Request::builder()
        .uri(routes::API_SPORT_IMPORT)
        .method("POST")
        .header("Content-Type", format!("multipart/form-data; boundary={}", boundary))
        .header("Cookie", cookie_header.clone())
        .body(body)
        .unwrap();
    let import_resp = app.call(import_req).await.unwrap();
    let (import_status, import_bytes) = print_response("运动导入", import_resp).await;
    assert_eq!(import_status, StatusCode::OK);
    let import_json: serde_json::Value = serde_json::from_slice(&import_bytes).unwrap();
    assert_eq!(import_json.get("success").unwrap().as_bool().unwrap(), true);
    assert_eq!(import_json.get("inserted").unwrap().as_u64().unwrap(), 2);

    let list_req = Request::builder()
        .uri(format!("{}?page=0&size=20", routes::API_SPORT_LIST))
        .method("GET")
        .header("Cookie", cookie_header.clone())
        .body(Body::empty())
        .unwrap();
    let list_resp = app.call(list_req).await.unwrap();
    let (list_status, list_bytes) = print_response("运动列表(导入后)", list_resp).await;
    assert_eq!(list_status, StatusCode::OK);
    let list_json: serde_json::Value = serde_json::from_slice(&list_bytes).unwrap();
    assert!(list_json.as_array().unwrap().len() >= 2);
}
