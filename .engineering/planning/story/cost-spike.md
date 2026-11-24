---
format: aep.planning-md/3
id: story:cost-spike
kind: story
status: draft
title: 'Spike: measure the cost of one evaluation'
tags:
- roadmap-step-3
relations:
- decomposes: epic:engine-slices
- depends_on: story:own-generator-smallest
scope:
- confidence: inferred
  path: Cargo.toml
revision: 5
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
