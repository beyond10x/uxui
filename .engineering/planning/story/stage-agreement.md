---
format: aep.planning-md/3
id: story:stage-agreement
kind: story
status: implemented
title: Stage agreement on the first funnel campaign
relations:
- decomposes: epic:engine-slices
- depends_on: story:browser-adapter
- depends_on: story:toy-climb
scope:
- confidence: inferred
  path: Cargo.toml
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T10:50:07Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-11-24T10:50:07Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
- {from: "active", to: "implemented", at: "2026-11-24T10:50:07Z", actor: "human:operator", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
---
# Story: Stage agreement on the first funnel campaign

## Outcome

A campaign that ranks on the cell model and then measures in a browser says how far the two agree,
so a misleading cell model is visible in its evidence.

## Context

The funnel's second honesty check (`docs/targets.md:55-56`), between stage 1 (brute force,
`ess-ui` adapter) and stage 2 (project, `browser` adapter) (`docs/targets.md:43-44`). Stage 2
projects through the binding table, which is held (`specification-blocker:funnel-nouns`), so this
story is blocked by it. The source asks for the check on every campaign; this story builds it on the
first one, and the check then runs on every later campaign.

Surface: not known yet; records `Cargo.toml` as an inferred surface until the engine's code home is
named.

## Acceptance

The first campaign that runs both stage 1 and stage 2 on the documentation landing page records in
its evidence the rank correlation between the two stages' orderings of the variants both evaluated.

## Provenance

Migrated from `docs/targets.md`, lines 51-56. Not in Git yet (untracked on 2026-11-24); dates itself
`Status: design, 2026-11-24` (line 3). "The first campaign" and "rank correlation" as the measure:
inferred (the source says "compared"; `docs/comparison.md:53-55` uses rank correlation for the same
kind of check).
