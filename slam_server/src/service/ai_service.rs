//! AI服务模块
//! 提供所有AI能力的统一访问接口


use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

// AI服务配置结构
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AIServiceConfig {
    /// API密钥配置
    pub api_key: Option<String>,
    /// 默认模型名称
    pub default_model: String,
    /// API端点URL
    pub api_endpoint: String,
    /// 请求超时时间（秒）
    pub timeout_seconds: u64,
    /// 最大重试次数
    pub max_retries: u8,
}

// AI服务核心结构
pub struct AIService {
    /// 服务配置
    pub config: AIServiceConfig,
    /// 内部状态
    pub state: Arc<Mutex<ServiceState>>,
    /// OpenAI客户端
    pub client: Client<OpenAIConfig>,
}

// 服务内部状态
#[derive(Debug, Default)]
pub struct ServiceState {
    /// 请求计数器
    pub request_count: u64,
    /// 最后请求时间戳
    pub last_request_time: u64,
}

impl AIServiceConfig {
    /// 返回豆包配置的AIServiceConfig实例
    pub fn doubao() -> AIServiceConfig {
        AIServiceConfig {
            api_key: None,
            default_model: "ERNIE-Bot-4".to_string(),
            api_endpoint: "https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
    pub fn openai() -> Self {
        Self {
            api_key: None,
            default_model: "gpt-3.5-turbo".to_string(),
            api_endpoint: "https://api.openai.com/v1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}











// AI服务实现
impl AIService {
    /// 创建新的AI服务实例
    pub fn new(config: AIServiceConfig) -> Self {
        let openai_config = OpenAIConfig::new()
            .with_api_key(config.api_key.clone().unwrap_or_default())
            .with_api_base(config.api_endpoint.clone());
        let client = Client::with_config(openai_config);

        Self {
            config,
            state: Arc::new(Mutex::new(ServiceState::default())),
            client,
        }
    }

    /// 创建默认配置的AI服务实例
    pub fn with_default_config() -> Self {
        Self::new(AIServiceConfig::openai())
    }

    /// 生成唯一的请求ID
    pub fn generate_request_id(&self) -> String {
        let mut rng = rand::thread_rng();
        let random_part: u64 = rng.gen();
        let timestamp = self.get_current_timestamp();
        format!("req_{}_{:x}", timestamp, random_part)
    }

    /// 获取当前时间戳
    pub fn get_current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    /// 更新服务状态
    pub async fn update_service_state(&self) {
        let mut state = self.state.lock().await;
        state.request_count += 1;
        state.last_request_time = self.get_current_timestamp();
    }

    /// 构建通用请求头
    pub fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // 添加内容类型
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        
        // 如果有API密钥，添加认证头
        if let Some(api_key) = &self.config.api_key {
            let auth_value = format!("Bearer {}", api_key);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                auth_value.parse().unwrap(),
            );
        }
        
        headers
    }

    /// 生成文本内容
    pub async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<AIResponse<TextGenerationResponse>, AIServiceError> {
        // 生成请求ID
        let request_id = self.generate_request_id();

        // 验证请求参数
        self.validate_text_request(&request)?;

        // 更新服务状态
        self.update_service_state().await;

        let timestamp = self.get_current_timestamp();

        // 记录请求信息
        info!(
            "Text generation request: {}, model: {:?}",
            request_id,
            request.model.as_ref().unwrap_or(&self.config.default_model)
        );

        // 准备API请求
        let mut request_builder = CreateChatCompletionRequestArgs::default();
        
        request_builder.model(request.model.unwrap_or_else(|| self.config.default_model.clone()));
        request_builder.messages([
            ChatCompletionRequestUserMessageArgs::default()
                .content(request.prompt)
                .build()
                .map_err(|e| AIServiceError::InternalError(format!("Failed to build message: {}", e)))?
                .into()
        ]);

        if let Some(max_tokens) = request.max_tokens {
            request_builder.max_tokens(max_tokens);
        }
        if let Some(temperature) = request.temperature {
            request_builder.temperature(temperature);
        }
        if let Some(top_p) = request.top_p {
            request_builder.top_p(top_p);
        }

        let api_request = request_builder.build().map_err(|e| AIServiceError::InternalError(format!("Failed to build request: {}", e)))?;

        let response = self.client.chat().create(api_request).await.map_err(|e| {
            // A simple mapping for now. A more advanced implementation could inspect `e`
            // to differentiate between network errors, auth errors, etc.
            AIServiceError::APIFailure(e.to_string())
        })?;


        // 提取生成的文本
        let generated_text = match response.choices.first() {
            Some(choice) => choice.message.content.clone().unwrap_or_default(),
            None => return Err(AIServiceError::InternalError("没有生成文本内容".to_string())),
        };

        // 构建响应
        let text_response = TextGenerationResponse {
            generated_text,
            model_used: response.model,
            token_usage: response.usage.map_or_else(
                || TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                },
                |usage| TokenUsage {
                    prompt_tokens: usage.prompt_tokens,
                    completion_tokens: usage.completion_tokens,
                    total_tokens: usage.total_tokens,
                },
            ),
        };

        // 返回成功响应
        Ok(AIResponse {
            success: true,
            data: Some(text_response),
            error: None,
            request_id,
            timestamp,
        })
    }

    /// 简化的文本生成方法，使用默认参数
    #[allow(unused)]
    pub async fn generate_text_simple(&self, prompt: &str) -> Result<AIResponse<TextGenerationResponse>, AIServiceError> {
        let request = TextGenerationRequest {
            prompt: prompt.to_string(),
            model: None,
            max_tokens: Some(500),
            temperature: Some(0.7),
            top_p: Some(1.0),
        };
        
        self.generate_text(request).await
    }
    
/*
    /// 生成图像内容
    pub async fn generate_image(
        &self,
        prompt: &str,
        n: Option<u32>,
        size: Option<&str>,
        response_format: Option<&str>,
    ) -> Result<AIResponse<ImageGenerationResponse>, AIServiceError> {
        // 生成请求ID
        let request_id = self.generate_request_id();
        
        // 验证请求参数
        self.validate_image_request(prompt)?;
        
        // 更新服务状态
        self.update_service_state().await;
        
        let timestamp = self.get_current_timestamp();
        
        // 记录请求信息
        info!("Image generation request: {}, prompt: {}", request_id, prompt);
        
        // 准备图像生成API请求
        let img_request = ImageGenerationRequestInternal {
            prompt: prompt.to_string(),
            n: n.or(Some(1)),
            size: size.map(String::from).or(Some("1024x1024".to_string())),
            response_format: response_format.map(String::from).or(Some("url".to_string())),
            quality: Some("standard".to_string()),
            style: Some("vivid".to_string()),
        };
        
        // 构建图像生成API的端点 (通常是/openai/v1/images/generations)
        let image_endpoint = if self.config.api_endpoint.contains("openai") {
            "https://api.openai.com/v1/images/generations".to_string()
        } else {
            // 如果是自定义端点，假设它已经正确配置
            self.config.api_endpoint.clone()
        };
        
        // 发送API请求
        let headers = self.build_headers();
        let response = match self
            .http_client
            .post(&image_endpoint)
            .headers(headers)
            .json(&img_request)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                return Err(AIServiceError::APIFailure(format!("HTTP请求失败: {}", e)));
            }
        };
        
        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let text = match response.text().await {
                Ok(t) => t,
                Err(_) => "无法获取错误详情".to_string(),
            };
            
            if status == 401 || status == 403 {
                return Err(AIServiceError::AuthenticationError(text));
            } else {
                return Err(AIServiceError::APIFailure(format!("API返回错误状态: {} - {}", status, text)));
            }
        }
        
        // 解析响应
        let api_response: ImageGenerationResponseInternal = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                return Err(AIServiceError::InternalError(format!("解析响应失败: {}", e)));
            }
        };
        
        // 提取生成的图像URLs或base64数据
        let mut image_data_results = Vec::new();
        for image_data in api_response.data {
            if let Some(url) = image_data.url {
                image_data_results.push(ImageData { url, base64_data: None });
            } else if let Some(b64) = image_data.b64_json {
                image_data_results.push(ImageData {
                    url: format!("data:image/png;base64,{}", b64),
                    base64_data: Some(b64),
                });
            }
        }
        
        // 如果没有生成任何图像
        if image_data_results.is_empty() {
            return Err(AIServiceError::InternalError("没有生成图像内容".to_string()));
        }
        
        let image_response = ImageGenerationResponse {
            images: image_data_results,
            model_used: "dall-e-3".to_string(), // 模型名称需要根据实际情况调整
        };

        // 返回成功响应
        Ok(AIResponse {
            success: true,
            data: Some(image_response),
            error: None,
            request_id,
            timestamp,
        })
    }
    */
    
/*
    /// 简化的图像生成方法，使用默认参数
    #[allow(unused)]
    pub async fn generate_image_simple(&self, prompt: &str) -> Result<AIResponse<ImageGenerationResponse>, AIServiceError> {
        self.generate_image(prompt, Some(1), Some("1024x1024"), Some("url")).await
    }
    */
    
    /// 将错误转换为标准错误响应
    pub fn error_to_response<T>(&self, error: AIServiceError, request_id: &str) -> AIResponse<T> {
        // 根据错误类型构建错误响应
        let (code, message, details) = match error {
            AIServiceError::ConfigurationError(msg) => (
                "CONFIGURATION_ERROR".to_string(),
                "配置错误".to_string(),
                Some(msg),
            ),
            AIServiceError::APIFailure(msg) => (
                "API_FAILURE".to_string(),
                "AI服务调用失败".to_string(),
                Some(msg),
            ),
            AIServiceError::AuthenticationError(msg) => (
                "AUTHENTICATION_ERROR".to_string(),
                "认证失败".to_string(),
                Some(msg),
            ),
            AIServiceError::InternalError(msg) => (
                "INTERNAL_ERROR".to_string(),
                "服务器内部错误".to_string(),
                Some(msg),
            ),
            AIServiceError::ValidationError(msg) => (
                "VALIDATION_ERROR".to_string(),
                "请求参数无效".to_string(),
                Some(msg),
            ),
            AIServiceError::TimeoutError(msg) => (
                "TIMEOUT_ERROR".to_string(),
                "请求超时".to_string(),
                Some(msg),
            ),
        };
        
        // 构建错误响应对象
        let error_response = ErrorResponse {
            code,
            message,
            details,
        };
        
        // 构建完整的API响应
        AIResponse {
            success: false,
            data: None,
            error: Some(error_response),
            request_id: request_id.to_string(),
            timestamp: self.get_current_timestamp(),
        }
    }
    
    /// 验证请求参数
    pub fn validate_text_request(&self, request: &TextGenerationRequest) -> Result<(), AIServiceError> {
        // 验证prompt不为空
        if request.prompt.trim().is_empty() {
            return Err(AIServiceError::ValidationError("提示文本不能为空".to_string()));
        }
        
        // 验证最大令牌数
        if let Some(max_tokens) = request.max_tokens {
            if max_tokens > 8192 {
                return Err(AIServiceError::ValidationError("最大令牌数不能超过8192".to_string()));
            }
            if max_tokens == 0 {
                return Err(AIServiceError::ValidationError("最大令牌数必须大于0".to_string()));
            }
        }
        
        // 验证温度参数
        if let Some(temp) = request.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(AIServiceError::ValidationError("温度参数必须在0.0到2.0之间".to_string()));
            }
        }
        
        // 验证top_p参数
        if let Some(top_p) = request.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                return Err(AIServiceError::ValidationError("top_p参数必须在0.0到1.0之间".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// 验证图像生成请求
    pub fn validate_image_request(&self, prompt: &str) -> Result<(), AIServiceError> {
        // 验证prompt不为空
        if prompt.trim().is_empty() {
            return Err(AIServiceError::ValidationError("图像提示不能为空".to_string()));
        }
        
        // 限制提示长度
        if prompt.len() > 1000 {
            return Err(AIServiceError::ValidationError("图像提示长度不能超过1000个字符".to_string()));
        }
        
        Ok(())
    }
}

// AI服务错误类型
#[derive(Debug)]
pub enum AIServiceError {
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
    #[allow(unused)]
    TimeoutError(String),
}

// 实现Display trait用于错误显示
impl std::fmt::Display for AIServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AIServiceError::ConfigurationError(msg) => write!(f, "配置错误: {}", msg),
            AIServiceError::APIFailure(msg) => write!(f, "API请求失败: {}", msg),
            AIServiceError::AuthenticationError(msg) => write!(f, "认证错误: {}", msg),
            AIServiceError::InternalError(msg) => write!(f, "内部错误: {}", msg),
            AIServiceError::ValidationError(msg) => write!(f, "参数验证错误: {}", msg),
            AIServiceError::TimeoutError(msg) => write!(f, "超时错误: {}", msg),
        }
    }
}

// 实现Error trait
impl std::error::Error for AIServiceError {}

// 错误响应结构
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

// AI响应通用结构
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AIResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ErrorResponse>,
    pub request_id: String,
    pub timestamp: u64,
}

// 文本生成请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TextGenerationRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
}

// 文本生成响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TextGenerationResponse {
    pub generated_text: String,
    pub model_used: String,
    pub token_usage: TokenUsage,
}

// 令牌使用统计
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}