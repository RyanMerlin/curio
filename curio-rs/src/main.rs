use anyhow::Result;
use clap::{CommandFactory, Parser};
use curio::{
    cli::{AgentCommands, Cli, Commands, ReviewCommands},
    commands::{
        agent::{
            run_agent_doctor, run_agent_launch, run_agent_list_plugins, run_agent_list_providers,
            run_agent_list_skills, run_agent_prepare, run_agent_print_env,
        },
        agent_analyze::run_agent_analyze,
        bootstrap::run_bootstrap,
        gold_publish::run_gold_publish,
        gold_resolve::run_gold_resolve,
        intake::run_intake_create,
        onboard::run_onboard,
        process_intake::run_process_intake,
        reindex::run_reindex,
        review::{run_review_approve, run_review_reject}, // Import review functions
        search::run_search,
    },
    config::load_config,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

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
        Some(Commands::Bootstrap {
            overwrite,
            confirm_nuke,
        }) => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_bootstrap(&config, cli.dry_run, cli.json, overwrite, confirm_nuke).await?;
        }
        Some(Commands::IntakeCreate {
            url,
            file,
            folder,
            subject_hint,
            metadata,
        }) => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_intake_create(
                &config,
                cli.dry_run,
                cli.json,
                &url,
                &file,
                &folder,
                &subject_hint,
                &metadata,
            )
            .await?;
        }
        Some(Commands::ProcessIntake { limit }) => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_process_intake(&config, cli.dry_run, cli.json, limit).await?;
        }
        Some(Commands::Search {
            labels,
            text,
            content_type,
            limit,
        }) => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_search(
                &config,
                cli.dry_run,
                cli.json,
                labels,
                text,
                content_type,
                limit,
            )
            .await?;
        }
        Some(Commands::AgentAnalyze {
            page_id,
            status,
            limit,
        }) => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_agent_analyze(&config, cli.dry_run, cli.json, &page_id, &status, limit).await?;
        }
        Some(Commands::GoldResolve { page_id }) => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_gold_resolve(&config, cli.dry_run, cli.json, page_id).await?;
        }
        Some(Commands::GoldPublish { page_id }) => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_gold_publish(&config, cli.dry_run, cli.json, page_id).await?;
        }
        Some(Commands::Reindex) => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_reindex(&config, cli.dry_run, cli.json).await?;
        }
        Some(Commands::Review(review_commands)) => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            match review_commands {
                ReviewCommands::Approve { page_id } => {
                    run_review_approve(&config, cli.dry_run, cli.json, page_id).await?;
                }
                ReviewCommands::Reject { page_id, reason } => {
                    run_review_reject(&config, cli.dry_run, cli.json, page_id, reason).await?;
                }
            }
        }
        None => {
            Cli::command().print_help()?;
            println!();
        }
    }

    Ok(())
}
