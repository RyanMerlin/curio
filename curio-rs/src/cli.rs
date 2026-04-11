use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "curio", version, about = "A CLI for Curio")]
pub struct Cli {
    #[arg(long, global = true, help = "Path to the configuration file")]
    pub config: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        help = "If true, print what would be done without making changes"
    )]
    pub dry_run: bool,

    #[arg(long, global = true, help = "Confluence space key to operate within")]
    pub space: Option<String>,

    #[arg(
        long,
        global = true,
        help = "Confluence output root folder ID to operate within"
    )]
    pub root_folder_id: Option<String>,

    #[arg(long, global = true, help = "Working directory for ephemeral files")]
    pub workdir: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        help = "Set the logging level (e.g., INFO, DEBUG, TRACE)"
    )]
    pub log_level: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect and synchronize the Curio onboarding state.
    ///
    /// By default, Curio prompts to install the user-level shim and treats
    /// Enter as yes so `curio` works from any terminal.
    Onboard {
        /// Force install or update the user-level Curio shim without prompting.
        #[arg(long)]
        install: bool,
    },

    /// Verify provider and harness prerequisites.
    Doctor {
        #[arg(value_enum)]
        provider: Option<AgentProvider>,
    },

    /// Create and verify the core Confluence structure (folders, templates).
    Bootstrap,

    /// Ingest content from various sources into Confluence.
    IntakeCreate {
        /// URL of a web page or Confluence link to ingest.
        #[arg(long, group = "source", help_heading = "Input Source (choose one)")]
        url: Option<String>,

        /// Path to a local file to ingest.
        #[arg(long, group = "source", help_heading = "Input Source (choose one)")]
        file: Option<std::path::PathBuf>,

        /// Path to a local folder to ingest recursively.
        #[arg(long, group = "source", help_heading = "Input Source (choose one)")]
        folder: Option<std::path::PathBuf>,

        /// (Optional) A hint for the subject of the content.
        #[arg(long, help_heading = "Options")]
        subject_hint: Option<String>,

        /// (Optional) JSON string of initial metadata to merge.
        #[arg(long, help_heading = "Options")]
        metadata: Option<String>,
    },

    /// Processes content from the Intake stage, moving it to Staged or Review.
    ProcessIntake {
        /// The maximum number of intake items to process in one run.
        #[arg(
            long,
            default_value = "10",
            help = "The maximum number of intake items to process in one run."
        )]
        limit: u32,
    },

    /// Searches Confluence for content based on various criteria.
    Search {
        /// Labels to filter by (e.g., "curio-status-staged"). Can be specified multiple times.
        #[arg(long)]
        labels: Vec<String>,

        /// Free-text search query.
        #[arg(long)]
        text: Option<String>,

        /// Content type to filter by (e.g., "page", "blogpost").
        #[arg(long)]
        content_type: Option<String>,

        /// Maximum number of results to return.
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Command for an external agent to analyze content in Confluence.
    AgentAnalyze {
        /// Optional: Process a specific page by its ID.
        #[arg(long)]
        page_id: Option<String>,

        /// Optional: Process pages with a specific status (e.g., "intake", "analyzing").
        #[arg(long)]
        status: Option<String>,

        /// The maximum number of items to analyze in one run.
        #[arg(long, default_value = "10")]
        limit: u32,
    },

    /// Finds or creates a canonical "gold" page for a given subject.
    GoldResolve {
        /// The ID of the page to resolve (e.g., a page from the "Staged" or "Review" areas).
        #[arg(long)]
        page_id: String,
    },

    /// Publishes resolved content to its canonical "gold" page.
    GoldPublish {
        /// The ID of the resolved page (which contains the link to the "gold" page).
        #[arg(long)]
        page_id: String,
    },

    /// A group of commands for reviewing content.
    #[command(subcommand)]
    Review(ReviewCommands),

    /// Harness commands for launching supported agent providers in the Curio repo.
    #[command(subcommand)]
    Agent(AgentCommands),
}

// New enum for the review subcommand
#[derive(Subcommand, Debug)]
pub enum ReviewCommands {
    /// Approves a staged item or a change proposal for publishing.
    Approve {
        /// The ID of the page to approve.
        #[arg(long)]
        page_id: String,
    },
    /// Rejects a staged item or a change proposal.
    Reject {
        /// The ID of the page to reject.
        #[arg(long)]
        page_id: String,
        /// The reason for the rejection.
        #[arg(long)]
        reason: String,
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
