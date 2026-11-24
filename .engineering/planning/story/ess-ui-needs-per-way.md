---
format: aep.planning-md/3
id: story:ess-ui-needs-per-way
kind: story
status: draft
title: What `ess-ui/1` needs, per way
relations:
- decomposes: epic:generators-and-themes
- depends_on: story:own-generator-smallest
scope:
- confidence: cited
  path: docs/ess-gaps.md
revision: 4
---
# Story: What `ess-ui/1` needs, per way

## Outcome

The ESS maintainers know what `ess-ui/1` needs for the stock way and for an own generator.

## Context

Story 7 of the generator round (`docs/generator-round.md:36-39`), drawn from stories 4 to 6, which
the source now records (`docs/generator-round.md:69-105`). The slot case study already sorts its gap
stories into format gaps and value-rendering gaps for ESS (`docs/generator-round.md:79-86`), and
both ways need the resolved document (P0 3 of `docs/ess-gaps.md`).

Shared surface: `docs/ess-gaps.md`, where this story appends new requirements and
`story:track-ess-gaps` marks gaps closed; each touches only its own lines.

## Acceptance

Each requirement on `ess-ui/1` found by the generator round (extension points, theming, per-node
overrides for the stock way; a resolved, versioned form and a conformance suite for an own generator)
is filed with the ESS maintainers as a story and listed in `docs/ess-gaps.md`.

## Provenance

Migrated from `docs/generator-round.md`, story 7, lines 36-39. Not in Git yet (untracked on
2026-11-24); dates itself `Status: plan, 2026-11-24` (line 3).
