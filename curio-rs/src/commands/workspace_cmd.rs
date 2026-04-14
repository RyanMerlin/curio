/// `curio workspace` — manage named KB workspaces.
use anyhow::Result;

use crate::workspace::{load_workspaces, remove_workspace, upsert_workspace};
use std::path::PathBuf;

pub fn run_workspace_list(json: bool) -> Result<()> {
    let workspaces = load_workspaces()?;
    if workspaces.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("No workspaces registered.");
            println!("\nTo add one:");
            println!("  curio workspace add --name <name> --path <path>");
            println!("\nTo create a new KB:");
            println!("  curio init-kb --name <name> --path <path>");
        }
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&workspaces)?);
        return Ok(());
    }

    let name_w = workspaces.iter().map(|w| w.name.len()).max().unwrap_or(4).max(4);
    println!("{:<name_w$}  {}", "NAME", "PATH");
    println!("{}", "-".repeat(name_w + 2 + 60));
    for ws in &workspaces {
        let resolved = ws.resolved_path();
        let exists = if resolved.exists() { "" } else { "  [missing]" };
        println!("{:<name_w$}  {}{}", ws.name, ws.path, exists);
        if let Some(desc) = &ws.description {
            println!("{:<name_w$}  {}", "", desc);
        }
    }
    Ok(())
}

pub fn run_workspace_add(name: String, path: PathBuf, description: Option<String>, json: bool) -> Result<()> {
    let abs_path = if path.is_absolute() {
        path.clone()
    } else {
        std::env::current_dir()?.join(&path)
    };

    if !abs_path.exists() {
        eprintln!("Warning: path does not exist: {}", abs_path.display());
        eprintln!("  The workspace will be registered but may not work until the KB is created.");
        eprintln!("  Run `curio init-kb --name {} --path {}` to create it.", name, abs_path.display());
    }

    let ws_file = upsert_workspace(&name, &abs_path, description.as_deref())?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "status": "ok",
                "name": name,
                "path": abs_path.to_string_lossy(),
                "workspace_file": ws_file.to_string_lossy(),
            })
        );
    } else {
        println!("Workspace '{}' registered → {}", name, abs_path.display());
        println!("Saved to: {}", ws_file.display());
        println!("\nUsage:");
        println!("  curio --workspace {} status", name);
        println!("  curio --workspace {} sync", name);
    }
    Ok(())
}

pub fn run_workspace_remove(name: String, json: bool) -> Result<()> {
    let removed = remove_workspace(&name)?;
    if json {
        println!("{}", serde_json::json!({ "removed": removed, "name": name }));
    } else if removed {
        println!("Workspace '{}' removed.", name);
        println!("(The KB directory itself was not deleted.)");
    } else {
        println!("Workspace '{}' not found — nothing changed.", name);
    }
    Ok(())
}
