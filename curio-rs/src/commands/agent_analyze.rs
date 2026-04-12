/// agent-analyze has been superseded by `curio process`.
/// This stub exists so existing references compile.
use anyhow::Result;
use crate::config::Config;

pub async fn run_agent_analyze(
    _config: &Config,
    _dry_run: bool,
    _json: bool,
) -> Result<()> {
    anyhow::bail!(
        "`agent-analyze` has been removed. Use `curio process` instead."
    )
}
