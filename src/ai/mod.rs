use anyhow::Result;
use async_trait::async_trait;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::openai;

#[async_trait]
pub trait TwigdropAI: Send + Sync {
    async fn summarize_diff(&self, diff: &str) -> Result<String>;
    async fn recommend_cleanup(&self, branch_info: &str) -> Result<String>;
    #[allow(dead_code)]
    async fn resolve_conflict(&self, conflict_block: &str) -> Result<String>;
}

pub struct RigAgentWrapper<M: rig::completion::CompletionModel + Send + Sync + 'static> {
    pub agent: rig::agent::Agent<M>,
}

#[async_trait]
impl<M: rig::completion::CompletionModel + Send + Sync + 'static> TwigdropAI
    for RigAgentWrapper<M>
{
    async fn summarize_diff(&self, diff: &str) -> Result<String> {
        let prompt = format!(
            "Summarize this git diff in 3 concise bullet points. Focus on intent and impact:\n\n{}",
            diff
        );
        Ok(self.agent.prompt(&prompt).await?)
    }

    async fn recommend_cleanup(&self, branch_info: &str) -> Result<String> {
        let prompt = format!(
            "Analyze this branch info. Should it be deleted? Provide a 1-sentence recommendation:\n\n{}",
            branch_info
        );
        Ok(self.agent.prompt(&prompt).await?)
    }

    async fn resolve_conflict(&self, conflict_block: &str) -> Result<String> {
        let prompt = format!(
            "Resolve the following git conflict. Return ONLY the resolved code block, no prose:\n\n{}",
            conflict_block
        );
        Ok(self.agent.prompt(&prompt).await?)
    }
}

pub struct AIWorker {
    pub inner: Box<dyn TwigdropAI>,
}

impl AIWorker {
    pub fn new(
        provider_type: &str,
        model: &str,
        api_key: Option<String>,
        url: Option<String>,
    ) -> Result<Self> {
        let preamble = "You are Twigdrop AI, a specialized git assistant. 
            Your goal is to analyze branches, summarize diffs, and resolve conflicts.
            Keep your responses concise and focused on the technical changes.";

        let key = api_key.unwrap_or_else(|| "unused".to_string());

        if provider_type == "openai" {
            let client = openai::Client::builder()
                .api_key(&key)
                .build()
                .map_err(|e| anyhow::anyhow!(e))?;
            let agent = client.agent(model).preamble(preamble).build();
            Ok(Self {
                inner: Box::new(RigAgentWrapper { agent }),
            })
        } else {
            let base_url = url.unwrap_or_else(|| "http://localhost:11434/v1".to_string());
            let client = openai::Client::builder()
                .api_key(&key)
                .base_url(&base_url)
                .build()
                .map_err(|e| anyhow::anyhow!(e))?;
            let agent = client.agent(model).preamble(preamble).build();
            Ok(Self {
                inner: Box::new(RigAgentWrapper { agent }),
            })
        }
    }
}
