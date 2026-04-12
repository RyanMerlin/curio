use anyhow::Result;
use clap::{CommandFactory, Parser};
use curio::{
    cli::{AgentCommands, Cli, Commands},
    commands::{
        agent::{
            run_agent_doctor, run_agent_launch, run_agent_list_plugins, run_agent_list_providers,
            run_agent_list_skills, run_agent_prepare, run_agent_print_env,
        },
        gold_publish::run_publish,
        gold_resolve::run_resolve,
        init::run_init,
        intake::run_intake,
        lint::run_lint,
        onboard::run_onboard,
        process_intake::run_process,
        query::run_query,
        reindex::run_reindex,
        review::run_review,
        search::run_search,
        sync::run_sync,
        tree::run_tree,
    },
    config::load_config,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path_str = cli.config.as_ref().and_then(|p| p.to_str());

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
        Some(Commands::Init { reset }) => {
            let config = load_config(config_path_str)?;
            run_init(&config, cli.dry_run, cli.json, reset).await?;
        }
        Some(Commands::Intake { url, file, folder, title, subject_hint, recursive }) => {
            let config = load_config(config_path_str)?;
            run_intake(&config, cli.dry_run, cli.json, &url, &file, &folder, &title, &subject_hint, recursive).await?;
        }
        Some(Commands::Process {
            limit,
            auto,
            route_file,
            slug,
            category,
            status,
            keywords,
            confidence,
            summary,
        }) => {
            let config = load_config(config_path_str)?;
            run_process(
                &config, cli.dry_run, cli.json,
                limit, auto, route_file,
                slug, category, status, keywords, confidence, summary,
            ).await?;
        }
        Some(Commands::Review { lane }) => {
            let config = load_config(config_path_str)?;
            run_review(&config, cli.dry_run, cli.json, &lane).await?;
        }
        Some(Commands::Resolve { slug, category }) => {
            let config = load_config(config_path_str)?;
            run_resolve(&config, cli.dry_run, cli.json, slug, category).await?;
        }
        Some(Commands::Publish { slug, category }) => {
            let config = load_config(config_path_str)?;
            run_publish(&config, cli.dry_run, cli.json, slug, category).await?;
        }
        Some(Commands::Search { keywords, category, status, text, limit }) => {
            let config = load_config(config_path_str)?;
            run_search(&config, cli.dry_run, cli.json, keywords, category, status, text, limit).await?;
        }
        Some(Commands::Reindex) => {
            let config = load_config(config_path_str)?;
            run_reindex(&config, cli.dry_run, cli.json).await?;
        }
        Some(Commands::Tree) => {
            let config = load_config(config_path_str)?;
            run_tree(&config, cli.dry_run, cli.json).await?;
        }
        Some(Commands::Sync { parent_page_id, dry_run }) => {
            let config = load_config(config_path_str)?;
            run_sync(&config, dry_run || cli.dry_run, cli.json, parent_page_id).await?;
        }
        Some(Commands::Lint { fix }) => {
            let config = load_config(config_path_str)?;
            run_lint(&config, cli.dry_run, cli.json, fix).await?;
        }
        Some(Commands::Query { question, save }) => {
            let config = load_config(config_path_str)?;
            run_query(&config, cli.dry_run, cli.json, question, save).await?;
        }
        None => {
            Cli::command().print_help()?;
            println!();
        }
    }

    Ok(())
}
