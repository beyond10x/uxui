---
format: aep.planning-md/3
id: story:point-old-skill-copies-here
kind: story
status: active
title: Point the older copies of the skill here
relations:
- decomposes: epic:skill
scope:
- confidence: inferred
  path: .claude-plugin/marketplace.json
- confidence: cited
  path: docs/roadmap.md
- confidence: inferred
  path: plugin/.claude-plugin/plugin.json
- confidence: inferred
  path: plugin/.codex-plugin/plugin.json
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:15Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:15Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
---
# Story: Point the older copies of the skill here

## Outcome

The skill has one source, and the older copies stop drifting from it.

## Context

A risk of the roadmap: two older copies of the skill exist outside this repository; until they point
here, they drift (`docs/roadmap.md:65`).

## Acceptance

Both older copies of the skill outside this repository load or link this repository's
`ui-iteration-loop` skill instead of a text of their own.

## Provenance

Migrated from `docs/roadmap.md`, "Risks", line 65. First written 2026-11-24 · last touched
2026-11-24 · 1 git revision. A risk, not a step: the story is inferred from it.
