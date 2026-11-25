---
format: aep.planning-md/3
id: story:cost-spike
kind: story
status: implemented
title: 'Spike: measure the cost of one evaluation'
tags:
- roadmap-step-3
relations:
- decomposes: epic:engine-slices
- depends_on: story:own-generator-smallest
scope:
- confidence: cited
  path: AGENTS.md
- confidence: inferred
  path: docs/cost-spike.md
- confidence: inferred
  path: docs/state-space.md
- confidence: cited
  path: engine/search
- confidence: cited
  path: tools/gate.sh
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:14Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:14Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-11-25T16:47:32Z", actor: "human:operator", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
# Story: Spike: measure the cost of one evaluation

## Outcome

Whether exhaustive search is affordable is known per scope, from a measurement instead of the 50 µs
per layout call `docs/state-space.md` assumes.

## Context

Roadmap step 3. Drive ESS's terminal renderer in process on one example page; time open + N steps +
geometry read; code in Rust, no third-party dependency beyond ESS (`docs/roadmap.md:31-34`).

Surface: not known yet. No Rust workspace exists in this tree; the root `Cargo.toml` is being
introduced by `story:own-generator-smallest`, so this story waits for it (`depends_on`), records
`Cargo.toml` as an inferred surface, and stays out of any concurrent set with another Rust story
until its code home is named.

## Acceptance

A report in the repository gives, for one ESS example page driven in process through ESS's terminal
renderer by a Rust program, the time per UI state (open, N steps, geometry read) and the memory, and
states go or no-go for exhaustive search per scope.

## Provenance

Migrated from `docs/roadmap.md`, step 3, lines 31-34. First written 2026-11-24 · last touched
2026-11-24 · 1 git revision.

## State on 2026-11-24

Exists: `tools/scaffold/cells/` (Rust, the ESS crates in process) measured, at host load about 100:
check a variant 2.6 ms, open a page 1.2 ms, draw one 80x24 state 0.4 ms, about 5 ms per variant
against about 190 ms through the CLI (`docs/scaffold-spike.md`, stage 1; `docs/state-space.md` § 2.3,
which lowers the call budget to about 300,000 calls per 120 s on one core). Run 2 of the docs-landing
campaign: 10.5 ms per variant and size (`docs/campaign-2026-11-24-docs-landing.md`). PR #1, PR #3.

Missing: the time of N steps between states (needs the UI state API, P0 1 of `docs/ess-gaps.md`),
the memory, a measurement on an idle host, and a go or no-go per scope stated as such in one report.

## Scope

Derived 2026-11-25 by `story-scoper`, decided by the coordinating session the same day. Every line is
**cited** (read from the story or the tree), **decided** (a choice the coordinator made, reversible)
or **inferred** (a reading that could be wrong).

- **Primary surface:** `engine/search/`, a new crate `ux-search` (lib, bin `ux-search`, one
  `tests/main.rs`) with its own `[workspace]` table, so the root workspace (`Cargo.toml` members,
  `cargo --workspace --locked` in `tools/gate.sh:37-39`) does not resolve it — decided; precedent
  `tools/scaffold/cells/Cargo.toml` (own `[workspace]`)
- **ESS dependency:** `ess-ui`, `ess-ui-check`, `ess-ui-tui` by git, `https://github.com/beyond10x/ess`,
  tag `0.99.4` (public repository), not by path: a path `../ess` resolves neither in a managed worktree
  nor on the forge — decided; API at the tag: `App::from_text` `app.rs:501`, `open_page` `:681`, `keys`
  `:785`, `advance` `:798`, `regions` `:657` — cited (ESS read, 2026-11-25)
- **Lock file:** `engine/search/Cargo.lock`, committed; the crate builds with `--locked` — decided
- **Gate:** `tools/gate.sh` gains fmt, clippy and test steps with `--manifest-path
  engine/search/Cargo.toml --locked`, so the forge Gate (`.github/workflows/ci.yml` runs `sh
  tools/gate.sh`) runs them — decided
- **Third-party crates:** only through ESS (ratatui 0.29.0, crossterm 0.28.1, clap, serde, serde_json,
  `ess-ui-tui/Cargo.toml`); `AGENTS.md` names this exception for `engine/search` — decided
- **Example page:** ESS `examples/partner-portal/ui.yaml` at tag 0.99.4 (themes light, dark, dense),
  copied with its source tag and SHA-256 under `engine/search/tests/fixtures/` (ESS is Apache-2.0)
  — inferred
- **N steps:** without ESS's UI state API (`docs/ess-gaps.md` P0 1, open in ESS 0.99.4) a step is a key
  replayed from `open`; the report says so — cited (ESS read, 2026-11-25)
- **Documents:** `docs/cost-spike.md` (new), `docs/state-space.md` (the 50 µs assumption, lines 72, 80,
  162) — inferred
- **Confidence:** medium — the ESS API is read at the tag; the cost is not measured yet
- **Would collide with:** `story:toy-climb` (same crate: one unit builds both, this story first), any
  unit on `tools/gate.sh` or `AGENTS.md`
- **Safety fact:** a crate with its own `[workspace]` is outside the root `cargo --workspace`, so no
  existing crate and no existing gate step changes behaviour — unproven
