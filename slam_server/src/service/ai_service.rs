//! AI服务模块
//! 提供所有AI能力的统一访问接口

use serde::{Deserialize, Serialize};

// AI服务配置结构
use utoipa::ToSchema;

use crate::model;
use crate::service::common;
use crate::service::llm;
use crate::service::llm::ChatCompletionRequest;
use crate::service::llm::LLM;

use crate::model::sport::{Sport};
use crate::service::image_service::ImageService;

// AI服务核心结构
pub struct AIService {
    /// 服务配置
    llm: LLM,
    image_service: ImageService,
}

struct ImageParser {}
impl ImageParser {

    fn system_prompt() -> llm::Message {
        llm::Message {
            role: "system".to_string(),
            content: vec![llm::ContentPart::Text(llm::TextContent {
                r#type: "text".to_string(),
                text: format!("你是一个专业的图片文字识别员,你的任务是根据图片中的文字,生成以下的XML格式数据: {}", model::sport::SAMPLE_XML).to_string(),
            })],
        }
    }
    fn user_prompt(base64_data: Vec<String>) -> llm::Message {
        let mut contents = base64_data
            .into_iter()
            .map(|base64| {
                llm::ContentPart::Image(llm::ImageContent {
                    r#type: "image_url".to_string(),
                    image_url: llm::ImageUrl { url: base64 },
                })
            })
            .collect::<Vec<_>>();
        contents.append(&mut vec![llm::ContentPart::Text(llm::TextContent {
            r#type: "text".to_string(),
            text: "请读取图中的数据".to_string(),
        })]);
        llm::Message {
            role: "user".to_string(),
            content: contents,
        }
    }
    /// 创建ChatCompletionRequest
    fn create_chat_completion_request(base64_data: Vec<String>) -> ChatCompletionRequest {
        ChatCompletionRequest {
            messages: vec![
                ImageParser::system_prompt(),
                ImageParser::user_prompt(base64_data),
            ],
        }
    }

}

// AI服务实现
impl AIService {
    /// 创建新的AI服务实例
    pub fn new() -> Self {
        Self {
            llm: LLM::doubao(),
            image_service: ImageService::new(),
        }
    }

    /// 生成文本内容
    pub async fn generate_text(
        &self,
        image_data: Vec<u8>,
    ) -> Result<AIResponse<Sport>, common::ServiceError> {
        // 生成请求ID
        let request_id = common::get_current_timestamp();

        let image_process_result = self.image_service.process_image(image_data.into());
        let base64_data = image_process_result
            .map(|res| res.base64_data)
            .map_err(|e| common::ServiceError {
                code: 500,
                message: e.to_string(),
            })?;
        let chat_request = ImageParser::create_chat_completion_request(base64_data);
        println!("chat_request: {:?}", chat_request);
        let response = self.llm.chat(chat_request).await.map_err(|e| {
            // A simple mapping for now. A more advanced implementation could inspect `e`
            // to differentiate between network errors, auth errors, etc.
            common::ServiceError {
                code: 500,
                message: e.to_string(),
            }
        })?;

        let sport = Sport::parse_from_xml(&response)
            .map_err(|e| common::ServiceError {
                code: 500,
                message: e.to_string(),
            })?;
            
        // 返回成功响应
        Ok(AIResponse {
            success: true,
            data: Some(sport),
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

// 令牌使用统计
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
