use anyhow::Result;
use clap::{CommandFactory, Parser};
use curio::{
    cli::{AgentCommands, Cli, Commands, WorkspaceCommands},
    commands::{
        agent::{
            run_agent_doctor, run_agent_launch, run_agent_list_plugins, run_agent_list_providers,
            run_agent_list_skills, run_agent_prepare, run_agent_print_env,
        },
        doctor::run_doctor,
        feedback::run_feedback,
        gold_publish::run_publish,
        gold_resolve::run_resolve,
        heal::{run_heal_apply, run_heal_prepare},
        init::run_init,
        init_kb::run_init_kb,
        intake::run_intake,
        lint::run_lint,
        onboard::run_onboard,
        process_intake::run_process,
        query::run_query,
        reindex::run_reindex,
        reject::run_reject,
        retrieve::{run_fetch, run_retrieve},
        review::run_review,
        search::run_search,
        sharpen::run_sharpen,
        status::run_status,
        sync::run_sync,
        tree::run_tree,
        workspace_cmd::{run_workspace_add, run_workspace_list, run_workspace_remove},
    },
    config::load_config,
    workspace::resolve_kb_dir,
};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.log_level.as_deref());
    let json_mode = cli.json;
    let cmd_name = command_name(&cli.command);
    match dispatch(cli).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            if json_mode {
                if let Some(validation) = err.downcast_ref::<curio::error::CliValidationError>() {
                    curio::output::emit_json_error(
                        cmd_name,
                        validation.code,
                        &validation.message,
                        Some(validation.hint),
                    );
                    return std::process::ExitCode::FAILURE;
                }
                // Machine-readable error envelope. Preserve the full anyhow
                // chain via `{:#}` so context isn't lost.
                curio::output::emit_json_error(
                    cmd_name,
                    "internal_error",
                    format!("{:#}", err),
                    None,
                );
            } else {
                eprintln!("Error: {:#}", err);
            }
            std::process::ExitCode::FAILURE
        }
    }
}

/// Stable short identifier for the current subcommand. Used in --json error
/// envelopes so colleagues' scripts can dispatch on it. Defaults to "curio"
/// when no subcommand is given.
fn command_name(cmd: &Option<Commands>) -> &'static str {
    match cmd {
        Some(Commands::Onboard { .. }) => "onboard",
        Some(Commands::Doctor { .. }) => "doctor",
        Some(Commands::Init { .. }) => "init",
        Some(Commands::Intake { .. }) => "intake",
        Some(Commands::Process { .. }) => "process",
        Some(Commands::InitKb { .. }) => "init-kb",
        Some(Commands::Workspace(_)) => "workspace",
        Some(Commands::Status { .. }) => "status",
        Some(Commands::Review { .. }) => "review",
        Some(Commands::Resolve { .. }) => "resolve",
        Some(Commands::Publish { .. }) => "publish",
        Some(Commands::Search { .. }) => "search",
        Some(Commands::Retrieve { .. }) => "retrieve",
        Some(Commands::Fetch { .. }) => "fetch",
        Some(Commands::Sharpen { .. }) => "sharpen",
        Some(Commands::Reindex) => "reindex",
        Some(Commands::Tree) => "tree",
        Some(Commands::Sync { .. }) => "sync",
        Some(Commands::Feedback { .. }) => "feedback",
        Some(Commands::Heal { .. }) => "heal",
        Some(Commands::Reject { .. }) => "reject",
        Some(Commands::Lint { .. }) => "lint",
        Some(Commands::Query { .. }) => "query",
        Some(Commands::Agent(_)) => "agent",
        Some(Commands::Slack(_)) => "slack",
        None => "curio",
    }
}

