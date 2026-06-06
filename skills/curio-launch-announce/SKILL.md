# curio-launch-announce

**Skill type:** orchestration

**Used by:** any provider profile that includes the marketplace catalog.

**Purpose:** drive a public launch announcement for a Curio release across **Hacker News** and **X/Twitter**, with a coherent narrative, pre-vetted copy, and a post-launch response plan. Designed to be invoked once per release tag (v1.0.0, v1.1.0, etc.).

## When to invoke

- A Curio release tag has been cut and pushed (`v<x.y.z>` is visible at `https://github.com/<org>/curio/releases/tag/v<x.y.z>`).
- The repo is `PUBLIC`.
- The release notes file (`/tmp/curio-v<x.y.z>-release-notes.md` or `docs/launch/<date>-v<x.y.z>-launch-plan.md`) is finalized.

Do NOT invoke this skill for a pre-release or draft tag; HN and X both punish reposts.

## Skill contract

This skill is **agent-driven, not deterministic** — the curio-rs binary does nothing here. The skill produces:

1. **A pre-launch checklist** the operator works through.
2. **A drafted HN submission** (title + URL + first comment).
3. **A drafted X thread** (3–7 tweets, hook + value prop + CTA).
4. **A draft LinkedIn post** (longer-form, professional context).
5. **A response-plan rubric** (how to engage on HN, what to ignore, what to escalate).

The operator copy-pastes from these drafts and posts manually. **The skill does not auto-post** — HN and X have rate limits, anti-spam heuristics, and platform-specific timing that make automation risky for one-shot launches.

## Inputs

The agent should collect from the operator before drafting:

- **Tag name** (e.g. `v1.0.0`)
- **Repo URL** (e.g. `https://github.com/RyanMerlin/curio`)
- **Release URL** (e.g. `https://github.com/RyanMerlin/curio/releases/tag/v1.0.0`)
- **Single‑sentence value prop** — what does this release let people do that they couldn't do yesterday?
- **3–5 highlight bullets** from the release notes
- **Operator handles** — X handle, HN username (if any), LinkedIn URL
- **Timing constraints** — operator's local timezone, any blackout windows

## Pre-launch checklist

Before any post goes out, the agent walks the operator through:

- [ ] `gh release view <tag>` confirms the release is published (not draft, not pre-release)
- [ ] `gh repo view --json visibility` returns `PUBLIC`
- [ ] README hero image renders on the public repo page (open in a logged-out browser tab)
- [ ] `LICENSE` is present and matches what the release notes claim
- [ ] CI workflow has run at least once on the public commit and is green
- [ ] `git ls-remote origin refs/tags/<tag>` matches `git rev-parse <tag>` locally
- [ ] Spot-check three sentinel files via `gh api repos/<org>/<repo>/contents/<file>` to confirm public presence
- [ ] Spot-check three should-be-secret files (`.env`, `.claude/settings.local.json`, any `*.bak`) return 404
- [ ] A clean `git clone` of the public repo builds and passes the smoke test from a fresh directory
- [ ] Operator has read the drafted posts aloud once for tone

## HN draft template

Hacker News punishes obvious self-promotion and rewards substantive technical narrative. The submission should:

- Title: **`<release-name> – <one-sentence value prop>`** (max 80 chars; HN truncates longer)
  - Avoid hype words ("revolutionary", "game-changing")
  - Avoid the word "we" — solo and small-team projects do better with first-person plural omitted
  - Include the project name once, the value prop once
- URL: the release tag URL, **not** the repo root (HN treats releases as more substantive)
- First comment (post within 60 seconds of submitting): the founder/maintainer's why-now post. 3–5 short paragraphs:
  - Paragraph 1: the editorial-loop pain Curio solves (most knowledge tools index, Curio editorializes)
  - Paragraph 2: the two-layer architecture (deterministic Rust substrate + agent harness) and why that split matters
  - Paragraph 3: what's actually shipped at this release, in plain language
  - Paragraph 4: explicit honesty about what's NOT done yet (T2-D embeddings, etc.) so the comments don't catch you on it
  - Paragraph 5: one technical detail worth discussing (e.g. the seven-dimension proposal scoring, or the publish-time re-gate)

The first comment **should not** be a wall of marketing copy. HN votes signal genuine engagement.

## X / Twitter draft template

A 5-tweet thread, posted as a single drafted reply chain:

1. **Hook tweet** — image (the Curio hero PNG) + one line: what is this + the link
2. **Why it exists** — the editorial-loop framing in plain language
3. **What's in v1.0.0** — three concrete capabilities (multi-tenant, multi-source synthesis, Confluence sync)
4. **The interesting architecture detail** — the agent-vs-deterministic split, or the config-driven product registry as SSOT
5. **CTA** — link to the release notes, invitation to file issues, brief mention of what's coming in v1.1

Keep each tweet under 240 chars to leave room for retweets-with-comment.

## LinkedIn draft template

Longer-form, written in first person, professional context:

- Opening line: the editorial-loop pain in business terms
- 3 paragraphs of substance (architecture, what shipped, what's next)
- Closing: a clear ask (try it, file issues, contribute)
- Hashtags: #knowledgebase #confluence #rust #ai (sparingly; LinkedIn punishes more than 3)

## Response plan (post-publish)

For the first 24 hours:

- **Engage every substantive comment** within 2 hours, even if just to acknowledge.
- **Do not engage with bait** — if a comment is hostile or argumentative without substance, leave it. The crowd self-moderates.
- **Surface real bug reports as GitHub issues** with a one-line acknowledgement in the HN comment thread linking the issue.
- **Track positive engagement signals** in a launch-tracking doc: who starred, who forked, who filed a thoughtful issue. These are early-community seed contacts.

For days 2–7:

- One follow-up post on X with a thread of "what we learned" (specific issues filed, surprising use-case feedback, anything that changed the v1.1 roadmap).
- A short blog post (if the operator runs a blog) consolidating the launch story.
- No new HN submission; HN rate-limits self-promotion across an account.

## Anti-patterns to refuse

The skill should refuse to draft copy that:

- Names companies or customers the operator hasn't explicitly authorized for public reference.
- Makes capability claims beyond what's in the release notes.
- Uses superlatives without evidence ("the best", "the fastest", "the only").
- Uses corporate-PR voice ("we are excited to announce..."). HN and X both punish this.

When the operator pushes for any of the above, the skill should redirect to plain technical voice and concrete capability statements.

## Skill outputs

The agent writes the following artifacts and offers them to the operator:

- `docs/launch/<date>-v<x.y.z>-launch-plan.md` — the full plan with all drafts, kept in the repo as a record
- `docs/launch/<date>-v<x.y.z>-response-log.md` — empty template the operator fills in during the first 24h with engagement notes
- The HN title + URL + first-comment text, ready to copy-paste
- The X thread, ready to copy-paste
- The LinkedIn post, ready to copy-paste
