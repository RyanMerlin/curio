use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "curio", version, about = "Git-native enterprise knowledge store")]
pub struct Cli {
    #[arg(long, global = true, help = "Path to the configuration file")]
    pub config: Option<PathBuf>,

    /// Path to the KB store directory to operate on.
    /// Overrides --workspace and CURIO_REPO_ROOT.
    #[arg(long, global = true, help = "Path to the KB store directory")]
    pub kb_dir: Option<PathBuf>,

    /// Named workspace from curio.workspaces.toml.
    /// Ignored when --kb-dir is provided.
    #[arg(long, global = true, help = "Named workspace to use")]
    pub workspace: Option<String>,

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

    /// Report structural health of the published knowledge base.
    ///
    /// Checks for low-quality pages, high-overlap pairs, stale content,
    /// orphaned cross-references, thin branch nodes, and missing keywords.
    ///
    /// Use `curio agent doctor` to verify provider/harness prerequisites.
    Doctor {
        /// Restrict scan to a NORTHSTAR path (e.g. product-tree/alteryx-server).
        /// Defaults to the full published KB.
        #[arg(long)]
        scope: Option<String>,
    },

    /// Create the wiki/ directory scaffold and seed index files.
    ///
    /// Idempotent — safe to run on an existing wiki.
    Init {
        /// Recreate all scaffold files from scratch and rebuild the managed Confluence tree.
        #[arg(long)]
        reset: bool,

        /// Required with --reset. Allows deleting managed descendants under the CURIO root page.
        #[arg(long)]
        confirm_nuke: bool,
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

        /// When ingesting a Confluence URL, also fetch all descendant pages.
        #[arg(long)]
        recursive: bool,
    },

