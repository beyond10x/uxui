---
format: aep.planning-md/3
id: story:toy-climb
kind: story
status: draft
title: First climb on a toy page
tags:
- roadmap-step-5
relations:
- decomposes: epic:engine-slices
- depends_on: story:cost-spike
scope:
- confidence: inferred
  path: Cargo.toml
revision: 5
---
# Story: First climb on a toy page

## Outcome

The engine is shown to search end to end: variants, evaluations on layout model v0, a ranked
frontier.

## Context

Roadmap step 5, first half (`docs/roadmap.md:39-40`). Needs the cost measured by `story:cost-spike`
and ESS gaps 1 and 2 (`docs/roadmap.md:30, 62`).

Surface: not known yet; records `Cargo.toml` as an inferred surface until the engine's code home is
named.

## Acceptance

An exhaustive search over two axes (theme × section order) on an ESS example page, evaluated on
layout model v0, prints a ranked frontier as text.

## Provenance

Migrated from `docs/roadmap.md`, step 5, lines 39-40. First written 2026-11-24 · last touched
2026-11-24 · 1 git revision.

## State on 2026-11-24

Exists: no engine. The scaffold spike searched exhaustively on layout model v0 and printed a
ranking: stage 1 (`tools/scaffold/stage1.mjs`, `tools/scaffold/campaign.mjs`, the Rust cell reader
`tools/scaffold/cells/` over the ESS crates) over 144 variants of the lifted docs hero and over 288
variants of the docs landing page (`docs/scaffold-spike.md`,
`docs/campaign-2026-11-24-docs-landing.md`; PR #1, PR #3). Neither is theme x section order on an
ESS example page.

Missing: the engine doing the search; the spike drivers are Node (`docs/scaffold-spike.md`, "Language
of the spike code"). `story:cost-spike`, which this story depends on, is partly met.
