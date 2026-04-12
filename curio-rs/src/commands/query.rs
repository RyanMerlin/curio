/// Query command: answer a natural-language question by querying the wiki.
///
/// This command outputs the question context (index + top matching pages) in a
/// format suitable for the LLM agent harness to synthesise an answer.
/// When run interactively the agent reads this output and provides the answer.
use anyhow::Result;

use crate::{
    config::Config,
    output::emit_json,
    wiki_index::{append_log, load_registry, read_index_md},
};

pub async fn run_query(
    config: &Config,
    _dry_run: bool,
    json: bool,
    question: String,
    save: bool,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let registry = load_registry(wiki_dir)?;
    let index_md = read_index_md(wiki_dir)?;

    let q_lower = question.to_lowercase();

    // Find most relevant pages by scoring keyword overlap with the question
    let mut scored: Vec<(f32, &crate::WikiIndexEntry)> = registry
        .pages
        .iter()
        .map(|e| {
            let haystack = format!(
                "{} {} {}",
                e.title.to_lowercase(),
                e.keywords.join(" ").to_lowercase(),
                e.summary.to_lowercase()
            );
            let words: Vec<&str> = q_lower.split_whitespace().collect();
            let hits = words.iter().filter(|&&w| w.len() >= 3 && haystack.contains(w)).count();
            let score = hits as f32 / words.len().max(1) as f32;
            (score, e)
        })
        .filter(|(s, _)| *s > 0.0)
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let top_entries: Vec<_> = scored.iter().take(5).map(|(_, e)| e).collect();

    // Read full content of top matching pages
    let mut page_contexts = Vec::new();
    for entry in &top_entries {
        let path = wiki_dir.join(&entry.path);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let body = crate::wiki_fs::strip_frontmatter(&content).to_string();
            page_contexts.push(serde_json::json!({
                "path": entry.path,
                "title": entry.title,
                "body": body,
            }));
        }
    }

    append_log(wiki_dir, &format!("query: \"{}\"", &question[..question.len().min(80)]))?;

    if json {
        let _ = emit_json(
            "query",
            true,
            &serde_json::json!({
                "question": question,
                "index_md": index_md,
                "top_pages": page_contexts,
                "save_answer": save,
                "instruction": "Use the index_md and top_pages to synthesise an answer to the question. If save_answer is true, write the answer as a new page in wiki/published/by-topic/.",
            }),
        );
    } else {
        println!("# Query: {}", question);
        println!();
        println!("## Wiki Index\n{}", index_md);
        println!();
        println!("## Top Matching Pages ({} found)", page_contexts.len());
        for p in &page_contexts {
            println!("\n### {}", p["title"].as_str().unwrap_or("?"));
            println!("{}", p["body"].as_str().unwrap_or(""));
        }
        if page_contexts.is_empty() {
            println!("No matching pages found. Try `curio intake` to add more content.");
        }
        if save {
            println!("\n---");
            println!("(save mode: write the answer above back to the wiki as a new page)");
        }
    }
    Ok(())
}
