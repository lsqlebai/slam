use rand::Rng;
use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServiceError {
    pub code: u32,
    pub message: String,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Context {
    pub uid: String,
}

// 实现Error trait
impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServiceError({}, {})", self.code, self.message)
    }
}

/// 获取当前时间戳
pub fn get_current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// 生成唯一的请求ID
pub fn generate_request_id() -> String {
    let mut rng = rand::thread_rng();
    let random_part: u64 = rng.gen();
    let timestamp = get_current_timestamp();
    format!("req_{}_{:x}", timestamp, random_part)
}

