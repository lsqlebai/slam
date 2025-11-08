use slam_server::app;

#[tokio::main]
async fn main() {
    // 启动服务器
    app::run().await;
}
