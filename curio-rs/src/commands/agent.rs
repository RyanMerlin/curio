use crate::cli::AgentProvider;
use crate::harness::{
    HarnessPaths, build_launch_plan, discover_skills, load_marketplace, run_checks,
};
use crate::output::emit_json;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::collections::BTreeMap;
use std::process::{Command, Stdio};

#[derive(Debug, Serialize)]
struct PreparedProviderOutput {
    provider: String,
    repo_root: String,
    entrypoint: String,
    profile: String,
    bootstrap_summary: String,
    command: String,
    skills_count: usize,
    enabled_plugins_count: usize,
}

#[derive(Debug, Serialize)]
struct DoctorOutput {
    provider: Option<String>,
    ok: bool,
    failed: usize,
    checks: Vec<crate::harness::CheckResult>,
}

#[derive(Debug, Serialize)]
struct ProviderAvailabilityOutput {
    provider: String,
    available: bool,
    command: Option<String>,
    profile: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProvidersOutput {
    providers: Vec<ProviderAvailabilityOutput>,
}

#[derive(Debug, Serialize)]
struct SkillsOutput {
    skills: Vec<crate::harness::SkillInfo>,
}

#[derive(Debug, Serialize)]
struct PluginsOutput {
    plugins: Vec<crate::harness::MarketplacePlugin>,
}

#[derive(Debug, Serialize)]
struct EnvOutput {
    provider: String,
    env: BTreeMap<String, String>,
}

pub fn run_agent_prepare(provider: AgentProvider, json_output: bool) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let plan = build_launch_plan(provider, &paths)?;
    let skills = discover_skills(&paths)?;
    let marketplace = load_marketplace(&paths)?;

    if json_output {
        return emit_json(
            "agent.prepare",
            true,
            PreparedProviderOutput {
                provider: provider.as_str().to_string(),
                repo_root: plan.repo_root.display().to_string(),
                entrypoint: plan.entrypoint_path.display().to_string(),
                profile: plan.profile_path.display().to_string(),
                bootstrap_summary: plan.bootstrap_summary.clone(),
                command: plan.command_line(),
                skills_count: skills.len(),
                enabled_plugins_count: marketplace
                    .plugins
                    .iter()
                    .filter(|plugin| plugin.enabled)
                    .count(),
            },
        );
    }

    println!("provider: {}", provider.as_str());
    println!("repo_root: {}", plan.repo_root.display());
    println!("entrypoint: {}", plan.entrypoint_path.display());
    println!("profile: {}", plan.profile_path.display());
    println!("bootstrap: {}", plan.bootstrap_summary);
    println!("command: {}", plan.command_line());
    println!("skills: {}", skills.len());
    println!(
        "plugins: {}",
        marketplace
            .plugins
            .iter()
            .filter(|plugin| plugin.enabled)
            .count()
    );

    Ok(())
}

pub fn run_agent_launch(provider: AgentProvider, dry_run: bool) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let plan = build_launch_plan(provider, &paths)?;

    if dry_run {
        println!(
            "(Dry run) Would launch {} in {}",
            provider.as_str(),
            plan.repo_root.display()
        );
        println!("(Dry run) Profile: {}", plan.profile_path.display());
        println!("(Dry run) Bootstrap: {}", plan.bootstrap_summary);
        println!("(Dry run) Command: {}", plan.command_line());
        return Ok(());
    }

    let status = Command::new(&plan.command)
        .args(&plan.args)
        .current_dir(&plan.repo_root)
        .envs(&plan.env)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| {
            format!(
                "Failed to launch {} via {}",
                provider.as_str(),
                plan.command
            )
        })?;

    if !status.success() {
        bail!("{} exited with status {}", provider.as_str(), status);
    }

    Ok(())
}

pub fn run_agent_doctor(provider: Option<AgentProvider>, json_output: bool) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let checks = run_checks(&paths, provider)?;
    let mut failed = 0usize;

    if json_output {
        for check in &checks {
            if !check.ok {
                failed += 1;
            }
        }
        emit_json(
            "agent.doctor",
            failed == 0,
            DoctorOutput {
                provider: provider.map(|value| value.as_str().to_string()),
                ok: failed == 0,
                failed,
                checks,
            },
        )?;
        if failed > 0 {
            bail!("Doctor found {} failing check(s)", failed);
        }
        return Ok(());
    }

    for check in &checks {
        let status = if check.ok { "OK" } else { "FAIL" };
        println!("[{}] {} :: {}", status, check.label, check.detail);
        if !check.ok {
            failed += 1;
        }
    }

    if failed > 0 {
        bail!("Doctor found {} failing check(s)", failed);
    }

    println!("All Curio harness checks passed.");
    Ok(())
}

pub fn run_agent_list_providers(json_output: bool) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let mut providers = Vec::new();

    for provider in [
        AgentProvider::Codex,
        AgentProvider::Claude,
        AgentProvider::Gemini,
    ] {
        let plan = build_launch_plan(provider, &paths);
        match plan {
            Ok(plan) => {
                if json_output {
                    providers.push(ProviderAvailabilityOutput {
                        provider: provider.as_str().to_string(),
                        available: true,
                        command: Some(plan.command_line()),
                        profile: Some(plan.profile_path.display().to_string()),
                        error: None,
                    });
                } else {
                    println!(
                        "{} :: available :: {} :: {}",
                        provider.as_str(),
                        plan.command_line(),
                        plan.profile_path.display()
                    );
                }
            }
            Err(err) => {
                if json_output {
                    providers.push(ProviderAvailabilityOutput {
                        provider: provider.as_str().to_string(),
                        available: false,
                        command: None,
                        profile: None,
                        error: Some(err.to_string()),
                    });
                } else {
                    println!("{} :: unavailable :: {}", provider.as_str(), err);
                }
            }
        }
    }

    if json_output {
        emit_json("agent.list-providers", true, ProvidersOutput { providers })?;
    }

    Ok(())
}

pub fn run_agent_list_skills(json_output: bool) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let skills = discover_skills(&paths)?;

    if json_output {
        emit_json("agent.list-skills", true, SkillsOutput { skills })?;
    } else {
        for skill in skills {
            println!("{} :: {}", skill.name, skill.path.display());
        }
    }

    Ok(())
}

pub fn run_agent_list_plugins(json_output: bool) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let marketplace = load_marketplace(&paths)?;

    if json_output {
        emit_json(
            "agent.list-plugins",
            true,
            PluginsOutput {
                plugins: marketplace.plugins,
            },
        )?;
    } else {
        for plugin in marketplace.plugins {
            println!(
                "{} :: enabled={} :: {} :: {}",
                plugin.name, plugin.enabled, plugin.path, plugin.description
            );
        }
    }

    Ok(())
}

pub fn run_agent_print_env(provider: AgentProvider, json_output: bool) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let plan = build_launch_plan(provider, &paths)?;

    if json_output {
        emit_json(
            "agent.print-env",
            true,
            EnvOutput {
                provider: provider.as_str().to_string(),
                env: plan.env,
            },
        )?;
    } else {
        for (key, value) in plan.env {
            println!("{}={}", key, value);
        }
    }

    Ok(())
}
