use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn summarize_diff(&self, diff: &str) -> Result<String>;
    async fn recommend_cleanup(&self, branch_info: &str) -> Result<String>;
    #[allow(dead_code)]
    async fn resolve_conflict(&self, base: &str, head: &str, conflict: &str) -> Result<String>;
}

pub mod ollama;
pub mod openai;
