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
revision: 4
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
