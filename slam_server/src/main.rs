use slam_server::app;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("warn,slam_server=info,tower_http=info")),
        )
        .with_target(false)
        .compact()
        .init();
    app::run().await;
}
