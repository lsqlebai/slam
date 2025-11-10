use std::env;

use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;

pub struct LLM {
    client: Client,
    api_key: String,
    model: String,
    url: String,
}

#[derive(Debug)]
pub enum LLMError {
    /// 配置错误
    #[allow(unused)]
    ConfigurationError(String),
    /// API请求失败
    APIFailure(String),
    /// 认证错误
    AuthenticationError(String),
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
            LLMError::AuthenticationError(msg) => write!(f, "LLM 认证错误: {}", msg),
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageUrl {
    pub url: String,
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

impl LLM {
    pub fn doubao() -> Self {
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

    pub async fn chat(&self, request: ChatCompletionRequest) -> Result<String, Box<dyn std::error::Error>> {
        let body = json!({
            "model": self.model.clone(),
            "max_completion_tokens": 65535,
            "messages": request.messages,
            "reasoning_effort": "medium"
        });
        let pretty_str = serde_json::to_string_pretty(&body).unwrap();
        println!("\n美观格式:\n{}", pretty_str);

        let resp = self
            .client
            .post(self.url.clone())
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .unwrap();

        // 5) Handle non-2xx status codes explicitly
        let status = resp.status();
        let text = resp.text().await.unwrap();
        if !status.is_success() {
            return Err(Box::new(LLMError::APIFailure(format!("Request failed: {}\n{}", status, text))));
        }

        // 6) Pretty-print the full JSON response
        let v: Value = serde_json::from_str(&text).unwrap();
        println!(
            "\n=== Full JSON response ===\n{}\n",
            serde_json::to_string_pretty(&v).unwrap()
        );

        // 7) Try to extract the assistant's message text (best-effort across common shapes)
        if let Some(extracted) = Self::extract_text_from_response(&v) {
            return Ok(extracted);
        } else {
            return Err(Box::new(LLMError::InternalError("\n(No recognizable message text field found; see full JSON above.)\n".to_string())));
        }
    }
}
