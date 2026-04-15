use anyhow::Result;
use clap::{CommandFactory, Parser};
use curio::{
    cli::{AgentCommands, Cli, Commands, WorkspaceCommands},
    commands::{
        agent::{
            run_agent_doctor, run_agent_launch, run_agent_list_plugins, run_agent_list_providers,
            run_agent_list_skills, run_agent_prepare, run_agent_print_env,
        },
        gold_publish::run_publish,
        gold_resolve::run_resolve,
        init::run_init,
        init_kb::run_init_kb,
        intake::run_intake,
        lint::run_lint,
        onboard::run_onboard,
        process_intake::run_process,
        query::run_query,
        reindex::run_reindex,
        review::run_review,
        search::run_search,
        sharpen::run_sharpen,
        status::run_status,
        sync::run_sync,
        feedback::run_feedback,
        reject::run_reject,
        tree::run_tree,
        workspace_cmd::{run_workspace_add, run_workspace_list, run_workspace_remove},
    },
    config::load_config,
    workspace::resolve_kb_dir,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());

    // Resolve KB directory: --kb-dir > --workspace > error (for commands that need a KB)
    // Commands that don't need a KB (agent, doctor, init-kb, workspace) skip this.
    let kb_dir_resolved = resolve_kb_dir(cli.kb_dir.as_ref(), cli.workspace.as_deref()).ok();

    match cli.command {
        Some(Commands::Onboard { install }) => {
            run_onboard(cli.dry_run, install).await?;
        }
        Some(Commands::Doctor { provider }) => {
            run_agent_doctor(provider, cli.json)?;
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

        // ── Workspace management (no KB required) ──────────────────────────
        Some(Commands::Workspace(ws_cmd)) => match ws_cmd {
            WorkspaceCommands::List => {
                run_workspace_list(cli.json)?;
            }
            WorkspaceCommands::Add { name, path, description } => {
                run_workspace_add(name, path, description, cli.json)?;
            }
            WorkspaceCommands::Remove { name } => {
                run_workspace_remove(name, cli.json)?;
            }
        },

        // ── KB initialisation (no existing KB required) ────────────────────
        Some(Commands::InitKb { path, name, description }) => {
            run_init_kb(path, name, description, cli.dry_run).await?;
        }

        // ── All commands below require a KB dir ────────────────────────────
        Some(Commands::Init { reset, confirm_nuke }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_init(&config, cli.dry_run, cli.json, reset, confirm_nuke).await?;
        }
        Some(Commands::Intake { url, file, folder, title, subject_hint, recursive }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_intake(&config, cli.dry_run, cli.json, &url, &file, &folder, &title, &subject_hint, recursive).await?;
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
                &config, cli.dry_run, cli.json,
                limit, all, prepare, route_file,
                slug, category, status, keywords, confidence, summary,
            ).await?;
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
        Some(Commands::Publish { slug, category }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_publish(&config, cli.dry_run, cli.json, slug, category).await?;
        }
        Some(Commands::Search { keywords, category, status, text, limit }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_search(&config, cli.dry_run, cli.json, keywords, category, status, text, limit).await?;
        }
        Some(Commands::Sharpen { prepare, proposal_file, limit }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_sharpen(&config, cli.dry_run, cli.json, prepare, proposal_file, limit).await?;
        }
        Some(Commands::Reindex) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_reindex(&config, cli.dry_run, cli.json).await?;
        }
        Some(Commands::Tree) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_tree(&config, cli.dry_run, cli.json).await?;
        }
        Some(Commands::Sync { parent_page_id, dry_run, all }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_sync(&config, dry_run || cli.dry_run, cli.json, parent_page_id, all).await?;
        }
        Some(Commands::Feedback { dry_run }) => {
            let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
            run_feedback(&config, dry_run || cli.dry_run).await?;
        }
        Some(Commands::Reject { slug_or_path, reason, force }) => {
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
        None => {
            Cli::command().print_help()?;
            println!();
        }
    }

    Ok(())
}
