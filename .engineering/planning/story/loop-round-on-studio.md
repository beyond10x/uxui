---
format: aep.planning-md/3
id: story:loop-round-on-studio
kind: story
status: implemented
title: 'Dogfooding: a loop round on the studio'
relations:
- decomposes: epic:bench
- depends_on: story:studio-stock-generator
scope:
- confidence: cited
  path: ess/ui
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:15Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:15Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-26T05:55:40Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
# Story: Dogfooding: a loop round on the studio

## Outcome

The studio's UX defects found by today's loop are on record, before the engine can look.

## Context

`docs/comparison.md:71-75`: the studio's `ess-ui/1` document is generated into a React app on
fixtures, and the loop runs a round on it now; findings on the studio go into `ess/ui/` like any
other target's.

Shared surface: `ess/ui/`, which `story:spec-hardening-pass` and `story:studio-stock-generator`
change; this story runs after the second (`depends_on`), so it reviews the fixed studio.

## Acceptance

The `ui-iteration-loop` skill runs one round on the React app generated from
`ess/ui/studio.ui.yaml` with fixtures, and each of its findings is recorded in `ess/ui/`.

## Provenance

Migrated from `docs/comparison.md`, "Dogfooding", lines 71-75. Not in Git yet (untracked on
2026-11-24); dates itself `Status: design, 2026-11-24` (line 3).
