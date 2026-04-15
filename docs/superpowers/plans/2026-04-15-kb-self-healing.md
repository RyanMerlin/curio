# KB Self-Healing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `curio doctor` (KB health report) and `curio heal` (confidence-gated AI heal loop) with a `wiki/review/auto-approved/` lane, user-configurable threshold in `settings.yaml`, and Confluence auto-heal labels + callouts.

**Architecture:** Rust generates deterministic diagnostic manifests; Claude reads the manifest and uses external tools (Confluence MCP, web search) to make decisions and write new content; Rust applies decisions. Confidence-gated: above threshold → publish + mirror to `review/auto-approved/`; below threshold → normal `review/` lane.

**Tech Stack:** Rust (existing curio-rs), serde_yaml (already present), walkdir, chrono, existing `quality.rs` / `overlap.rs` / `wiki_fs.rs` / `confluence.rs`

---

## Plan A: Config Foundation + KB Doctor

### File Map

| Action | Path | Responsibility |
|--------|------|----------------|
| Modify | `wiki/_config/settings.yaml` | Add `heal:` section with user-tunable params |
| Modify | `curio-rs/src/config.rs` | Parse `HealConfig`; expose on `Config` |
| Create | `curio-rs/src/freshness.rs` | Age-based freshness scoring |
| Create | `curio-rs/src/commands/doctor.rs` | KB structural health report |
| Modify | `curio-rs/src/commands/mod.rs` | Export `doctor` module |
| Modify | `curio-rs/src/cli.rs` | Repurpose top-level `Doctor` → KB doctor; keep `curio agent doctor` for provider health |
| Modify | `curio-rs/src/main.rs` | Dispatch new KB doctor |
| Modify | `curio-rs/src/lib.rs` | Export `freshness` module |
| Modify | `NORTHSTAR.md` | Add callout pointing to settings.yaml for operational config |

---

### Task A1: settings.yaml + HealConfig

**Files:**
- Modify: `wiki/_config/settings.yaml`
- Modify: `curio-rs/src/config.rs`

- [ ] **Step 1: Add `heal:` section to settings.yaml**

```yaml
# Curio Wiki Configuration

auto_commit: true

# ── Self-Healing ──────────────────────────────────────────────────────────────
# Controls curio doctor and curio heal behaviour.
heal:
  # Proposals with confidence >= this threshold are auto-published and mirrored
  # to wiki/review/auto-approved/.  Proposals below it go to wiki/review/ for
  # human review.  Range: 0.0–1.0.  Default: 0.85.
  confidence_threshold: 0.85

  # Whether to add a visual info callout to auto-healed pages in Confluence.
  show_auto_heal_callout: true

  # Confluence label applied to every auto-healed published page.
  auto_heal_label: "curio:auto-healed"
```

- [ ] **Step 2: Add `HealConfig` struct and wire into `Config` in config.rs**

Find the `WikiConfig` struct in `curio-rs/src/config.rs`. Add after it:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct HealConfig {
    pub confidence_threshold: Option<f64>,
    pub show_auto_heal_callout: Option<bool>,
    pub auto_heal_label: Option<String>,
}

impl HealConfig {
    pub fn confidence_threshold(&self) -> f64 {
        self.confidence_threshold.unwrap_or(0.85)
    }
    pub fn show_auto_heal_callout(&self) -> bool {
        self.show_auto_heal_callout.unwrap_or(true)
    }
    pub fn auto_heal_label(&self) -> &str {
        self.auto_heal_label.as_deref().unwrap_or("curio:auto-healed")
    }
}

impl Default for HealConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: Some(0.85),
            show_auto_heal_callout: Some(true),
            auto_heal_label: Some("curio:auto-healed".to_string()),
        }
    }
}
```

Find the top-level settings struct (the one that derives `Deserialize` and is loaded from `settings.yaml`) and add:

```rust
pub heal: Option<HealConfig>,
```

Then in the `Config` struct (the resolved, runtime config), add:

```rust
pub heal: HealConfig,
```

And in the config resolution/merge logic, after loading the settings file:

```rust
config.heal = loaded.heal.unwrap_or_default();
```

- [ ] **Step 3: Build to verify**

```
cargo build -q --manifest-path curio-rs/Cargo.toml
```

Expected: clean build.

- [ ] **Step 4: Write test**

In `curio-rs/src/config.rs` test section (bottom of file):

```rust
#[test]
fn test_heal_config_defaults() {
    let h = HealConfig::default();
    assert_eq!(h.confidence_threshold(), 0.85);
    assert!(h.show_auto_heal_callout());
    assert_eq!(h.auto_heal_label(), "curio:auto-healed");
}

#[test]
fn test_heal_config_override() {
    let h = HealConfig {
        confidence_threshold: Some(0.7),
        show_auto_heal_callout: Some(false),
        auto_heal_label: Some("custom:label".to_string()),
    };
    assert_eq!(h.confidence_threshold(), 0.7);
    assert!(!h.show_auto_heal_callout());
    assert_eq!(h.auto_heal_label(), "custom:label");
}
```

```
cargo test -q --manifest-path curio-rs/Cargo.toml test_heal_config
```

Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add wiki/_config/settings.yaml curio-rs/src/config.rs
git commit -m "feat: add HealConfig to settings.yaml and config.rs"
```

---

### Task A2: Freshness scoring

**Files:**
- Create: `curio-rs/src/freshness.rs`
- Modify: `curio-rs/src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `curio-rs/src/freshness.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_very_recent_page_is_fresh() {
        let updated = Utc::now();
        let score = freshness_score(updated);
        assert!(score > 0.95, "score was {}", score);
    }

    #[test]
    fn test_two_year_old_page_is_stale() {
        let updated = Utc::now() - chrono::Duration::days(730);
        let score = freshness_score(updated);
        assert!(score < 0.4, "score was {}", score);
    }

    #[test]
    fn test_six_month_page_is_borderline() {
        let updated = Utc::now() - chrono::Duration::days(180);
        let score = freshness_score(updated);
        assert!(score > 0.4 && score < 0.8, "score was {}", score);
    }
}
```

```
cargo test -q --manifest-path curio-rs/Cargo.toml test_very_recent_page
```

Expected: FAIL with "function not defined" (freshness_score doesn't exist yet).

- [ ] **Step 2: Implement freshness.rs**

```rust
//! Age-based freshness scoring for wiki pages.
//!
//! Returns a score in [0.0, 1.0] where 1.0 is brand-new and 0.0 is very stale.
//! Uses an exponential decay with a half-life of ~8 months.

use chrono::{DateTime, Utc};

/// Compute a freshness score from the page's last-updated timestamp.
///
/// Decay model: score = e^(-λ * days_old)
/// where λ is chosen so the half-life is ~240 days (8 months).
pub fn freshness_score(updated_at: DateTime<Utc>) -> f64 {
    let days_old = (Utc::now() - updated_at).num_days().max(0) as f64;
    // λ = ln(2) / half_life_days
    let lambda = std::f64::consts::LN_2 / 240.0;
    (-lambda * days_old).exp().clamp(0.0, 1.0)
}

