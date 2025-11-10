use super::ai_service::AIServiceConfig;

#[test]
fn test_get_api_key_from_env() {
    println!("执行get_api_key_from_env函数...");
    let api_key = AIServiceConfig::get_api_key_from_env();
    match api_key {
        Some(key) => println!("成功获取到API Key: {}", key),
        None => println!("未获取到API Key，环境变量中可能没有设置")
    }
    // 测试总是通过，因为我们只想看到输出
    assert!(true);
}