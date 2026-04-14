/// Routing eval harness — validates quality gates, manifest schema shape,
/// and shallow-route detection against fixture inputs.
///
/// These tests do NOT call the LLM. They validate the deterministic Rust
/// side of the routing pipeline: quality assessment, overlap detection,
/// proposal lane assignment, and manifest field completeness.

use curio::quality::assess_quality;
use curio::proposal::{required_lane, ProposalLane};

// ─── Quality gate tests ────────────────────────────────────────────────────

#[test]
fn low_signal_meeting_notes_should_not_be_publishable() {
    let title = "Notes from Meeting";
    let body = "Some rough notes taken during Q3 planning session.\nTODO: follow up with team.\nAction items TBD.";
    let q = assess_quality(title, body);
    // placeholder_pattern and low word count should kill publishability
    assert!(!q.publishable, "meeting notes with TODO/TBD should not be publishable");
    assert!(
        q.flags.iter().any(|f| f.contains("placeholder") || f.contains("low_word")),
        "expected placeholder or low_word_count flag, got: {:?}", q.flags
    );
}

#[test]
fn high_quality_upgrade_guide_is_publishable() {
    let title = "Alteryx Server 2024.1 Upgrade Guide";
    let body = r#"
# Alteryx Server 2024.1 Upgrade Guide

This guide covers the step-by-step upgrade procedure for Alteryx Server from 2023.2 to 2024.1.

## Prerequisites

- Administrative access to the server
- Backup of existing configuration files
- 4 hours maintenance window

## Upgrade Steps

1. Download the 2024.1 installer from the Alteryx portal
2. Stop all Alteryx Server services
3. Run the installer and follow the prompts
4. Verify service startup after installation
5. Run post-upgrade validation checks

## Common Issues

- Service startup failure: check Windows Event Log for startup errors
- Configuration rollback: restore from backup if validation fails
"#;
    let q = assess_quality(title, body);
    assert!(q.publishable, "detailed upgrade guide should be publishable; quality={:.2}, usability={:.2}, flags={:?}", q.information_quality, q.usability, q.flags);
    assert!(q.information_quality >= 0.45, "information_quality should be >= 0.45");
    assert!(q.usability >= 0.45, "usability should be >= 0.45");
}

#[test]
fn empty_body_is_not_publishable() {
    let q = assess_quality("Some Title", "");
    assert!(!q.publishable);
    assert!(q.information_quality < 0.45 || q.usability < 0.45);
}

#[test]
fn hub_body_with_only_child_list_gets_low_quality_score() {
    // Hub pages synthesized from children-only Confluence pages should be
    // recognised as low-quality and routed to review, not staged.
    let title = "Alteryx Server";
    let body = "Hub page: Alteryx Server\n\nThis page organizes the following sub-pages:\n\n- Upgrade Guides\n- Troubleshooting\n- Administration\n";
    let q = assess_quality(title, body);
    // Hub bodies are intentionally low-signal — they should not be staged directly.
    assert!(!q.publishable || q.information_quality < 0.6,
        "hub-only body should not pass the quality gate with high confidence; quality={:.2}", q.information_quality);
}

// ─── Proposal lane tests ───────────────────────────────────────────────────

#[test]
fn low_confidence_routes_to_review() {
    let lane = required_lane(0.60, 0.8, 0.8, 0.0, false, false);
    assert_eq!(lane, ProposalLane::Review);
}

#[test]
fn high_overlap_routes_to_review() {
    let lane = required_lane(0.9, 0.9, 0.9, 0.75, false, false);
    assert_eq!(lane, ProposalLane::Review);
}

#[test]
fn taxonomy_mutation_forces_review() {
    let lane = required_lane(0.9, 0.9, 0.9, 0.0, true, false);
    assert_eq!(lane, ProposalLane::Review);
}

#[test]
fn explicit_review_reason_forces_review() {
    let lane = required_lane(0.9, 0.9, 0.9, 0.0, false, true);
    assert_eq!(lane, ProposalLane::Review);
}

#[test]
fn strong_clean_signal_can_be_staged() {
    let lane = required_lane(0.85, 0.75, 0.80, 0.1, false, false);
    assert_eq!(lane, ProposalLane::Staged);
}

#[test]
fn quality_below_threshold_routes_to_review() {
    let lane = required_lane(0.9, 0.45, 0.9, 0.0, false, false);
    assert_eq!(lane, ProposalLane::Review);
}

// ─── Fixture manifest field tests ─────────────────────────────────────────

/// Verify that the fixture input files are valid wiki pages with the
/// expected frontmatter fields. This locks the fixture format.
#[test]
fn fixture_inputs_parse_correctly() {
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/routing");

    let cases = ["clean-leaf", "hub-page", "low-confidence"];
    for case in &cases {
        let input_path = fixture_dir.join(case).join("input.md");
        assert!(input_path.exists(), "fixture input.md missing for case '{}'", case);
        let page = curio::wiki_fs::parse_wiki_page(&input_path)
            .unwrap_or_else(|e| panic!("failed to parse fixture '{}': {}", case, e));
        assert!(!page.frontmatter.id.is_empty(), "id missing in fixture '{}'", case);
        assert!(!page.frontmatter.title.is_empty(), "title missing in fixture '{}'", case);
        assert_eq!(page.frontmatter.status.to_string(), "intake", "fixture '{}' should have intake status", case);
        assert!(!page.body.is_empty(), "body missing in fixture '{}'", case);
    }
}

/// Verify that clean-leaf fixture passes the quality gate (it's a real guide)
/// and hub/low-confidence fixtures do not.
#[test]
fn fixture_quality_gate_expectations() {
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/routing");

    let check = |case: &str, expect_publishable: bool| {
        let path = fixture_dir.join(case).join("input.md");
        let page = curio::wiki_fs::parse_wiki_page(&path).unwrap();
        let q = assess_quality(&page.frontmatter.title, &page.body);
        if expect_publishable {
            assert!(q.publishable, "fixture '{}' should be publishable; q={:.2}, u={:.2}, flags={:?}", case, q.information_quality, q.usability, q.flags);
        } else {
            assert!(!q.publishable, "fixture '{}' should NOT be publishable; q={:.2}, u={:.2}", case, q.information_quality, q.usability);
        }
    };

    check("clean-leaf", true);
    check("hub-page", false);
    check("low-confidence", false);
}
