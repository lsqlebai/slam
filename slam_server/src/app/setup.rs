use axum::{
    Router, extract::DefaultBodyLimit, routing::{get, post}
};
use std::net::SocketAddr;
use std::sync::Arc;
// removed unused imports
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// 导入服务相关模块
use crate::service::{ai_service::AIService, image_service::ImageService, user_service::UserService, sport_service::SportService};
use crate::dao::memory_cache::MemoryResultCache;
use crate::service::sport_service::StatSummary;
use crate::dao::sqlite_impl::SqliteImpl;
use std::sync::Arc as StdArc;
use crate::config::AppConfig;
use crate::handlers::jwt::Jwt;

// AppConfig 已迁移至 crate::config 模块

pub async fn run() {
    let config = AppConfig::default();
    let app = create_app(config.clone());

    let addr_str = format!("{}:{}", config.server.ip, config.server.port);
    let addr: SocketAddr = addr_str.parse().unwrap();
    println!("服务器正在监听 http://{}...", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("服务器正在监听 http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

/// 创建应用实例的通用函数
pub fn create_app(config: AppConfig) -> Router {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::handlers::get_status,
            crate::handlers::ai_handler::sports_image_recognition_handler,
            crate::handlers::root,
            crate::handlers::user_handler::user_register_handler,
            crate::handlers::user_handler::user_login_handler,
            crate::handlers::user_handler::user_info_handler,
            crate::handlers::user_handler::user_logout_handler,
            crate::handlers::sport_handler::insert_sport_handler,
            crate::handlers::sport_handler::import_sport_handler,
            crate::handlers::sport_handler::update_sport_handler,
            crate::handlers::sport_handler::list_sport_handler,
            crate::handlers::sport_handler::stats_handler,
            crate::handlers::sport_handler::delete_sport_handler
        ),
        components(
            schemas(
                crate::handlers::ApiResponse,
                crate::service::ai_service::TextGenerationRequest,
                crate::service::ai_service::TokenUsage,
                crate::service::ai_service::ErrorResponse,
                crate::model::sport::Sport,
                crate::handlers::ai_handler::AIResponseText,
                crate::handlers::user_handler::UserRegisterRequest,
                crate::handlers::user_handler::UserLoginRequest,
                crate::handlers::user_handler::UserActionResponse,
                crate::service::sport_service::StatBucket,
                crate::service::sport_service::TypeBucket,
                crate::service::sport_service::StatSummary,
                crate::handlers::sport_handler::ActionResponse,
                crate::handlers::sport_handler::ImportResponse,
                crate::handlers::sport_handler::DeleteRequest
            )
          ),
        tags(
            (name = "slam server", description = "Slam Server API")
        )
    )]
    struct ApiDoc;
    
    // 创建Swagger UI并组合路由和CORS
    let swagger_ui = SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi());
    create_production_router(config)
        .merge(swagger_ui)
}

pub struct AppState {
    pub ai_service: AIService,
    pub image_service: ImageService,
    pub user_service: UserService,
    pub sport_service: SportService,
    pub jwt: Jwt,
}
/// 创建生产环境的路由
fn create_production_router(config: AppConfig) -> Router {
    // 创建AI服务实例（使用默认配置）

    let sqlite_db = StdArc::new(SqliteImpl::new_sync(&config.db.path).expect("init sqlite dao"));
    let jwt = Jwt::new(config.security.jwt_ttl_seconds, config.security.key.clone());
    let cache = StdArc::new(MemoryResultCache::<StatSummary>::new());
    let app = Arc::new(AppState {
        ai_service: AIService::new(),
        image_service: ImageService::new(),
        user_service: UserService::new(sqlite_db.clone(), config.security.clone()),
        sport_service: SportService::new(sqlite_db.clone(), cache.clone()),
        jwt,
    });
    // 导入处理函数
    use crate::app::routes;
    use crate::handlers::*;

    // 创建路由，为所有处理函数提供相同的状态
    Router::new()
        .route("/", get(root))
        .route(routes::API_STATUS, get(get_status))
        .route(routes::API_IMAGE_PARSE, post(crate::handlers::ai_handler::sports_image_recognition_handler)).layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .route(routes::API_USER_REGISTER, post(crate::handlers::user_handler::user_register_handler))
        .route(routes::API_USER_LOGIN, post(crate::handlers::user_handler::user_login_handler))
        .route(routes::API_USER_INFO, get(crate::handlers::user_handler::user_info_handler))
        .route(routes::API_USER_LOGOUT, post(crate::handlers::user_handler::user_logout_handler))
        .route(routes::API_SPORT_INSERT, post(crate::handlers::sport_handler::insert_sport_handler))
        .route(routes::API_SPORT_IMPORT, post(crate::handlers::sport_handler::import_sport_handler))
        .route(routes::API_SPORT_UPDATE, post(crate::handlers::sport_handler::update_sport_handler))
        .route(routes::API_SPORT_DELETE, post(crate::handlers::sport_handler::delete_sport_handler))
        .route(routes::API_SPORT_LIST, get(crate::handlers::sport_handler::list_sport_handler))
        .route(routes::API_SPORT_STATS, get(crate::handlers::sport_handler::stats_handler))
        .with_state(app)
}
