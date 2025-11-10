//! AI服务模块
//! 提供所有AI能力的统一访问接口

use serde::{Deserialize, Serialize};

// AI服务配置结构
use utoipa::ToSchema;

use crate::service::common;
use crate::service::llm::ChatCompletionRequest;
use crate::service::llm::LLM;
use crate::service::llm;
// AI服务核心结构
pub struct AIService {
    /// 服务配置
    pub llm: LLM,
}

struct ImageParser {}
impl ImageParser {
    fn system_prompt() -> llm::Message {
        llm::Message {
            role: "system".to_string(),
            content: vec![llm::ContentPart::Text(llm::TextContent {
                r#type: "text".to_string(),
                text: "你是一个专业的图片文字识别员,你的任务是根据图片中的文字,生成结构化的JSON格式数据。".to_string(),
            })],
        }
    }
    fn user_prompt() -> llm::Message {
        llm::Message {
            role: "user".to_string(),
            content: vec![
                    llm::ContentPart::Image(llm::ImageContent {
                        r#type: "image_url".to_string(),
                        image_url: llm::ImageUrl {
                            url: "https://ark-project.tos-cn-beijing.ivolces.com/images/view.jpeg".to_string(),
                        },
                    }),
                    llm::ContentPart::Text(llm::TextContent {
                        r#type: "text".to_string(),
                        text: "请读取图中的数据".to_string(),
                    })
                ],
        }
    }
}
// AI服务实现
impl AIService {
    /// 创建新的AI服务实例
    pub fn new() -> Self {
        Self { llm: LLM::doubao() }
    }
    /// 创建ChatCompletionRequest
    fn create_chat_completion_request(request: &TextGenerationRequest) -> ChatCompletionRequest {
        ChatCompletionRequest {
            messages: vec![ImageParser::system_prompt(), ImageParser::user_prompt()],
        }
    }

    /// 生成文本内容
    pub async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<AIResponse<TextGenerationResponse>, common::ServiceError> {
        // 生成请求ID
        let request_id = common::get_current_timestamp();

        // 验证请求参数
        self.validate_text_request(&request)?;

        let chat_request = Self::create_chat_completion_request(&request);
        println!("chat_request: {:?}", chat_request);
        let response = self.llm.chat(chat_request).await.map_err(|e| {
            // A simple mapping for now. A more advanced implementation could inspect `e`
            // to differentiate between network errors, auth errors, etc.
            common::ServiceError {
                code: 500,
                message: e.to_string(),
            }
        })?;

        // 构建响应
        let text_response = TextGenerationResponse {
            generated_text: response,
        };

        // 返回成功响应
        Ok(AIResponse {
            success: true,
            data: Some(text_response),
            error: None,
            request_id: request_id.to_string(),
        })
    }

    /// 验证请求参数
    pub fn validate_text_request(
        &self,
        request: &TextGenerationRequest,
    ) -> Result<(), common::ServiceError> {
        // 验证消息不为空
        if request.messages.is_empty() {
            return Err(common::ServiceError {
                code: 400,
                message: "消息不能为空".to_string(),
            });
        }

        Ok(())
    }
}

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
}

// 文本生成请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TextGenerationRequest {
    pub messages: String,
}

// 文本生成响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TextGenerationResponse {
    pub generated_text: String,
}

// 令牌使用统计
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
