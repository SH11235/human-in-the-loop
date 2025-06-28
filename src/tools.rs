use rmcp::model::{CallToolResult, Tool};
use schemars::{JsonSchema, gen::SchemaGenerator};
use rmcp::model::CallToolRequestParam;
use std::sync::Arc;
use std::borrow::Cow;
use serde::{Deserialize, Serialize};

#[async_trait::async_trait]
pub trait Human: Send + Sync + 'static {
    async fn ask(&self, question: &str) -> anyhow::Result<String>;
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AskHumanTool {
    /// The question to ask the human. Be specific and provide context to help the human understand what information you need.
    question: String,
}
impl AskHumanTool {
    pub async fn call_tool(&self, human: &dyn Human) -> anyhow::Result<CallToolResult> {
        let answer = human.ask(&self.question).await?;
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(answer)]))
    }
}

pub enum HumanTools {
    AskHumanTool(AskHumanTool),
}

impl HumanTools {
    pub fn tools() -> Vec<Tool> {
        let mut generator = SchemaGenerator::default();
        let schema = AskHumanTool::json_schema(&mut generator);
        let schema_value = serde_json::to_value(&schema).unwrap();
        let schema_map = match schema_value {
            serde_json::Value::Object(map) => Arc::new(map),
            _ => Arc::new(serde_json::Map::new()),
        };

        vec![
            Tool {
                name: Cow::Borrowed("ask_human"),
                description: Some(Cow::Borrowed("Ask a human for information that only they would know, such as personal preferences, project-specific context, local environment details, or non-public information")),
                input_schema: schema_map,
                annotations: None,
            }
        ]
    }
}

impl TryFrom<CallToolRequestParam> for HumanTools {
    type Error = String;

    fn try_from(request: CallToolRequestParam) -> Result<Self, Self::Error> {
        match request.name.as_ref() {
            "ask_human" => {
                let tool: AskHumanTool = serde_json::from_value(serde_json::Value::Object(request.arguments.unwrap_or_default()))
                    .map_err(|e| format!("Failed to parse ask_human tool: {}", e))?;
                Ok(HumanTools::AskHumanTool(tool))
            }
            _ => Err(format!("Unknown tool: {}", request.name)),
        }
    }
}
