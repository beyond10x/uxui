---
format: aep.planning-md/3
id: story:thin-the-skill
kind: story
status: active
title: Thin the skill to a round on top of the engine
tags:
- roadmap-step-8
relations:
- decomposes: epic:skill
- depends_on: story:ux-static-slice
- depends_on: story:ratchet-presets-and-gates
- depends_on: story:loop-process-improvements
scope:
- confidence: cited
  path: plugin/skills/ui-iteration-loop/SKILL.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:16Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:16Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
---
# Story: Thin the skill to a round on top of the engine

## Outcome

A person running the skill gets the engine's findings as ratchet requirements and keeps design
decisions and taste.

## Context

Roadmap step 8 (`docs/roadmap.md:49-50`). Only a person accepts a trade-off
(`docs/roadmap.md:15-16`). Shared surface: `plugin/skills/ui-iteration-loop/SKILL.md`, which
`story:loop-process-improvements` changes first (`depends_on`).

## Acceptance

In a round the skill runs on the generated studio app, every finding the engine reports becomes a
ratchet requirement record.

## Provenance

Migrated from `docs/roadmap.md`, step 8, lines 49-50. First written 2026-11-24 · last touched
2026-11-24 · 1 git revision.
