---
format: aep.planning-md/3
id: story:migrate-smallest-ui-loop-copy
kind: story
status: draft
title: Migrate the smallest UI-loop copy onto ratchet and uxui
tags:
- roadmap-step-7
relations:
- decomposes: epic:ratchet-integration
- depends_on: story:ratchet-presets-and-gates
revision: 2
---
# Story: Migrate the smallest UI-loop copy onto ratchet and uxui

## Outcome

One of the organization's UI-loop copies stops keeping its own baseline file.

## Context

Roadmap step 7 (`docs/roadmap.md:47-48`): seven UI-loop copies exist, each with its own baseline
file; the smallest goes first. They live in other repositories, which this public repository does
not name. "Smallest" as the copy with the shortest baseline file is inferred.

## Acceptance

The UI-loop copy with the shortest baseline file runs a round on ratchet and uxui after its own
baseline file has been removed.

## Provenance

Migrated from `docs/roadmap.md`, step 7, lines 47-48. First written 2026-11-24 · last touched
2026-11-24 · 1 git revision.
