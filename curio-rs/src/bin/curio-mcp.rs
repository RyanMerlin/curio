//! Local, read-only MCP facade over Curio's shared retrieval library.
use anyhow::Result;
use curio::{
    acl::AccessContext,
    retrieval::{
        RetrieveRequest, accessible_published_pages, fetch_published_with_access,
        retrieve_published_with_access,
    },
    workspace::resolve_kb_dir,
};
use rmcp::{
    ServiceExt, handler::server::wrapper::Parameters, schemars, tool, tool_router, transport::stdio,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
struct SearchParams {
    query: String,
    category: Option<String>,
    limit: Option<usize>,
    freshness: Option<String>,
}
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
struct FetchParams {
    id: String,
}
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
struct EmptyParams {}

#[derive(Clone)]
struct CurioMcp {
    wiki_dir: PathBuf,
    access: Option<AccessContext>,
}
fn error(code: &str, message: impl Into<String>) -> String {
    serde_json::json!({"error":{"code":code,"message":message.into()}}).to_string()
}
fn json<T: Serialize>(value: T) -> String {
    serde_json::to_string(&value)
        .unwrap_or_else(|_| error("serialization_error", "Unable to serialize response"))
}

#[tool_router(server_handler)]
impl CurioMcp {
    #[tool(
        description = "Search canonical published Curio pages. Results include stable IDs and provenance."
    )]
    async fn search(&self, Parameters(params): Parameters<SearchParams>) -> String {
        let _freshness = params.freshness.as_deref();
        if params.query.trim().is_empty() {
            return error("invalid_query", "query must not be empty");
        }
        let limit = params.limit.unwrap_or(5).min(100);
        match retrieve_published_with_access(
            &self.wiki_dir,
            &RetrieveRequest {
                query: params.query,
                category: params.category,
                limit,
            },
            self.access.as_ref(),
        ) {
            Ok(response) => json(response),
            Err(e) => error("search_failed", format!("{e:#}")),
        }
    }

    #[tool(
        description = "Fetch canonical Markdown and complete provenance by stable Curio retrieval ID."
    )]
    async fn fetch(&self, Parameters(params): Parameters<FetchParams>) -> String {
        match fetch_published_with_access(&self.wiki_dir, &params.id, self.access.as_ref()) {
            Ok(response) => json(response),
            Err(e) => error("fetch_not_found", format!("{e:#}")),
        }
    }

    #[tool(description = "List categories visible in the selected workspace's published taxonomy.")]
    async fn list_categories(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        match accessible_published_pages(&self.wiki_dir, self.access.as_ref()) {
            Ok(pages) => {
                let categories = pages
                    .into_iter()
                    .map(|page| page.category)
                    .filter(|category| !category.is_empty())
                    .collect::<std::collections::BTreeSet<_>>();
                json(serde_json::json!({"categories": categories.into_iter().collect::<Vec<_>>() }))
            }
            Err(e) => error("metadata_failed", format!("{e:#}")),
        }
    }

    #[tool(
        description = "Report workspace identity, published page count, latest published Git revision, and index status."
    )]
    async fn knowledge_status(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let published = self.wiki_dir.join("published");
        let count = match accessible_published_pages(&self.wiki_dir, self.access.as_ref()) {
            Ok(pages) => pages.len(),
            Err(e) => return error("metadata_failed", format!("{e:#}")),
        };
        let workspace = self
            .wiki_dir
            .file_name()
            .and_then(|x| x.to_str())
            .filter(|name| *name != "wiki")
            .or_else(|| {
                self.wiki_dir
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|x| x.to_str())
            })
            .unwrap_or("workspace");
        json(
            serde_json::json!({"workspace":workspace,"published_page_count":count,"latest_published_commit":latest_commit(&self.wiki_dir),"index":{"exists":published.join("index.md").exists(),"status":"local"}}),
        )
    }
}

fn latest_commit(wiki_dir: &std::path::Path) -> Option<String> {
    let mut root = wiki_dir;
    let root = loop {
        if root.join(".git").exists() {
            break root;
        }
        root = root.parent()?;
    };
    let published = wiki_dir.join("published");
    let relative = published.strip_prefix(root).ok()?.to_string_lossy();
    let output = std::process::Command::new("git")
        .args(["log", "-1", "--format=%H", "--"])
        .arg(relative.as_ref())
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string()).filter(|x| !x.is_empty())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut kb_dir = None;
    let mut workspace = None;
    let mut principals = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--kb-dir" => kb_dir = args.next().map(PathBuf::from),
            "--workspace" => workspace = args.next(),
            "--principal" => {
                if let Some(id) = args.next() {
                    principals.push(id);
                }
            }
            "--help" | "-h" => {
                eprintln!("curio-mcp [--kb-dir PATH | --workspace NAME] [--principal ID]");
                return Ok(());
            }
            other => anyhow::bail!("unknown argument {other}"),
        }
    }
    let kb_root = resolve_kb_dir(kb_dir.as_ref(), workspace.as_deref())?;
    // Curio supports both the historical KB-root layout and external stores
    // whose configured wiki directory is a nested `wiki/` tree.
    let wiki_dir = if kb_root.join("published").exists() {
        kb_root
    } else {
        kb_root.join("wiki")
    };
    let access = (!principals.is_empty()).then(|| AccessContext::new(principals));
    let service = CurioMcp { wiki_dir, access }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
