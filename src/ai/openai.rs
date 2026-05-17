use crate::ai::AIProvider;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[async_trait]
impl AIProvider for OpenAIProvider {
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

impl OpenAIProvider {
    async fn query(&self, prompt: &str) -> Result<String> {
        let req = ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let res = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;

        Ok(res
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}
