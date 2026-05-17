use crate::ai::AIProvider;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OllamaProvider {
    client: Client,
    url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            url,
            model,
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn summarize_diff(&self, diff: &str) -> Result<String> {
        let prompt = format!(
            "Summarize the following git diff in 3 concise bullet points:\n\n{}",
            diff
        );
        self.query(&prompt).await
    }

    async fn recommend_cleanup(&self, branch_info: &str) -> Result<String> {
        let prompt = format!(
            "Based on the following branch information, should this branch be deleted? Why?\n\n{}",
            branch_info
        );
        self.query(&prompt).await
    }

    async fn resolve_conflict(&self, base: &str, head: &str, conflict: &str) -> Result<String> {
        let prompt = format!(
            "Resolve the following git conflict.\nBase version:\n{}\n\nHead version:\n{}\n\nConflict block:\n{}\n\nReturn only the resolved code block.",
            base, head, conflict
        );
        self.query(&prompt).await
    }
}

impl OllamaProvider {
    async fn query(&self, prompt: &str) -> Result<String> {
        let req = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let res = self
            .client
            .post(format!("{}/api/generate", self.url))
            .json(&req)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        Ok(res.response)
    }
}