async fn dispatch(cli: Cli) -> Result<()> {
    let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());

    // Resolve KB directory: --kb-dir > --workspace > error (for commands that need a KB)
    // Commands that don't need a KB (agent, doctor, init-kb, workspace) skip this.
    let kb_dir_resolved = resolve_kb_dir(cli.kb_dir.as_ref(), cli.workspace.as_deref()).ok();

    match cli.command {
        Some(Commands::Onboard { install }) => {
            run_onboard(cli.dry_run, install).await?;
        }
        Some(Commands::Doctor { scope, emit_review }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_doctor(&config, cli.dry_run, cli.json, scope, emit_review).await?;
        }
        Some(Commands::Agent(agent_commands)) => match agent_commands {
            AgentCommands::Prepare { provider } => {
                run_agent_prepare(provider, cli.json)?;
            }
            AgentCommands::Launch { provider } => {
                run_agent_launch(provider, cli.dry_run)?;
            }
            AgentCommands::Doctor { provider } => {
                run_agent_doctor(provider, cli.json)?;
            }
            AgentCommands::ListProviders => {
                run_agent_list_providers(cli.json)?;
            }
            AgentCommands::ListSkills => {
                run_agent_list_skills(cli.json)?;
            }
            AgentCommands::ListPlugins => {
                run_agent_list_plugins(cli.json)?;
            }
            AgentCommands::PrintEnv { provider } => {
                run_agent_print_env(provider, cli.json)?;
            }
        },
        Some(Commands::Slack(slack_commands)) => match slack_commands {
            curio::cli::SlackCommands::Process { payload_file } => {
                let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
                curio::commands::slack::run_slack_process(&config, payload_file, cli.json).await?;
            }
            curio::cli::SlackCommands::Authorize {
                user_id,
                channel_id,
            } => {
                let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
                curio::commands::slack::run_slack_authorize(
                    &config, user_id, channel_id, cli.json,
                )?;
            }
            curio::cli::SlackCommands::Contract => {
                let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
                curio::commands::slack::run_slack_contract(&config, cli.json)?;
            }
        },

        // ── Workspace management (no KB required) ──────────────────────────
        Some(Commands::Workspace(ws_cmd)) => match ws_cmd {
            WorkspaceCommands::List => {
                run_workspace_list(cli.json)?;
            }
            WorkspaceCommands::Add {
                name,
                path,
                description,
            } => {
                run_workspace_add(name, path, description, cli.json)?;
            }
            WorkspaceCommands::Remove { name } => {
                run_workspace_remove(name, cli.json)?;
            }
        },

        // ── KB initialisation (no existing KB required) ────────────────────
        Some(Commands::InitKb {
            path,
            name,
            description,
        }) => {
            run_init_kb(path, name, description, cli.dry_run).await?;
        }

        // ── All commands below require a KB dir ────────────────────────────
        Some(Commands::Init {
            reset,
            confirm_nuke,
        }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_init(&config, cli.dry_run, cli.json, reset, confirm_nuke).await?;
        }
        Some(Commands::Intake {
            url,
            file,
            folder,
            title,
            subject_hint,
            recursive,
        }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_intake(
                &config,
                cli.dry_run,
                cli.json,
                &url,
                &file,
                &folder,
                &title,
                &subject_hint,
                recursive,
            )
            .await?;
        }
        Some(Commands::Process {
            limit,
            all,
            prepare,
            route_file,
            slug,
            category,
            status,
            keywords,
            confidence,
            summary,
        }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_process(
                &config,
                cli.dry_run,
                cli.json,
                limit,
                all,
                prepare,
                route_file,
                slug,
                category,
                status,
                keywords,
                confidence,
                summary,
            )
            .await?;
        }
        Some(Commands::Status { all }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_status(&config, cli.json, all).await?;
        }
        Some(Commands::Review { lane }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_review(&config, cli.dry_run, cli.json, &lane).await?;
        }
        Some(Commands::Resolve { slug, category }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_resolve(&config, cli.dry_run, cli.json, slug, category).await?;
        }
        Some(Commands::Publish {
            slug,
            category,
            force,
        }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_publish(&config, cli.dry_run, cli.json, slug, category, force).await?;
        }
        Some(Commands::Search {
            keywords,
            category,
            status,
            text,
            limit,
        }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_search(
                &config,
                cli.dry_run,
                cli.json,
                keywords,
                category,
                status,
                text,
                limit,
            )
            .await?;
        }
        Some(Commands::Sharpen {
            prepare,
            proposal_file,
            limit,
        }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_sharpen(
                &config,
                cli.dry_run,
                cli.json,
                prepare,
                proposal_file,
                limit,
            )
            .await?;
        }
        Some(Commands::Reindex) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_reindex(&config, cli.dry_run, cli.json).await?;
        }
        Some(Commands::Tree) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_tree(&config, cli.dry_run, cli.json).await?;
        }
        Some(Commands::Sync {
            parent_page_id,
            dry_run,
            all,
            docs_only,
            downloads_only,
        }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_sync(
                &config,
                dry_run || cli.dry_run,
                cli.json,
                parent_page_id,
                all,
                docs_only,
                downloads_only,
            )
            .await?;
        }
        Some(Commands::Feedback { dry_run }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_feedback(&config, dry_run || cli.dry_run).await?;
        }
        Some(Commands::Heal {
            prepare,
            apply_file,
            scope,
            out,
            confidence,
            auto,
        }) => {
            let mut config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            // Override threshold if --confidence or --auto flags are set.
            if auto {
                config.heal.confidence_threshold = Some(0.0);
            } else if let Some(c) = confidence {
                config.heal.confidence_threshold = Some(c);
            }
            if prepare {
                run_heal_prepare(&config, scope, out).await?;
            } else if let Some(ref path) = apply_file {
                run_heal_apply(&config, cli.dry_run, path, scope).await?;
            } else {
                eprintln!("curio heal: specify --prepare or --apply-file <path>");
                std::process::exit(1);
            }
        }
        Some(Commands::Reject {
            slug_or_path,
            reason,
            force,
        }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_reject(&config, cli.dry_run, slug_or_path, reason, force).await?;
        }
        Some(Commands::Lint { fix }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_lint(&config, cli.dry_run, cli.json, fix).await?;
        }
        Some(Commands::Query { question, save }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_query(&config, cli.dry_run, cli.json, question, save).await?;
        }
        Some(Commands::Retrieve {
            query,
            category,
            limit,
        }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_retrieve(&config, cli.dry_run, cli.json, query, category, limit).await?;
        }
        Some(Commands::Fetch { id }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_fetch(&config, cli.dry_run, cli.json, id).await?;
        }
        None => {
            Cli::command().print_help()?;
            println!();
        }
    }

    Ok(())
}

/// Initialize tracing once for the CLI binary. Resolution order:
///   1. --log-level <LEVEL> CLI flag
///   2. RUST_LOG env var (handled by EnvFilter::from_default_env)
///   3. fall back to "warn" (CLI is not chatty by default)
fn init_tracing(cli_log_level: Option<&str>) {
    use tracing_subscriber::EnvFilter;

    let filter = if let Some(level) = cli_log_level.filter(|s| !s.trim().is_empty()) {
        EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("warn"))
    } else if std::env::var_os("RUST_LOG").is_some() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::new("warn")
    };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
