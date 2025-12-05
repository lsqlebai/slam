//! AI服务模块
//! 提供所有AI能力的统一访问接口

use serde::{Deserialize, Serialize};

// AI服务配置结构
use utoipa::ToSchema;

use crate::service::common;
use crate::service::llm;
use crate::service::llm::ChatCompletionRequest;
use crate::service::llm::LLM;
use crate::model::sport::Sport;
use crate::model::sport::{SAMPLE_XML_SWIMMING, SAMPLE_XML_RUNNING};
// AI服务核心结构
pub struct AIService {
    /// 服务配置
    llm: LLM,
}

struct ImageParser {}
impl ImageParser {

    fn system_prompt() -> llm::Message {
        llm::Message {
            role: "system".to_string(),
            content: vec![llm::ContentPart::Text(llm::TextContent {
                r#type: "text".to_string(),
                text: format!(
                    "你是图片文字识别员,根据图片内容输出XML。不同运动类型请输出对应的extra字段。示例(请根据实际类型选择其一):\n游泳示例:\n{}\n跑步示例:\n{}\n要求: 1) 按示例字段命名 2) tracks为分段数组 3) 缺失字段请忽略",
                    SAMPLE_XML_SWIMMING,
                    SAMPLE_XML_RUNNING,
                ),
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
    pub fn new() -> Self { Self { llm: LLM::doubao() } }

    /// 生成文本内容
    pub async fn sports_image_recognition(
        &self,
        base64_data: Vec<String>,
    ) -> Result<AIResponse<Sport>, common::ServiceError> {
        // 生成请求ID
        let request_id = common::get_current_timestamp();
        let chat_request = ImageParser::create_chat_completion_request(base64_data);
        println!("chat_request: {:?}", chat_request);
        let content = self.llm.chat(chat_request).await.map_err(|e| {
            // A simple mapping for now. A more advanced implementation could inspect `e`
            // to differentiate between network errors, auth errors, etc.
            common::ServiceError {
                code: 500,
                message: e.to_string(),
            }
        })?;
        println!("content: {:?}", content);


        let sport = Sport::parse_from_xml(&content)
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

}

impl Default for AIService {
    fn default() -> Self { Self::new() }
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