    /// Route intake pages to staged/ or review/.
    ///
    /// Agent-native: outputs a routing manifest for the agent to reason over,
    /// then applies decisions via --route-file. Default cap: 10 pages.
    Process {
        /// Maximum number of intake items to process. Ignored when --all is set.
        #[arg(long, default_value = "10")]
        limit: u32,

        /// Process all intake pages, removing the default limit cap.
        #[arg(long)]
        all: bool,

        /// Output a routing manifest JSON to stdout and exit (no routing applied).
        /// The agent reads this, makes decisions, then calls --route-file.
        #[arg(long)]
        prepare: bool,

        /// Apply a pre-computed routing decision JSON file produced by the agent.
        #[arg(long)]
        route_file: Option<PathBuf>,

        /// Route a specific page by its slug (manual single-page routing).
        #[arg(long)]
        slug: Option<String>,

        /// Category path for manual routing (e.g. "product-tree/alteryx-server").
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

    /// Initialize a new KB store at the given path.
    ///
    /// Creates the wiki/ scaffold, NORTHSTAR.md template, curio.toml template,
    /// and a .gitignore that excludes .env. Optionally registers the KB in
    /// curio.workspaces.toml so it can be referenced by --workspace.
    InitKb {
        /// Directory to create the KB in. Defaults to ~/curio-kb.
        #[arg(long)]
        path: Option<PathBuf>,

        /// Short name to register in curio.workspaces.toml.
        /// If omitted, the KB is not registered as a named workspace.
        #[arg(long)]
        name: Option<String>,

        /// Optional description for the workspace entry.
        #[arg(long)]
        description: Option<String>,
    },

    /// Manage named KB workspaces in curio.workspaces.toml.
    #[command(subcommand)]
    Workspace(WorkspaceCommands),

    /// Show pipeline status: intake/staged/review/published counts and index freshness.
    Status {
        /// Show status for all registered workspaces.
        #[arg(long)]
        all: bool,
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

        /// Optional: override category on resolve (e.g. "account-tree/acme").
        #[arg(long)]
        category: Option<String>,
    },

    /// Publish a staged/ page to published/.
    Publish {
        /// Slug of the staged page to publish.
        slug: String,

        /// Target category path within published/ (e.g. "account-tree/acme").
        /// Defaults to the category stored in the page's frontmatter.
        #[arg(long)]
        category: Option<String>,
    },

    /// Search the wiki registry by keyword, category, or status.
    Search {
        /// Keywords to search for.
        #[arg(long)]
        keywords: Option<String>,

        /// Filter by category (e.g. "account-tree").
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

    /// Prepare or store proposal-only knowledge-sharpening reviews.
    Sharpen {
        /// Emit an agent-facing sharpening manifest.
        #[arg(long)]
        prepare: bool,

        /// Persist an agent-authored proposal JSON file under wiki/.curio/sharpening-proposals/.
        #[arg(long)]
        proposal_file: Option<PathBuf>,

        /// Maximum number of pages to include in the manifest page listing.
        #[arg(long, default_value = "50")]
        limit: u32,
    },

    /// Rebuild all co-located `index.md` files by walking `wiki/**/*.md`.
    Reindex,

    /// Sync wiki/published/ directory structure to match the NORTHSTAR blueprint.
    /// Run this after editing NORTHSTAR to create new tree/subtree dirs and remove empty stale ones.
    Tree,

    /// Push wiki/published/ to Confluence (one-way sync).
    ///
    /// Default (incremental): only pushes new or changed pages. Skips pruning of
    /// deleted pages and skips re-posting pinned comments that are already set.
    ///
    /// Use --all for a full refresh: re-checks every page, prunes stale Confluence
    /// pages, and re-posts pinned reaction comments on all review pages.
    Sync {
        /// Override the Confluence parent page ID from config.
        #[arg(long)]
        parent_page_id: Option<String>,

        /// Show what would be pushed without making Confluence API calls.
        #[arg(long)]
        dry_run: bool,

        /// Full refresh: prune stale Confluence pages and re-post all pinned comments.
        /// Without this flag, sync is incremental (new/changed pages only, no pruning).
        #[arg(long)]
        all: bool,
    },

    /// Read Confluence review signals (labels, reactions, comments) and apply them to the wiki.
    ///
    /// Labels curio:approve / curio:reject / curio:rewrite (or 👍/👎/❓ reactions on the
    /// pinned comment) drive approve/reject/rewrite actions. Free-form comments are captured
    /// in <slug>.feedback.md without triggering an automatic state change.
    Feedback {
        /// Show planned actions without making any changes.
        #[arg(long)]
        dry_run: bool,
    },

    /// Confidence-gated AI heal loop for the knowledge base.
    ///
    /// Phase 1 (--prepare): scan a scope, emit a JSON heal manifest.
    /// Phase 2 (--apply-file): apply a Claude-produced routes file.
    ///
    /// Full two-phase example:
    ///   curio heal --prepare --scope product-tree/alteryx-server --out /tmp/heal.json
    ///   # → Claude reads /tmp/heal.json, uses tools, writes /tmp/heal-routes.json
    ///   curio heal --apply-file /tmp/heal-routes.json
    Heal {
        /// Phase 1: emit heal manifest instead of applying.
        #[arg(long)]
        prepare: bool,

        /// Phase 2: path to the Claude-produced routes JSON file.
        #[arg(long, value_name = "PATH")]
        apply_file: Option<String>,

        /// NORTHSTAR scope path (e.g. product-tree/alteryx-server).
        /// Defaults to full published KB.
        #[arg(long)]
        scope: Option<String>,

        /// Write manifest to file instead of stdout (used with --prepare).
        #[arg(long, value_name = "PATH")]
        out: Option<String>,

        /// Override confidence threshold from settings.yaml for this run.
        #[arg(long, value_name = "0.0-1.0")]
        confidence: Option<f64>,

        /// Auto-approve all actions regardless of confidence (sets threshold to 0.0).
        #[arg(long)]
        auto: bool,
    },

    /// Locally reject a wiki page without requiring a Confluence review signal.
    ///
    /// Accepts a bare slug, a wiki-relative path, or an absolute path.
    /// Deletes the page and all sidecars (.analysis.json, .sync-refs.json,
    /// .feedback.md, .md.proposal.json) and writes a rejection entry to
    /// wiki/_config/log.md.
    ///
    /// Use --force to reject a published page.
    Reject {
        /// Slug, wiki-relative path, or absolute path to the page.
        slug_or_path: String,

        /// Human-readable reason for the rejection (recorded in log.md).
        #[arg(long)]
        reason: Option<String>,

        /// Allow rejecting a published page (default: blocked for safety).
        #[arg(long)]
        force: bool,
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

#[derive(Subcommand, Debug)]
pub enum WorkspaceCommands {
    /// List all registered KB workspaces.
    List,

    /// Register a KB directory as a named workspace.
    Add {
        /// Short name for the workspace (e.g. "acme", "internal").
        #[arg(long)]
        name: String,

        /// Path to the KB store directory.
        #[arg(long)]
        path: PathBuf,

        /// Optional human-readable description.
        #[arg(long)]
        description: Option<String>,
    },

    /// Remove a workspace registration (does not delete the KB directory).
    Remove {
        /// Name of the workspace to remove.
        name: String,
    },
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
