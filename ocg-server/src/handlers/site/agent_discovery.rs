//! Machine-readable discovery resources for public and agent clients.

use axum::{
    extract::State,
    http::header::CONTENT_TYPE,
    response::{IntoResponse, Response},
};
use rust_embed::Embed;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    handlers::{error::HandlerError, extend_public_shared_cache_headers},
    router::State as RouterState,
};

const CONTENT_TYPE_LINKSET_JSON: &str = "application/linkset+json; charset=utf-8";
const CONTENT_TYPE_MARKDOWN: &str = "text/markdown; charset=utf-8";
const CONTENT_TYPE_XML: &str = "application/xml; charset=utf-8";
const CONTENT_TYPE_TEXT: &str = "text/plain; charset=utf-8";

#[derive(Embed)]
#[folder = "../docs"]
struct DocsAsset;

/// Serves crawler policy and public AI-content preferences.
pub(crate) async fn robots(State(state): State<RouterState>) -> Result<Response, HandlerError> {
    let base_url = canonical_base_url(&state.server_cfg.base_url);
    let body = format!(
        "User-agent: *\nAllow: /\nDisallow: /dashboard/\nDisallow: /api/\nDisallow: /log-in\nDisallow: /sign-up\nDisallow: /webhooks/\n\nUser-agent: GPTBot\nAllow: /\n\nUser-agent: OAI-SearchBot\nAllow: /\n\nUser-agent: Claude-Web\nAllow: /\n\nUser-agent: Google-Extended\nAllow: /\n\nContent-Signal: ai-train=no, search=yes, ai-input=no\nSitemap: {base_url}/sitemap.xml\n"
    );
    text_response(CONTENT_TYPE_TEXT, body)
}

/// Serves a canonical sitemap for static public pages and embedded documentation.
pub(crate) async fn sitemap(State(state): State<RouterState>) -> Result<Response, HandlerError> {
    let base_url = canonical_base_url(&state.server_cfg.base_url);
    let mut paths = vec![
        "/".to_string(),
        "/about".to_string(),
        "/docs".to_string(),
        "/explore".to_string(),
        "/jobs".to_string(),
        "/landscape".to_string(),
        "/privacy".to_string(),
        "/stats".to_string(),
        "/wiki".to_string(),
    ];
    paths.extend(documentation_paths());
    paths.sort();
    paths.dedup();

    let urls = paths
        .iter()
        .map(|path| {
            format!(
                "  <url><loc>{}</loc></url>",
                xml_escape(&format!("{base_url}{path}"))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    text_response(
        CONTENT_TYPE_XML,
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{urls}\n</urlset>\n"
        ),
    )
}

/// Serves the existing OpenAPI description at a stable public URL.
pub(crate) async fn openapi() -> Result<Response, HandlerError> {
    let Some(file) = DocsAsset::get("openapi.yaml") else {
        return Err(HandlerError::NotFound);
    };
    text_response(
        "application/vnd.oai.openapi;version=3.0.3; charset=utf-8",
        String::from_utf8(file.data.into_owned())
            .map_err(|error| HandlerError::Other(anyhow::anyhow!(error)))?,
    )
}

/// Serves API Catalog metadata for API-aware clients.
pub(crate) async fn api_catalog(
    State(state): State<RouterState>,
) -> Result<Response, HandlerError> {
    let base_url = canonical_base_url(&state.server_cfg.base_url);
    let body = serde_json::to_string_pretty(&json!({
        "linkset": [{
            "anchor": format!("{base_url}/api/v1"),
            "service-desc": [{"href": format!("{base_url}/openapi.yaml"), "type": "application/vnd.oai.openapi"}],
            "service-doc": [{"href": format!("{base_url}/docs/api")}],
            "status": [{"href": format!("{base_url}/api/v1/health")}]
        }]
    }))
    .map_err(|error| HandlerError::Other(anyhow::anyhow!(error)))?;
    text_response(CONTENT_TYPE_LINKSET_JSON, body)
}

/// Describes the authenticated operational MCP endpoint.
pub(crate) async fn mcp_server_card(
    State(state): State<RouterState>,
) -> Result<Response, HandlerError> {
    let base_url = canonical_base_url(&state.server_cfg.base_url);
    let body = serde_json::to_string_pretty(&json!({
        "serverInfo": {"name": "goup-vc", "version": env!("CARGO_PKG_VERSION")},
        "description": "GOUP Alliance operational tools. A bearer token is required.",
        "transport": {"type": "streamable-http", "url": format!("{base_url}/mcp")},
        "capabilities": {"tools": {"listChanged": false}},
        "authentication": {"schemes": ["bearer"]}
    }))
    .map_err(|error| HandlerError::Other(anyhow::anyhow!(error)))?;
    text_response("application/json; charset=utf-8", body)
}

/// Serves the Agent Skills discovery index for the MCP connection guide.
pub(crate) async fn agent_skills_index(
    State(state): State<RouterState>,
) -> Result<Response, HandlerError> {
    let base_url = canonical_base_url(&state.server_cfg.base_url);
    let skill = mcp_skill_document(&base_url);
    let digest = hex::encode(Sha256::digest(skill.as_bytes()));
    let body = serde_json::to_string_pretty(&json!({
        "$schema": "https://agentskills.io/schemas/agent-skills-index-v0.2.0.json",
        "skills": [{
            "name": "goup-mcp",
            "type": "skill-md",
            "description": "Connect to GOUP's authenticated operational MCP server.",
            "url": format!("{base_url}/.well-known/agent-skills/goup-mcp/SKILL.md"),
            "sha256": digest
        }]
    }))
    .map_err(|error| HandlerError::Other(anyhow::anyhow!(error)))?;
    text_response("application/json; charset=utf-8", body)
}

/// Serves the MCP connection guide referenced by the skills index.
pub(crate) async fn mcp_skill(State(state): State<RouterState>) -> Result<Response, HandlerError> {
    text_response(
        CONTENT_TYPE_MARKDOWN,
        mcp_skill_document(&canonical_base_url(&state.server_cfg.base_url)),
    )
}

fn text_response(content_type: &'static str, body: String) -> Result<Response, HandlerError> {
    Ok((
        extend_public_shared_cache_headers(&[(CONTENT_TYPE.as_str(), content_type)])?,
        body,
    )
        .into_response())
}

fn canonical_base_url(base_url: &str) -> String {
    base_url.trim_end_matches('/').to_string()
}

fn documentation_paths() -> Vec<String> {
    DocsAsset::iter()
        .filter_map(|path| {
            let path = path.as_ref();
            let stem = path.strip_suffix(".md")?;
            Some(if stem == "index" {
                "/docs".to_string()
            } else {
                format!("/docs/{stem}")
            })
        })
        .collect()
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn mcp_skill_document(base_url: &str) -> String {
    format!(
        "# GOUP MCP\n\nUse GOUP's operational MCP endpoint only with an authorized bearer token.\n\n- Endpoint: `{base_url}/mcp`\n- Transport: Streamable HTTP JSON-RPC\n- Authentication: `Authorization: Bearer <token>`\n- Read-only operations are available by default; mutations depend on server policy.\n\nUse `tools/list` before calling tools. Do not send credentials or personal data to any other endpoint.\n"
    )
}