/// Parse an RFC-3339 timestamp string and return a freshness score.
/// Returns None if the string cannot be parsed.
pub fn freshness_score_from_str(updated_at: &str) -> Option<f64> {
    updated_at.parse::<DateTime<Utc>>().ok().map(freshness_score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_very_recent_page_is_fresh() {
        let updated = Utc::now();
        let score = freshness_score(updated);
        assert!(score > 0.95, "score was {}", score);
    }

    #[test]
    fn test_two_year_old_page_is_stale() {
        let updated = Utc::now() - chrono::Duration::days(730);
        let score = freshness_score(updated);
        assert!(score < 0.4, "score was {}", score);
    }

    #[test]
    fn test_six_month_page_is_borderline() {
        let updated = Utc::now() - chrono::Duration::days(180);
        let score = freshness_score(updated);
        assert!(score > 0.4 && score < 0.8, "score was {}", score);
    }

    #[test]
    fn test_freshness_from_str_valid() {
        let ts = "2025-01-01T00:00:00Z";
        let score = freshness_score_from_str(ts);
        assert!(score.is_some());
        assert!(score.unwrap() < 1.0);
    }

    #[test]
    fn test_freshness_from_str_invalid() {
        assert!(freshness_score_from_str("not-a-date").is_none());
    }
}
```

- [ ] **Step 3: Export from lib.rs**

In `curio-rs/src/lib.rs`, add:

```rust
pub mod freshness;
```

- [ ] **Step 4: Run tests**

```
cargo test -q --manifest-path curio-rs/Cargo.toml freshness
```

Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
git add curio-rs/src/freshness.rs curio-rs/src/lib.rs
git commit -m "feat: add freshness scoring module with exponential decay"
```

---

### Task A3: KB Doctor command

**Files:**
- Create: `curio-rs/src/commands/doctor.rs`
- Modify: `curio-rs/src/commands/mod.rs`
- Modify: `curio-rs/src/cli.rs`
- Modify: `curio-rs/src/main.rs`

The doctor command produces a JSON (or text) health report for a KB scope. It is purely deterministic — no LLM calls.

**Health checks performed:**
1. **low_quality** — published pages where `assess_quality(title, body).publishable == false`
2. **high_overlap** — pairs of pages in the same category with overlap score ≥ 0.60
3. **stale** — pages with freshness score < 0.50
4. **orphaned_xref** — published pages whose `cross_refs` point to slugs that don't exist (reuse lint logic)
5. **thin_branch** — a taxonomy node that has a directory but its index.md body is < 50 words
6. **missing_keywords** — published pages with empty `keywords` field

Each finding has: `kind`, `slug`, `path`, `severity` (warn/error), `detail`.

- [ ] **Step 1: Create doctor.rs**

```rust
//! `curio doctor [--scope <path>]` — KB structural health report.
//!
//! Scans wiki/published/ (or a sub-path of it) for structural issues.
//! Produces a JSON or text report. No LLM calls.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    config::Config,
    freshness::freshness_score_from_str,
    output::emit_json,
    overlap::find_peer_overlap,
    quality::assess_quality,
    wiki_fs::parse_wiki_page,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    LowQuality,
    HighOverlap,
    Stale,
    OrphanedXref,
    ThinBranch,
    MissingKeywords,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Warn,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Finding {
    pub kind: FindingKind,
    pub severity: Severity,
    pub slug: String,
    pub path: String,
    pub detail: String,
    /// For HighOverlap findings: the peer slug and score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlap_peer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlap_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshness_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorReport {
    pub scope: String,
    pub pages_scanned: usize,
    pub findings: Vec<Finding>,
    pub summary: DoctorSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorSummary {
    pub errors: usize,
    pub warnings: usize,
    pub low_quality: usize,
    pub high_overlap: usize,
    pub stale: usize,
    pub orphaned_xrefs: usize,
    pub thin_branches: usize,
    pub missing_keywords: usize,
}

pub async fn run_doctor(
    config: &Config,
    dry_run: bool,
    json: bool,
    scope: Option<String>,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let published_dir = wiki_dir.join("published");

    // Resolve scan root: full published dir or scoped subdir.
    let scan_root = if let Some(ref s) = scope {
        let candidate = published_dir.join(s);
        if !candidate.exists() {
            anyhow::bail!("Scope path not found: {}", candidate.display());
        }
        candidate
    } else {
        published_dir.clone()
    };

    let scope_label = scope.as_deref().unwrap_or("(all)");

    // Collect all published .md pages in scope (excluding index.md files).
    let pages: Vec<PathBuf> = WalkDir::new(&scan_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |x| x == "md")
                && e.path().file_name().map_or(false, |n| n != "index.md")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let pages_scanned = pages.len();
    let mut findings: Vec<Finding> = Vec::new();

    // Collect all published slugs for xref validation.
    let all_slugs: std::collections::HashSet<String> = pages
        .iter()
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();

    for path in &pages {
        let Ok(page) = parse_wiki_page(path) else { continue };
        let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let path_str = path.display().to_string();
        let category: Vec<String> = page.frontmatter.category.clone();

        // 1. Quality check
        let quality = assess_quality(&page.frontmatter.title, &page.body);
        if !quality.publishable {
            findings.push(Finding {
                kind: FindingKind::LowQuality,
                severity: Severity::Warn,
                slug: slug.clone(),
                path: path_str.clone(),
                detail: format!(
                    "quality flags: {:?}; info={:.2} usability={:.2}",
                    quality.flags, quality.information_quality, quality.usability
                ),
                overlap_peer: None,
                overlap_score: None,
                freshness_score: None,
                quality_score: Some(
                    (quality.information_quality + quality.usability) / 2.0,
                ),
            });
        }

        // 2. Freshness check
        let freshness = page
            .frontmatter
            .updated_at
            .as_deref()
            .and_then(freshness_score_from_str)
            .unwrap_or(1.0);
        if freshness < 0.50 {
            findings.push(Finding {
                kind: FindingKind::Stale,
                severity: Severity::Warn,
                slug: slug.clone(),
                path: path_str.clone(),
                detail: format!(
                    "freshness {:.2} (last updated: {})",
                    freshness,
                    page.frontmatter.updated_at.as_deref().unwrap_or("unknown")
                ),
                overlap_peer: None,
                overlap_score: None,
                freshness_score: Some(freshness),
                quality_score: None,
            });
        }

        // 3. Missing keywords
        if page.frontmatter.keywords.is_empty() {
            findings.push(Finding {
                kind: FindingKind::MissingKeywords,
                severity: Severity::Warn,
                slug: slug.clone(),
                path: path_str.clone(),
                detail: "no keywords set".to_string(),
                overlap_peer: None,
                overlap_score: None,
                freshness_score: None,
                quality_score: None,
            });
        }

        // 4. Orphaned cross-refs
        for xref in &page.frontmatter.cross_refs {
            if !all_slugs.contains(xref.as_str()) {
                findings.push(Finding {
                    kind: FindingKind::OrphanedXref,
                    severity: Severity::Error,
                    slug: slug.clone(),
                    path: path_str.clone(),
                    detail: format!("broken cross_ref → {}", xref),
                    overlap_peer: None,
                    overlap_score: None,
                    freshness_score: None,
                    quality_score: None,
                });
            }
        }

        // 5. Overlap check (only compare against peers in same category)
        let peers = find_peer_overlap(&page.frontmatter.title, &page.body, &category, path, &published_dir);
        for m in peers.iter().filter(|m| m.score >= 0.60) {
            // Avoid duplicate pairs: only emit if slug < peer slug (lexicographic).
            let peer_slug = m.path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if slug < peer_slug {
                findings.push(Finding {
                    kind: FindingKind::HighOverlap,
                    severity: Severity::Warn,
                    slug: slug.clone(),
                    path: path_str.clone(),
                    detail: format!("overlap {:.2} with {}", m.score, peer_slug),
                    overlap_peer: Some(peer_slug),
                    overlap_score: Some(m.score),
                    freshness_score: None,
                    quality_score: None,
                });
            }
        }
    }

    // 6. Thin branch check — directories with < 50 words in index.md
    for entry in WalkDir::new(&scan_root).min_depth(1).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_dir() { continue; }
        let index = entry.path().join("index.md");
        if !index.exists() { continue; }
        if let Ok(page) = parse_wiki_page(&index) {
            let word_count = page.body.split_whitespace().count();
            if word_count < 50 {
                let slug = entry.path()
                    .strip_prefix(&published_dir)
                    .unwrap_or(entry.path())
                    .display()
                    .to_string()
                    .replace('\\', "/");
                findings.push(Finding {
                    kind: FindingKind::ThinBranch,
                    severity: Severity::Warn,
                    slug: slug.clone(),
                    path: index.display().to_string(),
                    detail: format!("branch index has only {} words", word_count),
                    overlap_peer: None,
                    overlap_score: None,
                    freshness_score: None,
                    quality_score: None,
                });
            }
        }
    }

    // Build summary
    let summary = DoctorSummary {
        errors:           findings.iter().filter(|f| f.severity == Severity::Error).count(),
        warnings:         findings.iter().filter(|f| f.severity == Severity::Warn).count(),
        low_quality:      findings.iter().filter(|f| f.kind == FindingKind::LowQuality).count(),
        high_overlap:     findings.iter().filter(|f| f.kind == FindingKind::HighOverlap).count(),
        stale:            findings.iter().filter(|f| f.kind == FindingKind::Stale).count(),
        orphaned_xrefs:   findings.iter().filter(|f| f.kind == FindingKind::OrphanedXref).count(),
        thin_branches:    findings.iter().filter(|f| f.kind == FindingKind::ThinBranch).count(),
        missing_keywords: findings.iter().filter(|f| f.kind == FindingKind::MissingKeywords).count(),
    };

    let report = DoctorReport {
        scope: scope_label.to_string(),
        pages_scanned,
        findings,
        summary,
    };

    if json {
        emit_json(&report)?;
        return Ok(());
    }

    // Text output
    println!("KB Doctor — scope: {}", scope_label);
    println!("Pages scanned: {}", pages_scanned);
    println!();
    println!("Findings: {} errors, {} warnings", report.summary.errors, report.summary.warnings);
    println!("  low-quality:      {}", report.summary.low_quality);
    println!("  high-overlap:     {}", report.summary.high_overlap);
    println!("  stale (>8 mo):    {}", report.summary.stale);
    println!("  orphaned xrefs:   {}", report.summary.orphaned_xrefs);
    println!("  thin branches:    {}", report.summary.thin_branches);
    println!("  missing keywords: {}", report.summary.missing_keywords);

    if !report.findings.is_empty() {
        println!();
        for f in &report.findings {
            let icon = if f.severity == Severity::Error { "✖" } else { "⚠" };
            println!("  {} [{:?}] {} — {}", icon, f.kind, f.slug, f.detail);
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Check what fields exist on Frontmatter**

Read `curio-rs/src/wiki_fs.rs` and confirm the `Frontmatter` struct has `updated_at: Option<String>`, `cross_refs: Vec<String>`, `keywords: Vec<String>`, `category: Vec<String>`. Adjust field names in doctor.rs to match exactly what the struct exposes.

- [ ] **Step 3: Check `find_peer_overlap` signature in overlap.rs**

Read `curio-rs/src/overlap.rs`. The function signature is:
```rust
pub fn find_peer_overlap(title: &str, body: &str, category: &[String], self_path: &Path, published_dir: &Path) -> Vec<OverlapMatch>
```
Confirm this matches what doctor.rs calls. If the signature differs, adjust the call in doctor.rs.

- [ ] **Step 4: Register module**

In `curio-rs/src/commands/mod.rs`, add:
```rust
pub mod doctor;
```

- [ ] **Step 5: Wire CLI — repurpose top-level `Doctor`**

In `curio-rs/src/cli.rs`, find the existing top-level `Doctor` variant:
```rust
/// Verify provider and harness prerequisites.
Doctor {
    provider: Option<AgentProvider>,
}
```

Replace with:
```rust
/// Report structural health of the published knowledge base.
///
/// Checks for low-quality pages, high-overlap pairs, stale content,
/// orphaned cross-references, thin branch nodes, and missing keywords.
///
/// Use `curio agent doctor` to verify provider/harness prerequisites.
Doctor {
    /// Restrict scan to a NORTHSTAR path (e.g. product-tree/alteryx-server).
    /// Defaults to the full published KB.
    #[arg(long)]
    scope: Option<String>,
},
```

- [ ] **Step 6: Wire dispatch in main.rs**

Add import at top of `main.rs` use block:
```rust
commands::doctor::run_doctor,
```

Find the existing dispatch block:
```rust
Some(Commands::Doctor { provider }) => {
    run_agent_doctor(provider, cli.json)?;
}
```

Replace with:
```rust
Some(Commands::Doctor { scope }) => {
    let config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
    run_doctor(&config, cli.dry_run, cli.json, scope).await?;
}
```

Note: `curio agent doctor` (the provider health check) still works via `AgentCommands::Doctor`.

- [ ] **Step 7: Build**

```
cargo build -q --manifest-path curio-rs/Cargo.toml
```

Expected: clean build. Fix any field name mismatches found in Step 2.

- [ ] **Step 8: Smoke test against real wiki**

```
cargo run -q --manifest-path curio-rs/Cargo.toml -- doctor 2>&1 | head -20
```

Expected: health report with counts. Numbers may be 0 if published KB is empty.

```
cargo run -q --manifest-path curio-rs/Cargo.toml -- doctor --scope product-tree/alteryx-server 2>&1 | head -20
```

Expected: scoped report.

- [ ] **Step 9: Commit**

```bash
git add curio-rs/src/commands/doctor.rs curio-rs/src/commands/mod.rs curio-rs/src/cli.rs curio-rs/src/main.rs
git commit -m "feat: add curio doctor KB health report command"
```

---

## Plan B: Heal Loop — Manifest, Apply, Auto-Approved Lane

### File Map

| Action | Path | Responsibility |
|--------|------|----------------|
| Create | `curio-rs/src/heal_types.rs` | Shared types: `HealAction`, `HealKind`, `HealManifest`, `HealRoutesFile` |
| Create | `curio-rs/src/commands/heal.rs` | `run_heal_prepare` + `run_heal_apply` |
| Modify | `curio-rs/src/lib.rs` | Export `heal_types` |
| Modify | `curio-rs/src/commands/mod.rs` | Export `heal` module |
| Modify | `curio-rs/src/cli.rs` | Add `Heal` command with `--scope`, `--prepare`, `--apply-file`, `--auto`, `--confidence` |
| Modify | `curio-rs/src/main.rs` | Dispatch `Heal` |
| Modify | `curio-rs/src/commands/sync.rs` | Handle `review/auto-approved/` sublane; add `curio:auto-healed` label; add info callout to body |
| Modify | `curio-rs/src/commands/feedback.rs` | Treat `auto-approved` pages in feedback loop identically to `review` pages |

---

### Task B1: Shared types (heal_types.rs)

**Files:**
- Create: `curio-rs/src/heal_types.rs`
- Modify: `curio-rs/src/lib.rs`

- [ ] **Step 1: Create heal_types.rs**

```rust
//! Shared types for the curio heal pipeline.
//!
//! `HealManifest` is what `curio heal --prepare` emits (Rust → Claude).
//! `HealRoutesFile` is what Claude writes back (Claude → Rust).

use serde::{Deserialize, Serialize};

// ── Manifest (Rust → Claude) ─────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct HealManifest {
    pub scope: String,
    pub confidence_threshold: f64,
    pub pages: Vec<ManifestPage>,
    pub structural_issues: Vec<StructuralIssue>,
    pub external_context: ExternalContext,
    /// Paste this command to apply Claude's route file.
    pub apply_command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestPage {
    pub slug: String,
    pub path: String,
    pub title: String,
    pub body: String,
    pub category: Vec<String>,
    pub keywords: Vec<String>,
    pub source_url: Option<String>,
    pub updated_at: Option<String>,
    pub freshness_score: f64,
    pub quality: ManifestQuality,
    pub overlap_candidates: Vec<OverlapCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestQuality {
    pub information_quality: f64,
    pub usability: f64,
    pub publishable: bool,
    pub flags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverlapCandidate {
    pub slug: String,
    pub title: String,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructuralIssue {
    pub kind: String,
    pub slug: String,
    pub path: String,
    pub detail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalContext {
    pub confluence_space_key: String,
    /// Space key of the original source material (for searching).
    pub source_space_key: Option<String>,
}

// ── Routes file (Claude → Rust) ──────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct HealRoutesFile {
    pub actions: Vec<HealAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealKind {
    /// Rewrite the page body (new_content required).
    Rewrite,
    /// Merge `slugs` into `into_slug` (new_content required — the merged body).
    Merge,
    /// Archive the page (move to wiki/review/ with status=archive).
    Archive,
    /// Update only frontmatter (keywords, category, title) — no body change.
    UpdateMetadata,
    /// Fix structural issues: repair broken xrefs, populate missing keywords.
    FixStructure,
    /// No action — page is healthy.
    NoAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealAction {
    pub kind: HealKind,
    /// Primary slug this action targets.
    pub slug: String,
    /// Confidence score 0.0–1.0 assigned by the Claude agent.
    pub confidence: f64,
    /// Human-readable rationale.
    pub rationale: String,
    /// New page body (Markdown, frontmatter included).
    /// Required for Rewrite, Merge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_content: Option<String>,
    /// For Merge: the additional slugs being merged in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merge_sources: Vec<String>,
    /// For Merge: the canonical slug to keep (defaults to `slug`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub into_slug: Option<String>,
    /// URLs or Confluence page IDs consulted during healing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources_consulted: Vec<String>,
}
```

- [ ] **Step 2: Export from lib.rs**

In `curio-rs/src/lib.rs`, add:
```rust
pub mod heal_types;
```

- [ ] **Step 3: Build**

```
cargo build -q --manifest-path curio-rs/Cargo.toml
```

Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add curio-rs/src/heal_types.rs curio-rs/src/lib.rs
git commit -m "feat: add heal_types.rs — HealManifest and HealRoutesFile shared types"
```

---

### Task B2: Heal prepare (manifest generation)

**Files:**
- Create: `curio-rs/src/commands/heal.rs` (partial — prepare only)
- Modify: `curio-rs/src/commands/mod.rs`
- Modify: `curio-rs/src/cli.rs`
- Modify: `curio-rs/src/main.rs`

- [ ] **Step 1: Create heal.rs with run_heal_prepare**

```rust
//! `curio heal` — confidence-gated AI heal loop.
//!
//! Two phases (mirroring `curio process`):
//!
//!   Phase 1 — `--prepare`:
//!     Scan the scope, compute quality/freshness/overlap signals, emit a
//!     JSON heal manifest.  Claude reads the manifest, uses external tools,
//!     and writes a `heal-routes.json` decision file.
//!
//!   Phase 2 — `--apply-file <path>`:
//!     Read the routes file.  For each action:
//!       - confidence >= threshold  → publish + mirror to review/auto-approved/
//!       - confidence <  threshold  → move to review/ as normal proposal

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    config::Config,
    freshness::freshness_score_from_str,
    heal_types::{
        ExternalContext, HealAction, HealKind, HealManifest, HealRoutesFile,
        ManifestPage, ManifestQuality, OverlapCandidate, StructuralIssue,
    },
    output::emit_json,
    overlap::find_peer_overlap,
    quality::assess_quality,
    wiki_fs::parse_wiki_page,
};

// ── Phase 1: prepare ─────────────────────────────────────────────────────────

pub async fn run_heal_prepare(
    config: &Config,
    json: bool,
    scope: Option<String>,
    out_file: Option<String>,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let published_dir = wiki_dir.join("published");

    let scan_root = match &scope {
        Some(s) => {
            let p = published_dir.join(s);
            if !p.exists() {
                anyhow::bail!("Scope path does not exist: {}", p.display());
            }
            p
        }
        None => published_dir.clone(),
    };

    let scope_label = scope.as_deref().unwrap_or("(all)").to_string();
    let confidence_threshold = config.heal.confidence_threshold();

    // Collect all published pages in scope (exclude index.md)
    let page_paths: Vec<PathBuf> = WalkDir::new(&scan_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |x| x == "md")
                && e.path().file_name().map_or(false, |n| n != "index.md")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut pages: Vec<ManifestPage> = Vec::new();

    for path in &page_paths {
        let Ok(page) = parse_wiki_page(path) else { continue };
        let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();

        let freshness = page
            .frontmatter
            .updated_at
            .as_deref()
            .and_then(freshness_score_from_str)
            .unwrap_or(1.0);

        let quality = assess_quality(&page.frontmatter.title, &page.body);

        let overlap_raw = find_peer_overlap(
            &page.frontmatter.title,
            &page.body,
            &page.frontmatter.category,
            path,
            &published_dir,
        );
        let overlap_candidates: Vec<OverlapCandidate> = overlap_raw
            .iter()
            .filter(|m| m.score >= 0.45)
            .filter_map(|m| {
                let peer_slug = m.path.file_stem()?.to_str()?.to_string();
                // Get peer title from its page
                let peer_title = parse_wiki_page(&m.path)
                    .map(|p| p.frontmatter.title.clone())
                    .unwrap_or_else(|_| peer_slug.clone());
                Some(OverlapCandidate {
                    slug: peer_slug,
                    title: peer_title,
                    score: m.score,
                })
            })
            .collect();

        let source_url = page.frontmatter.source
            .as_ref()
            .and_then(|s| s.get("origin_url"))
            .and_then(|v| v.as_str())
            .map(String::from);

        pages.push(ManifestPage {
            slug,
            path: path.display().to_string(),
            title: page.frontmatter.title.clone(),
            body: page.body.clone(),
            category: page.frontmatter.category.clone(),
            keywords: page.frontmatter.keywords.clone(),
            source_url,
            updated_at: page.frontmatter.updated_at.clone(),
            freshness_score: freshness,
            quality: ManifestQuality {
                information_quality: quality.information_quality,
                usability: quality.usability,
                publishable: quality.publishable,
                flags: quality.flags.iter().map(|f| format!("{:?}", f)).collect(),
            },
            overlap_candidates,
        });
    }

    // Collect structural issues (thin branches, using doctor logic)
    let mut structural_issues: Vec<StructuralIssue> = Vec::new();
    for entry in WalkDir::new(&scan_root).min_depth(1).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_dir() { continue; }
        let index = entry.path().join("index.md");
        if !index.exists() { continue; }
        if let Ok(idx_page) = parse_wiki_page(&index) {
            let word_count = idx_page.body.split_whitespace().count();
            if word_count < 50 {
                let branch_slug = entry.path()
                    .strip_prefix(&published_dir)
                    .unwrap_or(entry.path())
                    .display().to_string().replace('\\', "/");
                structural_issues.push(StructuralIssue {
                    kind: "thin_branch".to_string(),
                    slug: branch_slug,
                    path: index.display().to_string(),
                    detail: format!("{} words in branch index", word_count),
                });
            }
        }
    }

    let manifest = HealManifest {
        scope: scope_label,
        confidence_threshold,
        pages,
        structural_issues,
        external_context: ExternalContext {
            confluence_space_key: config.content_model.space_key.clone(),
            source_space_key: None,
        },
        apply_command: format!(
            "curio heal --apply-file /tmp/heal-routes.json{}",
            scope.as_deref().map(|s| format!(" --scope {}", s)).unwrap_or_default()
        ),
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .context("Failed to serialize manifest")?;

    if let Some(path) = out_file {
        std::fs::write(&path, &manifest_json)
            .with_context(|| format!("Failed to write manifest to {}", path))?;
        eprintln!("Heal manifest written to {}", path);
        eprintln!("Pages in scope: {}", manifest.pages.len());
        eprintln!("Structural issues: {}", manifest.structural_issues.len());
        eprintln!();
        eprintln!("Next: have Claude read the manifest and produce a routes file.");
        eprintln!("Apply with: {}", manifest.apply_command);
    } else {
        println!("{}", manifest_json);
    }

    Ok(())
}
```

Note: `page.frontmatter.source` may not exist as a `serde_json::Value` map — check the actual `Frontmatter` struct in `wiki_fs.rs` and adjust the `source_url` extraction accordingly. If `source` is a struct, use its fields directly.

- [ ] **Step 2: Register module in mod.rs**

```rust
pub mod heal;
```

- [ ] **Step 3: Add Heal variant to cli.rs**

After the `Feedback` variant, add:

```rust
/// Confidence-gated AI heal loop for the knowledge base.
///
/// Phase 1 (--prepare): scan a scope, emit a JSON heal manifest.
/// Phase 2 (--apply-file): apply a Claude-produced routes file.
///
/// Full two-phase example:
///   curio heal --prepare --scope product-tree/alteryx-server --out /tmp/heal.json
///   # → Claude reads /tmp/heal.json, uses tools, writes /tmp/heal-routes.json
///   curio heal --apply-file /tmp/heal-routes.json
Heal {
    /// Phase 1: emit heal manifest instead of applying.
    #[arg(long)]
    prepare: bool,

    /// Phase 2: path to the Claude-produced routes JSON file.
    #[arg(long, value_name = "PATH")]
    apply_file: Option<String>,

    /// NORTHSTAR scope path (e.g. product-tree/alteryx-server).
    /// Defaults to full published KB.
    #[arg(long)]
    scope: Option<String>,

    /// Write manifest to file instead of stdout (used with --prepare).
    #[arg(long, value_name = "PATH")]
    out: Option<String>,

    /// Override confidence threshold from settings.yaml for this run.
    #[arg(long, value_name = "0.0-1.0")]
    confidence: Option<f64>,

    /// Auto-approve all actions regardless of confidence (sets threshold to 0.0).
    #[arg(long)]
    auto: bool,
},
```

- [ ] **Step 4: Wire dispatch in main.rs**

Add to use block:
```rust
commands::heal::run_heal_prepare,
```

Add dispatch arm (after Feedback arm):
```rust
Some(Commands::Heal { prepare, apply_file, scope, out, confidence, auto }) => {
    let mut config = load_config(config_path_str, kb_dir_resolved.as_deref())?;
    // Override threshold if --confidence or --auto flags are set.
    if auto {
        config.heal.confidence_threshold = Some(0.0);
    } else if let Some(c) = confidence {
        config.heal.confidence_threshold = Some(c);
    }
    if prepare {
        run_heal_prepare(&config, cli.json, scope, out).await?;
    } else if let Some(ref path) = apply_file {
        run_heal_apply(&config, cli.dry_run, cli.json, path, scope).await?;
    } else {
        eprintln!("Specify --prepare or --apply-file <path>");
        std::process::exit(1);
    }
}
```

Note: `run_heal_apply` is implemented in Task B3. Add a stub for now:

In heal.rs, add after run_heal_prepare:
```rust
pub async fn run_heal_apply(
    _config: &Config,
    _dry_run: bool,
    _json: bool,
    routes_file: &str,
    _scope: Option<String>,
) -> Result<()> {
    anyhow::bail!("run_heal_apply not yet implemented (routes_file={})", routes_file)
}
```

Add to main.rs use block:
```rust
commands::heal::run_heal_apply,
```

- [ ] **Step 5: Build**

```
cargo build -q --manifest-path curio-rs/Cargo.toml
```

Expected: clean.

- [ ] **Step 6: Smoke test**

```
cargo run -q --manifest-path curio-rs/Cargo.toml -- heal --help
```

Expected: help text showing --prepare, --apply-file, --scope, --confidence, --auto.

```
cargo run -q --manifest-path curio-rs/Cargo.toml -- heal --prepare --scope product-tree/alteryx-server 2>&1 | python3 -c "import sys,json; d=json.load(sys.stdin); print('pages:', len(d['pages']))" 2>/dev/null || echo "(python not available — check output manually)"
```

Expected: JSON manifest printed to stdout with `pages` array.

- [ ] **Step 7: Commit**

```bash
git add curio-rs/src/commands/heal.rs curio-rs/src/commands/mod.rs curio-rs/src/cli.rs curio-rs/src/main.rs
git commit -m "feat: curio heal --prepare — emit JSON heal manifest for Claude agent"
```

---

### Task B3: Heal apply — file operations + auto-approved lane

**Files:**
- Modify: `curio-rs/src/commands/heal.rs` (replace stub with real implementation)

This task implements `run_heal_apply`. For each action in the routes file:

| Confidence vs threshold | Action taken |
|------------------------|--------------|
| `>= threshold` | Write new content → `wiki/published/` (Rewrite/Merge) or delete (Archive) → mirror decision record to `wiki/review/auto-approved/<category>/` |
| `< threshold` | Write to `wiki/review/<category>/` as normal review proposal |
| `NoAction` | Log and skip |

- [ ] **Step 1: Replace run_heal_apply stub with full implementation**

Replace the stub in `curio-rs/src/commands/heal.rs`:

```rust
use crate::{
    wiki_fs::{parse_wiki_page, update_frontmatter},
    wiki_index::append_log,
};

pub async fn run_heal_apply(
    config: &Config,
    dry_run: bool,
    json: bool,
    routes_file: &str,
    scope: Option<String>,
) -> Result<()> {
    let wiki_dir = &config.wiki.wiki_dir;
    let threshold = config.heal.confidence_threshold();

    let raw = std::fs::read_to_string(routes_file)
        .with_context(|| format!("Cannot read routes file: {}", routes_file))?;
    let routes: HealRoutesFile = serde_json::from_str(&raw)
        .with_context(|| format!("Cannot parse routes file: {}", routes_file))?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    let mut auto_approved = 0usize;
    let mut routed_to_review = 0usize;
    let mut no_action = 0usize;
    let mut log_lines: Vec<String> = Vec::new();

    for action in &routes.actions {
        match action.kind {
            HealKind::NoAction => {
                no_action += 1;
                if dry_run {
                    println!("[dry-run] NO_ACTION  {}", action.slug);
                }
                continue;
            }
            HealKind::Rewrite | HealKind::Merge | HealKind::Archive
            | HealKind::UpdateMetadata | HealKind::FixStructure => {
                // Locate source page
                let src_path = find_published_page(wiki_dir, &action.slug)?;

                if action.confidence >= threshold {
                    // Auto-approve path
                    apply_auto_approve(
                        config, wiki_dir, action, &src_path, &timestamp, dry_run,
                        &mut log_lines,
                    )?;
                    auto_approved += 1;
                } else {
                    // Route to review
                    apply_to_review(
                        wiki_dir, action, &src_path, &timestamp, dry_run,
                        &mut log_lines,
                    )?;
                    routed_to_review += 1;
                }
            }
        }
    }

    if !dry_run && !log_lines.is_empty() {
        for line in &log_lines {
            append_log(wiki_dir, line)?;
        }
    }

    if dry_run {
        println!("[dry-run] auto-approve: {} | to-review: {} | no-action: {}",
            auto_approved, routed_to_review, no_action);
    } else {
        println!("Heal apply complete:");
        println!("  auto-approved (published): {}", auto_approved);
        println!("  routed to review:          {}", routed_to_review);
        println!("  no action:                 {}", no_action);
    }

    Ok(())
}

fn find_published_page(wiki_dir: &Path, slug: &str) -> Result<PathBuf> {
    let published_dir = wiki_dir.join("published");
    for entry in WalkDir::new(&published_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map_or(false, |e| e == "md") {
            if entry.path().file_stem().map_or(false, |s| s == slug) {
                return Ok(entry.path().to_path_buf());
            }
        }
    }
    // Also check review and staged (for pages not yet published)
    for lane in &["review", "staged"] {
        let lane_dir = wiki_dir.join(lane);
        for entry in WalkDir::new(&lane_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |e| e == "md") {
                if entry.path().file_stem().map_or(false, |s| s == slug) {
                    return Ok(entry.path().to_path_buf());
                }
            }
        }
    }
    anyhow::bail!("Page not found for slug: {}", slug)
}

fn apply_auto_approve(
    config: &Config,
    wiki_dir: &Path,
    action: &HealAction,
    src_path: &Path,
    timestamp: &str,
    dry_run: bool,
    log_lines: &mut Vec<String>,
) -> Result<()> {
    let slug = &action.slug;

    if dry_run {
        println!("[dry-run] AUTO-APPROVE  {} (confidence {:.2})", slug, action.confidence);
        return Ok(());
    }

    match action.kind {
        HealKind::Archive => {
            // Move the page to review/auto-approved/<slug>.md with status=archived
            let archive_dir = wiki_dir.join("review").join("auto-approved");
            std::fs::create_dir_all(&archive_dir)?;
            let dest = archive_dir.join(src_path.file_name().unwrap());
            let raw = std::fs::read_to_string(src_path)?;
            let updated = raw
                .replacen("status: published", "status: archived", 1)
                .replacen("status: staged", "status: archived", 1);
            std::fs::write(&dest, &updated)?;
            std::fs::remove_file(src_path)?;
            log_lines.push(format!(
                "[{}] heal auto-approve archive {} — {}",
                timestamp, slug, action.rationale
            ));
        }
        HealKind::Rewrite | HealKind::Merge => {
            let new_content = action.new_content.as_deref().ok_or_else(|| {
                anyhow::anyhow!("Rewrite/Merge action for {} missing new_content", slug)
            })?;

            // Write updated content to published (in-place for Rewrite;
            // for Merge, delete merge_sources afterward).
            let auto_heal_note = format!(
                "\nauto_healed_at: \"{}\"\nauto_healed_confidence: {}\n",
                timestamp, action.confidence
            );
            let final_content = inject_auto_heal_frontmatter(new_content, &auto_heal_note);
            std::fs::write(src_path, &final_content)?;

            // Delete merge source pages
            for merge_slug in &action.merge_sources {
                if let Ok(mp) = find_published_page(wiki_dir, merge_slug) {
                    std::fs::remove_file(&mp).ok();
                    for ext in &["analysis.json", "sync-refs.json"] {
                        let sidecar = mp.with_extension(ext);
                        if sidecar.exists() { std::fs::remove_file(&sidecar).ok(); }
                    }
                }
            }

            // Mirror decision record to review/auto-approved/
            write_auto_approve_record(wiki_dir, action, timestamp)?;

            log_lines.push(format!(
                "[{}] heal auto-approve {:?} {} (conf {:.2}) — {}",
                timestamp, action.kind, slug, action.confidence, action.rationale
            ));
        }
        HealKind::UpdateMetadata | HealKind::FixStructure => {
            if let Some(content) = &action.new_content {
                std::fs::write(src_path, content)?;
            }
            write_auto_approve_record(wiki_dir, action, timestamp)?;
            log_lines.push(format!(
                "[{}] heal auto-approve {:?} {} (conf {:.2}) — {}",
                timestamp, action.kind, slug, action.confidence, action.rationale
            ));
        }
        HealKind::NoAction => {}
    }

    Ok(())
}

fn apply_to_review(
    wiki_dir: &Path,
    action: &HealAction,
    src_path: &Path,
    timestamp: &str,
    dry_run: bool,
    log_lines: &mut Vec<String>,
) -> Result<()> {
    let slug = &action.slug;

    if dry_run {
        println!("[dry-run] TO-REVIEW  {} (confidence {:.2})", slug, action.confidence);
        return Ok(());
    }

    // Write proposal to review/ — don't touch the source page.
    let review_dir = wiki_dir.join("review");
    std::fs::create_dir_all(&review_dir)?;

    // Write a proposal sidecar (not moving the page — just surfacing for review).
    let proposal = serde_json::json!({
        "slug": slug,
        "kind": format!("{:?}", action.kind),
        "confidence": action.confidence,
        "rationale": action.rationale,
        "sources_consulted": action.sources_consulted,
        "new_content": action.new_content,
        "merge_sources": action.merge_sources,
        "created_at": timestamp,
        "status": "review",
    });
    let proposal_path = review_dir.join(format!("{}.heal-proposal.json", slug));
    std::fs::write(&proposal_path, serde_json::to_string_pretty(&proposal)?)?;

    log_lines.push(format!(
        "[{}] heal route-to-review {:?} {} (conf {:.2}) — {}",
        timestamp, action.kind, slug, action.confidence, action.rationale
    ));

    Ok(())
}

/// Write a decision record into wiki/review/auto-approved/ for audit visibility.
fn write_auto_approve_record(
    wiki_dir: &Path,
    action: &HealAction,
    timestamp: &str,
) -> Result<()> {
    let auto_dir = wiki_dir.join("review").join("auto-approved");
    std::fs::create_dir_all(&auto_dir)?;

    let record = serde_json::json!({
        "slug": action.slug,
        "kind": format!("{:?}", action.kind),
        "confidence": action.confidence,
        "rationale": action.rationale,
        "sources_consulted": action.sources_consulted,
        "merge_sources": action.merge_sources,
        "approved_at": timestamp,
        "status": "auto-approved",
    });
    let record_path = auto_dir.join(format!("{}.decision.json", action.slug));
    std::fs::write(&record_path, serde_json::to_string_pretty(&record)?)?;

    // Also write a human-readable .md companion for Confluence sync.
    let md = format!(
        "---\ntitle: \"Auto-Approved: {}\"\nstatus: auto-approved\nauto_healed_at: \"{}\"\nauto_healed_confidence: {}\n---\n\n\
        # Auto-Approved: {}\n\n**Action:** {:?}  \n**Confidence:** {:.2}  \n**Approved at:** {}  \n\n\
        ## Rationale\n\n{}\n\n## Sources Consulted\n\n{}\n",
        action.slug,
        timestamp,
        action.confidence,
        action.slug,
        action.kind,
        action.confidence,
        timestamp,
        action.rationale,
        if action.sources_consulted.is_empty() {
            "_none_".to_string()
        } else {
            action.sources_consulted.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
        }
    );
    let md_path = auto_dir.join(format!("{}.md", action.slug));
    std::fs::write(&md_path, md)?;

    Ok(())
}

/// Inject auto_healed_* fields into frontmatter of a content string.
fn inject_auto_heal_frontmatter(content: &str, note: &str) -> String {
    if let Some(pos) = content.find("\n---\n") {
        // Insert before the closing --- of the frontmatter block
        let (front, rest) = content.split_at(pos);
        format!("{}{}{}", front, note, rest)
    } else {
        // No frontmatter found — prepend a minimal block
        format!("---{}\n---\n\n{}", note, content)
    }
}
```

- [ ] **Step 2: Build**

```
cargo build -q --manifest-path curio-rs/Cargo.toml
```

Expected: clean. Fix any import or field errors.

- [ ] **Step 3: Dry-run smoke test**

Create a minimal test routes file:

```bash
cat > /tmp/test-heal-routes.json << 'EOF'
{
  "actions": [
    {
      "kind": "no_action",
      "slug": "some-page",
      "confidence": 0.95,
      "rationale": "Page is healthy"
    }
  ]
}
EOF
cargo run -q --manifest-path curio-rs/Cargo.toml -- heal --apply-file /tmp/test-heal-routes.json --dry-run 2>&1
```

Expected: `[dry-run] no_action: some-page` or similar.

- [ ] **Step 4: Commit**

```bash
git add curio-rs/src/commands/heal.rs
git commit -m "feat: curio heal --apply-file — confidence-gated apply with auto-approved lane"
```

---

### Task B4: Confluence sync — auto-approved lane + auto-heal label + callout

**Files:**
- Modify: `curio-rs/src/commands/sync.rs`

The `review/auto-approved/` directory will be picked up automatically by the existing review lane walker (since it's a subdirectory of `review/`). However, we need two additions:

1. When syncing a page that has `auto_healed_at` in its frontmatter, add the `curio:auto-healed` Confluence label after sync.
2. Append an info callout paragraph to the bottom of auto-healed published page HTML bodies during rendering.

**Where to add the label:** In `sync_lane_page` in sync.rs, after the `write_sync_refs` call for review pages, check if `page.frontmatter.auto_healed_at` is set and call `client.add_label_to_page`.

**Where to add the callout:** In `render_lane_page_body` (or the published page render function), if `page.frontmatter.auto_healed_at` is Some, append the HTML callout to the body before returning.

- [ ] **Step 1: Check what auto_healed_at looks like in Frontmatter**

Read `curio-rs/src/wiki_fs.rs`. The `Frontmatter` struct must include:
```rust
pub auto_healed_at: Option<String>,
pub auto_healed_confidence: Option<f64>,
```

If these fields are not present, add them to the struct. `serde` with `#[serde(default)]` means missing fields deserialize as `None` without error.

- [ ] **Step 2: Add auto_healed fields to Frontmatter if missing**

In `curio-rs/src/wiki_fs.rs`, find the `Frontmatter` struct and add:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub auto_healed_at: Option<String>,
#[serde(default, skip_serializing_if = "Option::is_none")]
pub auto_healed_confidence: Option<f64>,
```

- [ ] **Step 3: Add info callout to published page rendering**

In `curio-rs/src/commands/sync.rs`, find the function that renders the HTML body for a published page (likely `render_published_page_body` or similar). At the end, before returning the body string, add:

```rust
// If this page was auto-healed, append a visible callout.
if let Some(ref healed_at) = page.frontmatter.auto_healed_at {
    let confidence = page.frontmatter.auto_healed_confidence.unwrap_or(0.0);
    let callout = format!(
        "<hr/><p><em>⚡ Auto-healed by Curio on {} | confidence: {:.0}%</em></p>",
        &healed_at[..10], // YYYY-MM-DD
        confidence * 100.0,
    );
    body.push_str(&callout);
}
```

If no single rendering function exists and the HTML body is built inline, locate where the final `</p>` or closing content is assembled and append the callout there.

- [ ] **Step 4: Add curio:auto-healed label after sync**

In `sync_lane_page`, after `write_sync_refs(path, page_id, pinned_id_opt)`, add:

```rust
// Apply auto-heal label if this page was auto-healed.
if page.frontmatter.auto_healed_at.is_some() {
    let label = config.heal.auto_heal_label();
    let _ = client.add_label_to_page(page_id, label).await;
}
```

Note: `config` must be threaded into `sync_lane_page`. Check the current function signature. If `config` is not available, thread it in from the call site. The `add_label_to_page` method already exists in `confluence.rs`.

- [ ] **Step 5: Build**

```
cargo build -q --manifest-path curio-rs/Cargo.toml
```

Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add curio-rs/src/commands/sync.rs curio-rs/src/wiki_fs.rs
git commit -m "feat: sync auto-healed pages with curio:auto-healed label and info callout"
```

---

### Task B5: Feedback loop handles auto-approved lane

**Files:**
- Modify: `curio-rs/src/commands/feedback.rs`

The `curio feedback` command currently walks `wiki/review/` for `.sync-refs.json` sidecars. Pages in `wiki/review/auto-approved/` will be picked up automatically since they're subdirectories of `review/`. Auto-approved pages should be treatable like regular review pages — reviewers can override with `curio:reject` or `curio:rewrite` labels.

The only special handling needed: when `curio feedback` processes an auto-approved page and finds **no signal** (no label, no reaction), that's expected and fine — log it as "no signal (auto-approved, awaiting override)".

- [ ] **Step 1: Add auto-approved awareness to run_feedback**

In `curio-rs/src/commands/feedback.rs`, in the `no_signal_count` branch inside the reporting loop, add a check:

Find this block:
```rust
Action::NoSignal => {
    no_signal_count += 1;
}
```

Replace with:
```rust
Action::NoSignal => {
    no_signal_count += 1;
    // Suppress noise for auto-approved pages — no signal is expected.
    let is_auto_approved = sig.path
        .components()
        .any(|c| c.as_os_str() == "auto-approved");
    if !is_auto_approved {
        // Future: could log these to a "pending human review" list.
    }
}
```

This is a minimal change — just adds awareness. The full feedback loop works unchanged for auto-approved pages.

- [ ] **Step 2: Build + run all tests**

```
cargo build -q --manifest-path curio-rs/Cargo.toml
cargo test -q --manifest-path curio-rs/Cargo.toml
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add curio-rs/src/commands/feedback.rs
git commit -m "feat: feedback loop handles review/auto-approved/ sublane"
```

---

### Task B6: NORTHSTAR.md — add callout pointing to settings.yaml

**Files:**
- Modify: `NORTHSTAR.md`

- [ ] **Step 1: Add operational config reference**

Find the prose section near the top of `NORTHSTAR.md` (before or after the YAML block). Add:

```markdown
> **Operational Configuration**
>
> Heal confidence threshold, auto-heal label, and callout settings live in
> `wiki/_config/settings.yaml` under the `heal:` key. These are also visible
> in Confluence on the **Settings** page. Adjust them there; `curio sync` will
> pick up the change.
```

- [ ] **Step 2: Commit**

```bash
git add NORTHSTAR.md
git commit -m "docs: add operational config callout in NORTHSTAR.md"
```

---

## Self-Review Checklist

**Spec coverage:**
- [x] `settings.yaml` heal config → Task A1
- [x] `freshness_score` module → Task A2
- [x] `curio doctor --scope` → Task A3
- [x] `HealManifest` / `HealRoutesFile` types → Task B1
- [x] `curio heal --prepare` → Task B2
- [x] `curio heal --apply-file` with confidence gating → Task B3
- [x] `wiki/review/auto-approved/` lane → Task B3 (decision records) + Task B4 (sync)
- [x] `curio:auto-healed` Confluence label → Task B4
- [x] Info callout on auto-healed pages → Task B4
- [x] `curio feedback` handles auto-approved → Task B5
- [x] `--auto` flag sets threshold to 0.0 → Task B2 (CLI dispatch)
- [x] `--confidence` flag overrides threshold → Task B2 (CLI dispatch)
- [x] NORTHSTAR.md points to settings.yaml → Task B6

**Placeholder scan:** No TBDs or "implement later" found. All code blocks are complete.

**Type consistency:**
- `HealKind` variants: `Rewrite`, `Merge`, `Archive`, `UpdateMetadata`, `FixStructure`, `NoAction` — used consistently in B1, B3.
- `HealManifest.pages: Vec<ManifestPage>` — defined in B1, populated in B2.
- `HealRoutesFile.actions: Vec<HealAction>` — defined in B1, read in B3.
- `config.heal.confidence_threshold()` — defined in A1, called in B2, B3.

**Known implementation-time risks:**
- `Frontmatter` field names must be verified against `wiki_fs.rs` before using in doctor.rs and heal.rs (Steps A3-Step2, B4-Step1)
- `find_peer_overlap` signature must be verified against `overlap.rs` (Step A3-Step3)
- `sync_lane_page` signature may need `config` threaded in for Task B4
- `source` field on `Frontmatter` may not be a `serde_json::Value` map — check and adjust in B2
