use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "curio", version, about = "Git-native enterprise knowledge store")]
pub struct Cli {
    #[arg(long, global = true, help = "Path to the configuration file")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, help = "Print what would be done without making changes")]
    pub dry_run: bool,

    #[arg(long, global = true, help = "Emit machine-readable JSON output")]
    pub json: bool,

    #[arg(long, global = true, help = "Set the logging level (e.g. INFO, DEBUG)")]
    pub log_level: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect and synchronize the Curio onboarding state.
    Onboard {
        #[arg(long)]
        install: bool,
    },

    /// Verify provider and harness prerequisites.
    Doctor {
        #[arg(value_enum)]
        provider: Option<AgentProvider>,
    },

    /// Create the wiki/ directory scaffold and seed index files.
    ///
    /// Idempotent — safe to run on an existing wiki.
    Init {
        /// Recreate all scaffold files from scratch (preserves existing pages).
        #[arg(long)]
        reset: bool,
    },

    /// Ingest content from a URL, file, or Confluence page into wiki/intake/.
    Intake {
        /// Web URL or Confluence page URL to ingest.
        #[arg(long, group = "source")]
        url: Option<String>,

        /// Path to a local file to ingest.
        #[arg(long, group = "source")]
        file: Option<PathBuf>,

        /// Path to a local folder to ingest recursively.
        #[arg(long, group = "source")]
        folder: Option<PathBuf>,

        /// Optional title hint.
        #[arg(long)]
        title: Option<String>,

        /// Optional subject or category hint.
        #[arg(long)]
        subject_hint: Option<String>,
    },

    /// Route intake pages to staged/ or review/ using heuristics or agent decisions.
    Process {
        /// Maximum number of intake items to process.
        #[arg(long, default_value = "10")]
        limit: u32,

        /// If set, use only heuristics (no agent).
        #[arg(long)]
        auto: bool,

        /// Apply a pre-computed routing decision JSON file.
        #[arg(long)]
        route_file: Option<PathBuf>,

        /// Route a specific page by its slug.
        #[arg(long)]
        slug: Option<String>,

        /// Category path for the routed page (e.g. "by-account/acme").
        #[arg(long)]
        category: Option<String>,

        /// Status to route to: staged or review.
        #[arg(long)]
        status: Option<String>,

        /// Comma-separated keywords.
        #[arg(long)]
        keywords: Option<String>,

        /// Confidence score (0.0–1.0).
        #[arg(long)]
        confidence: Option<f32>,

        /// Short summary of the content.
        #[arg(long)]
        summary: Option<String>,
    },

    /// List items in review/ and staged/ with status summaries.
    Review {
        /// Filter to only show items in a specific lane: review, staged, or all (default).
        #[arg(long, default_value = "all")]
        lane: String,
    },

    /// Move a review/ item to staged/ after human judgment.
    Resolve {
        /// Slug of the page to resolve.
        slug: String,

        /// Optional: override category on resolve (e.g. "by-account/acme").
        #[arg(long)]
        category: Option<String>,
    },

    /// Publish a staged/ page to published/.
    Publish {
        /// Slug of the staged page to publish.
        slug: String,

        /// Target category path within published/ (e.g. "by-account/acme").
        /// Defaults to the category stored in the page's frontmatter.
        #[arg(long)]
        category: Option<String>,
    },

    /// Search the wiki registry by keyword, category, or status.
    Search {
        /// Keywords to search for.
        #[arg(long)]
        keywords: Option<String>,

        /// Filter by category (e.g. "by-account").
        #[arg(long)]
        category: Option<String>,

        /// Filter by status (intake | staged | review | published).
        #[arg(long)]
        status: Option<String>,

        /// Free-text search across page bodies.
        #[arg(long)]
        text: Option<String>,

        /// Maximum results.
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Rebuild all wiki/_index/ files by walking wiki/**/*.md.
    Reindex,

    /// Sync wiki/published/ directory structure to match the NORTHSTAR blueprint.
    /// Run this after editing NORTHSTAR to create new tree/subtree dirs and remove empty stale ones.
    Tree,

    /// Push wiki/published/ to Confluence (one-way sync).
    Sync {
        /// Override the Confluence parent page ID from config.
        #[arg(long)]
        parent_page_id: Option<String>,

        /// Show what would be pushed without making Confluence API calls.
        #[arg(long)]
        dry_run: bool,
    },

    /// Scan the wiki for contradictions, stale claims, and orphaned cross-references.
    Lint {
        /// Auto-fix orphaned cross-references.
        #[arg(long)]
        fix: bool,
    },

    /// Answer a natural-language question by querying the wiki.
    Query {
        /// The question to answer.
        question: String,

        /// File the synthesized answer back into the wiki.
        #[arg(long)]
        save: bool,
    },

    /// Harness commands for launching supported agent providers.
    #[command(subcommand)]
    Agent(AgentCommands),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum AgentProvider {
    Codex,
    Claude,
    Gemini,
}

impl AgentProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            AgentProvider::Codex => "codex",
            AgentProvider::Claude => "claude",
            AgentProvider::Gemini => "gemini",
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    /// Prepare the Curio harness environment for a provider.
    Prepare {
        #[arg(value_enum)]
        provider: AgentProvider,
    },

    /// Launch a provider inside the Curio harness workspace.
    Launch {
        #[arg(value_enum)]
        provider: AgentProvider,
    },

    /// Verify provider and harness prerequisites.
    Doctor {
        #[arg(value_enum)]
        provider: Option<AgentProvider>,
    },

    /// List supported providers and their detected availability.
    ListProviders,

    /// List Curio-authored skills available to the harness.
    ListSkills,

    /// List Curio plugins from the marketplace catalog.
    ListPlugins,

    /// Print the environment Curio will provide to a provider.
    PrintEnv {
        #[arg(value_enum)]
        provider: AgentProvider,
    },
}
