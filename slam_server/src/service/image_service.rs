//! 图片处理服务模块
//! 提供图片压缩和base64转换功能

use crate::service::common::ServiceError;
use image::{DynamicImage, GenericImageView, ImageOutputFormat};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use base64::{engine::general_purpose::STANDARD as BASE64_ENGINE, Engine};

// 图片处理服务结构
pub struct ImageService;

// 图片处理服务实现
impl ImageService {
    /// 创建新的图片处理服务实例
    pub fn new() -> Self {
        Self
    }
    
    /// 图片切割功能：递归切割图片，确保最终所有图片的宽高都不超过指定阈值
    pub fn split_image(&self, img: DynamicImage, threshold: u32) -> Vec<DynamicImage> {
        let mut result = Vec::new();
        Self::split_image_recursive(img, threshold, &mut result);
        result
    }
    
    /// 递归辅助函数，用于切割单个图片直到满足尺寸要求
    fn split_image_recursive(mut img: DynamicImage, threshold: u32, result: &mut Vec<DynamicImage>) {
        let (width, height) = img.dimensions();
        
        // 终止条件：如果宽和高都小于等于阈值，直接添加到结果中
        if width <= threshold && height <= threshold {
            result.push(img);
            return;
        }
        
        // 根据尺寸决定切割方向
        if width > height && width > threshold {
            // 水平切割（宽大于高且宽大于阈值）
            let mid_width = width / 2;
            let left_part = img.crop(0, 0, mid_width, height);
            let right_part = img.crop(mid_width, 0, width - mid_width, height);
            
            // 递归处理切割后的两部分
            Self::split_image_recursive(left_part, threshold, result);
            Self::split_image_recursive(right_part, threshold, result);
        } else if height > threshold {
            // 垂直切割（高大于阈值）
            let mid_height = height / 2;
            let top_part = img.crop(0, 0, width, mid_height);
            let bottom_part = img.crop(0, mid_height, width, height - mid_height);
            
            // 递归处理切割后的两部分
            Self::split_image_recursive(top_part, threshold, result);
            Self::split_image_recursive(bottom_part, threshold, result);
        } else {
            // 其他情况，直接添加到结果中
            result.push(img);
        }
    }
    
    /// 处理图片：压缩并转换为base64
    pub fn process_image(&self, image_data: Vec<u8>) -> Result<ImageProcessResponse, ServiceError> {
        // 解码图片
        let img = image::load_from_memory(&image_data)
            .map_err(|e| ServiceError {
                code: 400,
                message: format!("无法解码图片: {}", e),
            })?;
        
        // 压缩图片
        let compressed_img = self.compress_image(img, 256);

        let split_imgs = self.split_image(compressed_img, 1024);
        #[cfg(debug_assertions)]
        for (i, img) in split_imgs.iter().enumerate() {
            img.save_with_format(format!("./compressed_{}.jpeg", i), image::ImageFormat::Jpeg).map_err(|e| ServiceError {
                code: 500,
                message: format!("无法保存压缩图片: {}", e),
            })?;
        }

        let base64_data: Vec<String> = split_imgs.iter().map(|img| self.image_to_base64(img.clone())).collect::<Result<Vec<_>, _>>()?;
        // 转换为base64
        if !base64_data.is_empty() {            
            // 返回响应
            Ok(ImageProcessResponse {
                base64_data
            })
        } else {
            Err(ServiceError {
                code: 500,
                message: "无法切割图片".to_string(),
            })
        }
        

    }
    
    /// 等比例压缩图片到长和宽其中之一小于指定阈值
    fn compress_image(&self, img: DynamicImage, threshold: u32) -> DynamicImage {
        let (width, height) = img.dimensions();
        
        // 如果已经有一个维度小于阈值，不需要压缩
        if width <= threshold || height <= threshold {
            return img;
        }
        
        // 计算压缩比例
        let scale_factor = (threshold as f64 / width.min(height) as f64).min(1.0);
        let new_width = (width as f64 * scale_factor) as u32;
        let new_height = (height as f64 * scale_factor) as u32;
        
        // 调整图片大小
        img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
    }
    
    /// 将图片转换为base64编码
    fn image_to_base64(&self, img: DynamicImage) -> Result<String, ServiceError> {
        let mut buffer = Cursor::new(Vec::new());
        
        // 将图片保存到buffer中（使用PNG格式保证质量）
        img.write_to(&mut buffer, ImageOutputFormat::Jpeg(90))
            .map_err(|e| ServiceError {
                code: 500,
                message: format!("无法转换图片格式: {}", e),
            })?;
        
        // 获取buffer内容并转换为base64
        let encoded = BASE64_ENGINE.encode(buffer.into_inner());
        Ok(format!("data:image/jpeg;base64,{}", encoded))
    }
}

impl Default for ImageService { fn default() -> Self { Self::new() } }

// 图片处理请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageProcessRequest {
    pub image: Vec<u8>, // 原始图片数据
}

/// 图片处理响应
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ImageProcessResponse {
    #[schema(example = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==")]
    pub base64_data: Vec<String>, // base64编码的图片数据
}
