use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use slam_server::app::{self, AppConfig};
use slam_server::model::sport::SAMPLE_XML_SWIMMING;
use slam_server::service::llm::{ChatCompletionRequest, LLM};

struct MockLlm {
    delay: Duration,
}

#[async_trait]
impl LLM for MockLlm {
    async fn chat(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if !self.delay.is_zero() {
            tokio::time::sleep(self.delay).await;
        }
        Ok(SAMPLE_XML_SWIMMING.to_string())
    }
}

#[tokio::main]
async fn main() {
    let mut config = AppConfig::default();
    config.server.ip = "127.0.0.1".to_string();
    config.server.port = 3000;
    if let Ok(path) = std::env::var("SLAM_E2E_DB_PATH") {
        config.db.path = path;
    }
    if let Ok(path) = std::env::var("SLAM_E2E_JOB_DIR") {
        config.ai.job_dir = path;
    }
    config.ai.retry_delays_seconds = vec![0, 0];

    let delay = std::env::var("SLAM_E2E_LLM_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_default();
    let router = app::create_app_with_llm(config.clone(), Arc::new(MockLlm { delay })).await;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", config.server.port))
        .await
        .expect("bind mock server");
    println!("mock AI server listening on http://127.0.0.1:3000");
    axum::serve(listener, router)
        .await
        .expect("serve mock server");
}
