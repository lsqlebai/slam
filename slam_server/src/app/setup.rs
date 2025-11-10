use axum::{Router, routing::{get, post}};
use tower_http::cors::{Any, CorsLayer};
use std::sync::Arc;
use std::net::SocketAddr;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// 导入AI服务相关模块
use crate::service::ai_service::AIService;

/// 应用配置选项
pub struct AppConfig;

impl Default for AppConfig {
    fn default() -> Self {
        Self
    }
}

pub async fn run() {
    let config = AppConfig::default();
    let app = create_app(config);

    // 设置监听地址
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("服务器正在监听 http://{}...", addr);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("服务器正在监听 http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .unwrap();
}


/// 创建应用实例的通用函数
pub fn create_app(config: AppConfig) -> Router {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::handlers::get_status,
            crate::handlers::generate_text_handler,
            crate::handlers::root
        ),
        components(
            schemas(
                crate::handlers::ApiResponse,
                crate::service::ai_service::TextGenerationRequest,
                crate::service::ai_service::TokenUsage,
                crate::service::ai_service::ErrorResponse,
                crate::service::ai_service::TextGenerationResponse,
                crate::handlers::AIResponseText
            )
        ),
        tags(
            (name = "slam server", description = "Slam Server API")
        )
    )]
    struct ApiDoc;

    // 配置CORS中间件
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    // 创建Swagger UI并组合路由和CORS
    let swagger_ui = SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi());
    create_production_router(config).merge(swagger_ui).layer(cors)
}

/// 创建生产环境的路由
fn create_production_router(_config: AppConfig) -> Router {
    // 创建AI服务实例（使用默认配置）
    let ai_service = Arc::new(AIService::new());

    // 导入处理函数
    use crate::handlers::*;
    use crate::app::routes;

    // 创建路由
    Router::new()
        .route("/", get(root))
        .route(routes::API_STATUS, get(get_status))
        .route(routes::API_AI_GENERATE_TEXT, post(generate_text_handler))
        .with_state(ai_service)

}