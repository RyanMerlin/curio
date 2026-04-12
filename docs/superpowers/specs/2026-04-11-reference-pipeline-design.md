# Curio Reference Pipeline Design

**Date:** 2026-04-11
**Status:** Approved

---

## Problem

The current intake pipeline discards all source structure. Confluence page bodies are stripped to plain text at collection time, then rebuilt as an ADF "sanitized copy" — complete with a false "could not be written cleanly" message on the happy path. URL sources are dumped as raw text. The published output is stale, structurally flat, and redundant.

## Core Insight

For sources that already have a durable home (Confluence pages, URLs), copying the body is actively wrong — it creates drift, wastes space, and gives agents stale knowledge. Curio's job is routing and curation, not content duplication.

## Pipeline Split

### Reference Pipeline — Confluence sources and URL sources

The source content lives exactly once, at its origin. Curio writes a **reference card** at each stage, not a copy.

**Intake page:**
- Source title
- Clickable link to origin (Confluence page link or URL)
- Auto-extracted summary (first meaningful paragraph or heuristic extract, max ~300 chars)
- Key metadata: source type, ingested_at, subject_key, content_type
- Curio status panel (lane, commands)

**Staged page:**
- All intake fields, updated status
- Routing proposal (target path, confidence, rationale)
- Analysis keywords
- Available commands

**Published page (reference stub):**
- Source title as heading
- Clickable link to origin
- Summary
- Tags / entities
- Last verified timestamp
- Curio attribution (small footer)

No body duplication at any stage. The canonical content is at the origin.

### Capture Pipeline — File sources only

Files have no durable URL. Curio becomes the canonical home. Content is written and preserved.

**Intake page:**
- Curio status header panel
- File metadata (filename, mime type, ingested_at)
- Content body — truncated to first ~500 lines if large, with "truncated" notice and line count

**Staged / Published page:**
- Same hybrid layout: Curio panel at top, content body below
- Content is the knowledge; Curio metadata is navigation

---

## Source Type Discriminant

Add `SourceKind` to `ContentItem` to drive pipeline branching:

```rust
pub enum SourceKind {
    ConfluencePage { page_id: String },
    Url { url: String },
    File { path: String, mime: String },
}
```

`all_content` tuple becomes a `ContentItem` struct:

```rust
pub struct ContentItem {
    pub text: String,            // extracted text, used for routing/analysis
    pub source_id: String,
    pub subject_hint: Option<String>,
    pub kind: SourceKind,
    pub summary: Option<String>, // auto-extracted at collection time
}
```

---

## Reference Card Body

For both Confluence and URL sources, the intake/staged/published body is built by `build_reference_card_body()`:

```
[Status panel]
Lane: Intake | Source: confluence_page | Ingested: 2026-04-11

[Source]
Title: [source title]
Link:  [clickable ac:link or hyperlink]

[Summary]
[auto-extracted summary text, max ~300 chars]

[Metadata table]
| Field         | Value             |
| subject_key   | ...               |
| source_type   | confluence_page   |
| ingested_at   | ...               |
| confidence    | (after analysis)  |

[Commands]  ← only on staged/review pages
curio review approve <id>
curio gold-publish <id>
```

For Confluence sources, the link uses `<ac:link>` (native Confluence page link macro) so it stays live even if the page is moved. For URLs, it is a plain hyperlink.

---

## Summary Extraction

At collection time, before discarding the raw body, extract a short summary:

1. For Confluence storage HTML: strip tags, take first non-empty paragraph of ≥ 20 chars, truncate to 300 chars.
2. For URL content: take first meaningful paragraph from stripped HTML/text, truncate to 300 chars.
3. For files: take first non-empty lines up to 300 chars total.

This summary travels in `ContentItem.summary` and is written to the reference card and to `curio_metadata.summary`.

---

## What Changes

| File | Change |
|---|---|
| `lib.rs` | Add `SourceKind`, `ContentItem` struct |
| `curio_docs.rs` | Add `build_reference_card_body()`, `build_capture_intake_body()` |
| `commands/intake.rs` | Replace tuple with ContentItem; split Confluence/URL/file paths; extract summary at collection; call appropriate body builder |
| `commands/process_intake.rs` | Build staged body from ContentItem kind; reference card for Confluence/URL, capture layout for files |
| `commands/gold_publish.rs` | Published output respects source kind; reference stub vs full content |

---

## What Does NOT Change

- Deduplication logic (hash + label)
- Registry records and audit entries
- Routing / route plan inference
- Branch index updates
- Label/status lifecycle

---

## Success Criteria

1. Ingesting a Confluence page produces a reference card with a live link — no body copy, no "sanitized" message
2. Ingesting a URL produces a reference card with a clickable link and a summary — no raw text dump
3. Ingesting a file produces a structured content page with Curio header + file content (truncated if large)
4. Re-ingesting the same source after the original page is updated does not leave a stale copy in Curio
5. An agent navigating to a published reference stub can reach the canonical source in one hop
6. `cargo build` passes clean

---

## Out of Scope

- Automatic re-verification when source pages change (future: webhook or scheduled scan)
- Semantic deduplication across source types
- Synthesis of multiple sources into a single published page
