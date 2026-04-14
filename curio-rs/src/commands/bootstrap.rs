/// Forwarding shim: the old `bootstrap` command now delegates to `init`.
use anyhow::Result;
use crate::config::Config;

pub async fn run_bootstrap(config: &Config, dry_run: bool, json: bool, _overwrite: bool, _confirm_nuke: bool) -> Result<()> {
    super::init::run_init(config, dry_run, json, _overwrite, _confirm_nuke).await
}
