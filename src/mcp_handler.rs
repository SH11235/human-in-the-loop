use rmcp::{ServerHandler, RoleServer, Error as McpError};
use rmcp::model::{
    CallToolRequestParam, CallToolResult, ListToolsResult, PaginatedRequestParam,
};
use rmcp::service::RequestContext;

use crate::tools::{Human, HumanTools};

pub struct Handler<H> {
    human: H,
}

impl<H: Human> Handler<H> {
    pub fn new(human: H) -> Self {
        Self { human }
    }
}

impl<H: Human> ServerHandler for Handler<H> {
    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        async move {
            Ok(ListToolsResult {
                tools: HumanTools::tools(),
                next_cursor: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            let tool_params: HumanTools =
                HumanTools::try_from(request).map_err(|e| McpError::internal_error(e.to_string(), None))?;

            match tool_params {
                HumanTools::AskHumanTool(ask_human_tool) => {
                    ask_human_tool.call_tool(&self.human).await
                        .map_err(|e| McpError::internal_error(e.to_string(), None))
                }
            }
        }
    }
}
