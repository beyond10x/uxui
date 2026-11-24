---
format: aep.planning-md/3
id: story:stock-or-own-decision
kind: story
status: implemented
title: Stock generator or our own, per UI
relations:
- decomposes: epic:generators-and-themes
- depends_on: story:own-generator-smallest
scope:
- confidence: cited
  path: docs/generator-round.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T10:50:06Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-24T10:50:07Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-24T10:50:07Z", actor: "human:operator", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
# Story: Stock generator or our own, per UI

## Outcome

Each UI of the round is built the way that costs less to adjust, and the reason is written down.

## Context

The round's standing question (`docs/generator-round.md:107-111`), and the standing rule of
`AGENTS.md:40-43`: reassess at any time, and record the reason. While this store was being created
the source recorded both decisions: slot chose its own generator after 45 minutes on the stock one
(`docs/generator-round.md:91-92`), and the studio goes to an own generator under two conditions
(held to `ess/ui/tests/studio.yaml` through a browser run; ESS ships the resolved document and a
conformance suite), with the adjustment costs that decided it (`docs/generator-round.md:96-105`).
The acceptance below reads as met by those lines; the story stays a draft because the migration
moves nothing on its own say-so, and it stays open as the place where a reassessment lands.

## Acceptance

`docs/generator-round.md` records, for the studio and for slot's web UI, stock generator or own
generator, with the adjustment costs that decided it.

## Provenance

Migrated from `docs/generator-round.md`, "Standing question", lines 107-111 (lines 69-73 when first
read on 2026-11-24). Not in Git yet (untracked on 2026-11-24); dates itself `Status: plan,
2026-11-24` (line 3).
