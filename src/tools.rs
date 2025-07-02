use rmcp::{
    handler::server::tool::{Parameters, ToolRouter},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::future::Future;

#[async_trait::async_trait]
pub trait Human: Send + Sync + 'static {
    async fn ask(&self, question: &str) -> anyhow::Result<String>;
    async fn log_conversation(
        &self,
        role: &str,
        message: &str,
        context: Option<&str>,
    ) -> anyhow::Result<()>;
}

pub struct HumanInTheLoop<H> {
    human: H,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AskHumanRequest {
    #[schemars(
        description = "The question to ask the human. Be specific and provide context to help the human understand what information you need. Ask the only one question at once."
    )]
    question: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LogConversationRequest {
    #[schemars(description = "The role of the message sender: 'human', 'assistant', or 'system'")]
    role: String,
    #[schemars(description = "The message content to log")]
    message: String,
    #[schemars(description = "Optional context or metadata about the message")]
    context: Option<String>,
}

#[tool_router]
impl<H> HumanInTheLoop<H>
where
    H: Human,
{
    pub fn new(human: H) -> Self {
        Self {
            human,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Ask a human for information that only they would know, such as personal preferences, project-specific context, local environment details, or non-public information"
    )]
    async fn ask_human(
        &self,
        Parameters(AskHumanRequest { question }): Parameters<AskHumanRequest>,
    ) -> Result<String, rmcp::Error> {
        self.human
            .ask(&question)
            .await
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))
    }

    #[tool(
        description = "Log conversation messages to Discord for review and history tracking. Use this to record important interactions, decisions, or context that should be preserved"
    )]
    async fn log_conversation(
        &self,
        Parameters(LogConversationRequest {
            role,
            message,
            context,
        }): Parameters<LogConversationRequest>,
    ) -> Result<String, rmcp::Error> {
        self.human
            .log_conversation(&role, &message, context.as_deref())
            .await
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))?;
        Ok("Message logged successfully".to_string())
    }
}

#[tool_handler]
impl<H> ServerHandler for HumanInTheLoop<H>
where
    H: Human,
{
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "This is a Human-in-the-Loop MCP server that enables AI assistants to request \
             information from humans via Discord. Use the 'ask_human' tool when you need \
             information that only a human would know, such as: personal preferences, \
             project-specific context, local environment details, or any information that \
             is not publicly available or documented. The human will be notified in Discord \
             and their response will be returned to you. \
             \
             IMPORTANT: Please proactively use the 'log_conversation' tool to maintain a record of: \
             1. User requests and requirements (role='human') \
             2. Your responses and implementations (role='assistant') \
             3. Important decisions or milestones \
             4. Errors or issues encountered (role='system') \
             This helps maintain project history and enables better collaboration. \
             Logged messages will be formatted with role indicators and timestamps in a dedicated thread."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
