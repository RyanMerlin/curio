use anyhow::Result;
use clap::Parser;
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
        review::{run_review_approve, run_review_reject}, // Import review functions
        search::run_search,
    },
    config::load_config,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Onboard => {
            run_onboard(cli.dry_run).await?;
        }
        Commands::Doctor { provider } => {
            run_agent_doctor(provider)?;
        }
        Commands::Agent(agent_commands) => match agent_commands {
            AgentCommands::Prepare { provider } => {
                run_agent_prepare(provider)?;
            }
            AgentCommands::Launch { provider } => {
                run_agent_launch(provider, cli.dry_run)?;
            }
            AgentCommands::Doctor { provider } => {
                run_agent_doctor(provider)?;
            }
            AgentCommands::ListProviders => {
                run_agent_list_providers()?;
            }
            AgentCommands::ListSkills => {
                run_agent_list_skills()?;
            }
            AgentCommands::ListPlugins => {
                run_agent_list_plugins()?;
            }
            AgentCommands::PrintEnv { provider } => {
                run_agent_print_env(provider)?;
            }
        },
        Commands::Bootstrap => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_bootstrap(&config, cli.dry_run).await?;
        }
        Commands::IntakeCreate {
            url,
            file,
            folder,
            subject_hint,
            metadata,
        } => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_intake_create(
                &config,
                cli.dry_run,
                &url,
                &file,
                &folder,
                &subject_hint,
                &metadata,
            )
            .await?;
        }
        Commands::ProcessIntake { limit } => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_process_intake(&config, cli.dry_run, limit).await?;
        }
        Commands::Search {
            labels,
            text,
            content_type,
            limit,
        } => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_search(&config, cli.dry_run, labels, text, content_type, limit).await?;
        }
        Commands::AgentAnalyze {
            page_id,
            status,
            limit,
        } => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_agent_analyze(&config, cli.dry_run, &page_id, &status, limit).await?;
        }
        Commands::GoldResolve { page_id } => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_gold_resolve(&config, cli.dry_run, page_id).await?;
        }
        Commands::GoldPublish { page_id } => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            run_gold_publish(&config, cli.dry_run, page_id).await?;
        }
        Commands::Review(review_commands) => {
            let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());
            let config = load_config(config_path_str)?;
            match review_commands {
                ReviewCommands::Approve { page_id } => {
                    run_review_approve(&config, cli.dry_run, page_id).await?;
                }
                ReviewCommands::Reject { page_id, reason } => {
                    run_review_reject(&config, cli.dry_run, page_id, reason).await?;
                }
            }
        }
    }

    Ok(())
}
