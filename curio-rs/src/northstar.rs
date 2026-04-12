use anyhow::{Context, Result, bail};
use pulldown_cmark::{Options, Parser, html};
use std::fs;
use std::path::Path;

pub const NORTHSTAR_FILENAME: &str = "NORTHSTAR.md";

pub fn northstar_path(repo_root: &Path) -> std::path::PathBuf {
    repo_root.join(NORTHSTAR_FILENAME)
}

pub fn default_northstar_markdown() -> String {
    include_str!("../../NORTHSTAR.md").to_string()
}

pub fn ensure_northstar_markdown(repo_root: &Path, dry_run: bool) -> Result<()> {
    let path = northstar_path(repo_root);
    if path.is_file() {
        return Ok(());
    }

    if dry_run {
        println!("[WARN] northstar_doc :: missing {}", path.display());
        return Ok(());
    }

    fs::write(&path, default_northstar_markdown())
        .with_context(|| format!("Failed to write {}", path.display()))?;
    println!("[OK] northstar_doc :: created {}", path.display());
    Ok(())
}

pub fn read_northstar_markdown(repo_root: &Path) -> Result<String> {
    let path = northstar_path(repo_root);
    if !path.is_file() {
        bail!(
            "Missing NORTHSTAR.md at {}. Run `curio onboard` to create the charter file.",
            path.display()
        );
    }

    fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
}

pub fn render_northstar_markdown(markdown: &str) -> String {
    let mut html_output = String::new();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, options);
    html::push_html(&mut html_output, parser);
    html_output
}
