use crate::asn_ip::{self, ASNInfo, upstream::Upstream};
use rmcp::{
    ErrorData, Json, RoleServer, ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{
        Implementation, InitializeRequestParam, InitializeResult, ProtocolVersion,
        ServerCapabilities, ServerInfo,
    },
    schemars::JsonSchema,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ASNSubnet {
    upstream: Arc<RwLock<Upstream>>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SubnetRequest {
    asn: u32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SubnetResponse {
    asn: u32,
    subnets: Subnets,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Subnets {
    ipv4: Vec<String>,
    ipv6: Vec<String>,
}

impl From<asn_ip::Subnets> for Subnets {
    fn from(subnets: asn_ip::Subnets) -> Self {
        Self {
            ipv4: subnets.ipv4.iter().map(|net| net.to_string()).collect(),
            ipv6: subnets.ipv6.iter().map(|net| net.to_string()).collect(),
        }
    }
}

#[tool_router]
impl ASNSubnet {
    pub fn new(upstream: Arc<RwLock<Upstream>>) -> Self {
        Self {
            upstream,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Get subnet information for a given ASN number",
        name = "get_asn_subnets"
    )]
    async fn get_asn_subnets(
        &self,
        Parameters(request): Parameters<SubnetRequest>,
    ) -> Result<Json<SubnetResponse>, String> {
        let upstream = self.upstream.read().await;

        // Get the ASN file path
        let file_path = upstream.get_asn_file_path(request.asn);

        // Read and parse the ASN info
        let asn_data = tokio::fs::read_to_string(&file_path)
            .await
            .map_err(|e| format!("Failed to read ASN file: {}", e))?;

        let asn_info: ASNInfo = serde_json::from_str(&asn_data)
            .map_err(|e| format!("Failed to parse ASN data: {}", e))?;

        Ok(Json(SubnetResponse {
            asn: request.asn,
            subnets: asn_info.subnets.into(),
        }))
    }
}

#[tool_handler]
impl ServerHandler for ASNSubnet {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides counter tools and prompts. Tools: increment, decrement, get_value, say_hello, echo, sum. Prompts: example_prompt (takes a message), counter_analysis (analyzes counter state with a goal).".to_string()),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
        }
        Ok(ServerHandler::get_info(self))
    }
}
