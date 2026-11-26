---
format: aep.planning-md/3
id: story:trial-target
kind: story
status: implemented
title: 'Trial target: an admin console page'
tags:
- roadmap-step-6
relations:
- decomposes: epic:targets
- depends_on: story:toy-climb
scope:
- confidence: inferred
  path: AGENTS.md
- confidence: cited
  path: docs/roadmap.md
- confidence: inferred
  path: docs/targets.md
- confidence: cited
  path: docs/trial-2026-11-26-outside-target.md
- confidence: cited
  path: engine/browser/src/session.rs
- confidence: inferred
  path: engine/search/Cargo.toml
- confidence: inferred
  path: engine/search/src/campaign
- confidence: inferred
  path: engine/search/src/lib.rs
- confidence: inferred
  path: engine/search/src/main.rs
- confidence: inferred
  path: engine/search/src/probes/campaign.js
- confidence: cited
  path: engine/search/src/profile.rs
- confidence: inferred
  path: engine/search/tests/campaign.rs
- confidence: cited
  path: engine/search/tests/campaign_live.rs
- confidence: inferred
  path: engine/search/tests/fixtures/campaign
- confidence: inferred
  path: engine/search/tests/main.rs
- confidence: cited
  path: tools/studio-campaign.sh
- confidence: inferred
  path: website/data/status.json
- confidence: inferred
  path: website/docs/roadmap.md
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:16Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:16Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-26T10:00:59Z", actor: "human:operator", revision: 15, decided_on: {"recorded":{"test_result":2,"review_outcome":2}}}
---
# Story: Trial target: an admin console page

## Outcome

The engine is tried on a product UI it was not built around, and the generic findings come back
here.

## Context

Roadmap step 6 (`docs/roadmap.md:43-45`): one page or component of an admin console generated from
an `ess-ui/1` document, in a private repository whose fixtures make it I/O-free. This public
repository does not name it. Results stay in that repository; only generic findings come back here.

## Acceptance

A campaign of the engine on one page or component of the admin console, run in that repository on
its fixtures, ends with a ranked frontier.

## Provenance

Migrated from `docs/roadmap.md`, step 6, lines 43-45. First written 2026-11-24 · last touched
2026-11-24 · 1 git revision.

## Result on 2026-11-26

Unit 1 (2026-11-26, second wave): `ux-search campaign`, the campaign on a declaration file, proven
on the generated studio.

Unit 2 (third wave): one campaign on one list page of the admin console, run in that repository on
its fixture build, served on a loopback port, under the conditions given with the wave's decision
D14: fixture data only, nothing committed there, nothing of the target in this repository. It ended
with a ranked frontier: 32 variants, 192 renders, a frontier of 6, the page as shipped at rank 28 of
32, two runs with the same bytes of measurements. The first run measured the page with a navigation
open under the resting pointer. The tool was changed for that (the pointer is parked; a mode's
`expect` is read back; an axis that changes nothing is named; notes behind the counts; `--help`; the
profile directory), and the second run, with the changed engine, measured the page at rest and ended
with the same six variants.

Unit 3: the generic findings are `docs/trial-2026-11-26-outside-target.md`.

The acceptance is met by the second run. What the trial leaves is in two drafts:
`story:campaign-terms-for-list-pages` and `story:campaign-run-follow-ups`.
