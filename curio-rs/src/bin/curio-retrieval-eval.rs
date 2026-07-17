use anyhow::Result;
use clap::Parser;
use curio::retrieval_eval::{evaluate, load_corpus};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "curio-retrieval-eval",
    about = "Evaluate deterministic published-page retrieval"
)]
struct Args {
    /// JSON corpus file. Its sibling `wiki/` directory is evaluated.
    #[arg(long)]
    corpus: PathBuf,

    /// Emit the complete structured report as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct JsonEnvelope<T> {
    command: &'static str,
    ok: bool,
    data: T,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let corpus = load_corpus(&args.corpus)?;
    let wiki_dir = args
        .corpus
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("wiki");
    let report = evaluate(&corpus, &wiki_dir)?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&JsonEnvelope {
                command: "retrieval-eval",
                ok: true,
                data: report,
            })?
        );
    } else {
        println!("Retrieval evaluation ({} queries)", report.query_count);
        println!("  recall@k:            {:.3}", report.metrics.recall_at_k);
        println!(
            "  mean reciprocal rank: {:.3}",
            report.metrics.mean_reciprocal_rank
        );
        println!(
            "  citation coverage:   {:.3}",
            report.metrics.citation_coverage
        );
        println!(
            "  stale result rate:    {:.3}",
            report.metrics.stale_result_rate
        );
        println!("  ACL leaks:            {}", report.metrics.acl_leak_count);
    }
    Ok(())
}
