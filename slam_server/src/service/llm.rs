use std::env;

use async_trait::async_trait;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;

#[async_trait]
pub trait LLM: Send + Sync {
    async fn chat(&self, request: ChatCompletionRequest) -> Result<String, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub enum LLMError {
    /// 配置错误
    #[allow(unused)]
    ConfigurationError(String),
    /// API请求失败
    APIFailure(String),
    /// 鉴权错误
    LLMAuthenticationError(String),
    /// 内部错误
    InternalError(String),
    /// 参数错误
    ValidationError(String),
    /// 请求超时
    TimeoutError(String),
}

impl std::error::Error for LLMError {}
// 实现Error trait
impl std::fmt::Display for LLMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMError::ConfigurationError(msg) => write!(f, "LLM 配置错误: {}", msg),
            LLMError::APIFailure(msg) => write!(f, "LLM API 请求失败: {}", msg),
            LLMError::LLMAuthenticationError(msg) => write!(f, "LLM 鉴权失败: {}", msg),
            LLMError::InternalError(msg) => write!(f, "LLM 内部错误: {}", msg),
            LLMError::ValidationError(msg) => write!(f, "LLM 参数错误: {}", msg),
            LLMError::TimeoutError(msg) => write!(f, "LLM 请求超时: {}", msg),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TextContent {
    pub r#type: String,
    pub text: String,
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum ContentPart {
    Text(TextContent),
    Image(ImageContent),
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ImageUrl {
    pub url: String,
}

impl std::fmt::Debug for ImageUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.url.as_str();
        let is_data = s.starts_with("data:") && s.contains(";base64,");
        let is_http = s.starts_with("http://") || s.starts_with("https://");
        if is_http {
            f.debug_struct("ImageUrl")
                .field("type", &"url")
                .field("url", &self.url)
                .finish()
        } else if is_data || probable_base64(s) {
            f.debug_struct("ImageUrl")
                .field("type", &"base64")
                .field("len", &self.url.len())
                .finish()
        } else {
            f.debug_struct("ImageUrl")
                .field("type", &"url")
                .field("url", &self.url)
                .finish()
        }
    }
}

fn probable_base64(s: &str) -> bool {
    if s.len() < 64 { return false; }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || c == '-' || c == '_')
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageContent {
    pub r#type: String,
    pub image_url: ImageUrl,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentPart>,
}


pub fn get_api_key_from_env() -> Option<String> {
    if let Ok(api_key) = env::var("AI_API_KEY") {
        if !api_key.trim().is_empty() {
            return Some(api_key);
        }
    }
    // 如果所有环境变量都不存在或为空，则返回None
    None
}

fn extract_text_from_response(v: &Value) -> Option<String> {
    if let Some(s) = v
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")
        .and_then(|c| c.as_str())
    {
        return Some(s.to_string());
    }
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
    if let Some(s) = v.get("content").and_then(|c| c.as_str()) {
        return Some(s.to_string());
    }
    None
}

pub struct Doubao {
    client: Client,
    api_key: String,
    model: String,
    url: String,
}

impl Doubao {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("ark-rust-example/0.1")
            .build()
            .unwrap();
        Self {
            client,
            api_key: get_api_key_from_env().unwrap(),
            model: "doubao-seed-1-6-251015".to_string(),
            url: "https://ark.cn-beijing.volces.com/api/v3/chat/completions".to_string(),
        }
    }
}

#[async_trait]
impl LLM for Doubao {
    async fn chat(&self, request: ChatCompletionRequest) -> Result<String, Box<dyn std::error::Error>> {
        let body = json!({
            "model": self.model.clone(),
            "max_completion_tokens": 65535,
            "messages": request.messages,
            "reasoning_effort": "medium"
        });

        let resp = self
            .client
            .post(self.url.clone())
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .unwrap();

        let status = resp.status();
        let text = resp.text().await.unwrap();
        if !status.is_success() {
            let code = status.as_u16();
            if code == 401 || code == 403 {
                return Err(Box::new(LLMError::LLMAuthenticationError("鉴权失败: API Key 无效或权限不足".to_string())));
            }
            println!("Request failed: {}\n{}", status, text);
            return Err(Box::new(LLMError::APIFailure(format!("Request failed: {}\n{}", status, text))));
        }

        let v: Value = serde_json::from_str(&text).unwrap();

        // 7) Try to extract the assistant's message text (best-effort across common shapes)
        if let Some(extracted) = extract_text_from_response(&v) {
            Ok(extracted)
        } else {
            Err(Box::new(LLMError::InternalError("\n(No recognizable message text field found; see full JSON above.)\n".to_string())))
        }
    }
}
