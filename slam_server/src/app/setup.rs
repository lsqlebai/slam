use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;
// removed unused imports
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// 导入服务相关模块
use crate::config::AppConfig;
use crate::dao::Repository;
use crate::dao::cache::memory::MemoryResultCache;
use crate::handlers::jwt::Jwt;
use crate::service::sport_service::StatSummary;
use crate::service::{
    ai_job_service::AIJobService, ai_job_worker::start_workers, ai_service::AIService,
    image_service::ImageService, llm::LLM, sport_service::SportService, user_service::UserService,
};
use std::sync::Arc as StdArc;

// AppConfig 已迁移至 crate::config 模块

pub async fn run() {
    let config = AppConfig::default();
    let app = create_app(config.clone()).await;

    let addr_str = format!("{}:{}", config.server.ip, config.server.port);
    let addr: SocketAddr = addr_str.parse().unwrap();
    println!("服务器正在监听 http://{}...", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("服务器正在监听 http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

/// 创建应用实例的通用函数
pub async fn create_app(config: AppConfig) -> Router {
    create_app_inner(config, None).await
}

pub async fn create_app_with_llm(config: AppConfig, llm: Arc<dyn LLM + Send + Sync>) -> Router {
    create_app_inner(config, Some(llm)).await
}

async fn create_app_inner(config: AppConfig, llm: Option<Arc<dyn LLM + Send + Sync>>) -> Router {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::handlers::get_status,
            crate::handlers::ai_handler::sports_image_recognition_handler,
            crate::handlers::ai_job_handler::create_ai_job_handler,
            crate::handlers::ai_job_handler::list_ai_jobs_handler,
            crate::handlers::ai_job_handler::get_ai_job_handler,
            crate::handlers::ai_job_handler::delete_ai_job_handler,
            crate::handlers::ai_job_handler::retry_ai_job_handler,
            crate::handlers::ai_job_handler::get_ai_asset_handler,
            crate::handlers::ai_job_handler::get_ai_asset_thumbnail_handler,
            crate::handlers::root,
            crate::handlers::user_handler::user_register_handler,
            crate::handlers::user_handler::user_login_handler,
            crate::handlers::user_handler::user_info_handler,
            crate::handlers::user_handler::user_logout_handler,
            crate::handlers::user_handler::user_avatar_upload_handler,
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
                crate::model::ai_job::AiJobView,
                crate::model::ai_job::AiJobAsset,
                crate::handlers::ai_handler::AIResponseText,
                crate::handlers::user_handler::UserRegisterRequest,
                crate::handlers::user_handler::UserLoginRequest,
                crate::handlers::user_handler::UserActionResponse,
                crate::handlers::user_handler::AvatarUploadResponse,
                crate::service::sport_service::StatBucket,
                crate::service::sport_service::TypeBucket,
                crate::service::sport_service::StatSummary,
                crate::handlers::sport_handler::ActionResponse,
                crate::handlers::sport_handler::InsertSportRequest,
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
    create_production_router(config, llm)
        .await
        .merge(swagger_ui)
}

pub struct AppState {
    pub ai_service: Arc<AIService>,
    pub image_service: Arc<ImageService>,
    pub ai_job_service: Arc<AIJobService>,
    pub user_service: UserService,
    pub sport_service: SportService,
    pub jwt: Jwt,
}
/// 创建生产环境的路由
async fn create_production_router(
    config: AppConfig,
    llm: Option<Arc<dyn LLM + Send + Sync>>,
) -> Router {
    // 创建AI服务实例（使用默认配置）

    let sqlite_db = StdArc::new(
        Repository::new(&config.db.path)
            .await
            .expect("init repository"),
    );
    let jwt = Jwt::new(config.security.jwt_ttl_seconds, config.security.key.clone());
    let cache_total = StdArc::new(MemoryResultCache::<StatSummary, i32>::new());
    let cache_year = StdArc::new(MemoryResultCache::<StatSummary, String>::new());
    let ai_service = Arc::new(match llm {
        Some(llm) => AIService::with_llm(llm),
        None => AIService::with_config(config.ai.model.clone(), config.ai.key.clone()),
    });
    let image_service = Arc::new(ImageService::new());
    let notify = Arc::new(tokio::sync::Notify::new());
    let ai_job_service = Arc::new(AIJobService::new(
        sqlite_db.clone(),
        image_service.clone(),
        config.ai.job_dir.clone(),
        notify,
    ));
    start_workers(
        config.ai.worker_concurrency,
        config.ai.max_attempts,
        config.ai.retry_delays_seconds.clone(),
        ai_job_service.clone(),
        ai_service.clone(),
        image_service.clone(),
    );
    let app = Arc::new(AppState {
        ai_service,
        image_service,
        ai_job_service,
        user_service: UserService::new(sqlite_db.clone(), config.security.clone()),
        sport_service: SportService::new(
            sqlite_db.clone(),
            cache_total.clone(),
            cache_year.clone(),
        ),
        jwt,
    });
    // 导入处理函数
    use crate::app::routes;
    use crate::handlers::*;

    // 创建路由，为所有处理函数提供相同的状态
    Router::new()
        .route("/", get(root))
        .route(routes::API_STATUS, get(get_status))
        .route(
            routes::API_IMAGE_PARSE,
            post(crate::handlers::ai_handler::sports_image_recognition_handler)
                .layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
        .route(
            routes::API_AI_JOBS,
            post(crate::handlers::ai_job_handler::create_ai_job_handler)
                .get(crate::handlers::ai_job_handler::list_ai_jobs_handler)
                .layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
        .route(
            routes::API_AI_JOB,
            get(crate::handlers::ai_job_handler::get_ai_job_handler)
                .delete(crate::handlers::ai_job_handler::delete_ai_job_handler),
        )
        .route(
            routes::API_AI_JOB_RETRY,
            post(crate::handlers::ai_job_handler::retry_ai_job_handler),
        )
        .route(
            routes::API_AI_ASSET,
            get(crate::handlers::ai_job_handler::get_ai_asset_handler),
        )
        .route(
            routes::API_AI_ASSET_THUMBNAIL,
            get(crate::handlers::ai_job_handler::get_ai_asset_thumbnail_handler),
        )
        .route(
            routes::API_USER_REGISTER,
            post(crate::handlers::user_handler::user_register_handler),
        )
        .route(
            routes::API_USER_LOGIN,
            post(crate::handlers::user_handler::user_login_handler),
        )
        .route(
            routes::API_USER_INFO,
            get(crate::handlers::user_handler::user_info_handler),
        )
        .route(
            routes::API_USER_LOGOUT,
            post(crate::handlers::user_handler::user_logout_handler),
        )
        .route(
            routes::API_USER_AVATAR_UPLOAD,
            post(crate::handlers::user_handler::user_avatar_upload_handler)
                .layer(DefaultBodyLimit::max(20 * 1024 * 1024)),
        )
        .route(
            routes::API_SPORT_INSERT,
            post(crate::handlers::sport_handler::insert_sport_handler),
        )
        .route(
            routes::API_SPORT_IMPORT,
            post(crate::handlers::sport_handler::import_sport_handler),
        )
        .route(
            routes::API_SPORT_UPDATE,
            post(crate::handlers::sport_handler::update_sport_handler),
        )
        .route(
            routes::API_SPORT_DELETE,
            post(crate::handlers::sport_handler::delete_sport_handler),
        )
        .route(
            routes::API_SPORT_LIST,
            get(crate::handlers::sport_handler::list_sport_handler),
        )
        .route(
            routes::API_SPORT_STATS,
            get(crate::handlers::sport_handler::stats_handler),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
        .with_state(app)
}
