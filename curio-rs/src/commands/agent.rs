use crate::cli::AgentProvider;
use crate::harness::{
    HarnessPaths, build_launch_plan, discover_skills, load_marketplace, run_checks,
};
use anyhow::{Context, Result, bail};
use std::process::{Command, Stdio};

pub fn run_agent_prepare(provider: AgentProvider) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let plan = build_launch_plan(provider, &paths)?;
    let skills = discover_skills(&paths)?;
    let marketplace = load_marketplace(&paths)?;

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

pub fn run_agent_doctor(provider: Option<AgentProvider>) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let checks = run_checks(&paths, provider)?;
    let mut failed = 0usize;

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

pub fn run_agent_list_providers() -> Result<()> {
    let paths = HarnessPaths::discover()?;

    for provider in [
        AgentProvider::Codex,
        AgentProvider::Claude,
        AgentProvider::Gemini,
    ] {
        let plan = build_launch_plan(provider, &paths);
        match plan {
            Ok(plan) => println!(
                "{} :: available :: {} :: {}",
                provider.as_str(),
                plan.command_line(),
                plan.profile_path.display()
            ),
            Err(err) => println!("{} :: unavailable :: {}", provider.as_str(), err),
        }
    }

    Ok(())
}

pub fn run_agent_list_skills() -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let skills = discover_skills(&paths)?;

    for skill in skills {
        println!("{} :: {}", skill.name, skill.path.display());
    }

    Ok(())
}

pub fn run_agent_list_plugins() -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let marketplace = load_marketplace(&paths)?;

    for plugin in marketplace.plugins {
        println!(
            "{} :: enabled={} :: {} :: {}",
            plugin.name, plugin.enabled, plugin.path, plugin.description
        );
    }

    Ok(())
}

pub fn run_agent_print_env(provider: AgentProvider) -> Result<()> {
    let paths = HarnessPaths::discover()?;
    let plan = build_launch_plan(provider, &paths)?;

    for (key, value) in plan.env {
        println!("{}={}", key, value);
    }

    Ok(())
}
