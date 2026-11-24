---
format: aep.planning-md/3
id: story:track-ess-gaps
kind: story
status: draft
title: Track the ESS gaps and adopt each release
tags:
- roadmap-step-2
relations:
- decomposes: epic:ess-dependencies
scope:
- confidence: cited
  path: docs/ess-gaps.md
revision: 3
---
# Story: Track the ESS gaps and adopt each release

## Outcome

uxui uses each ESS gap as soon as an ESS release closes it, and drops the workaround.

## Context

Roadmap step 2 (`docs/roadmap.md:26-30`). All 18 gaps of `docs/ess-gaps.md` are filed with the ESS
maintainers (`docs/ess-gaps.md:3-4`); their implementation is ESS's work, not uxui's. The gaps, by
priority:

- P0: 1 UI state API, 2 canonical form and digest (G7), 3 resolved document (F3).
- P1: 4 per-step trace, 5 component harness (G11), 6 chart order (F1), 7 param-scoped live channel
  in tests, 8 a view read through a list field, 8a tab focus order.
- P2: 9 for all over related rows (G4), 10 (G1), 11 (G3), 12 (G2), 13 (G8), 14 (G5), 15 (G6),
  16 (G10), 17 (G9).

Gaps 1 and 2 block `story:toy-climb` (`dependency-blocker:ess-ui-state-api`,
`dependency-blocker:ess-ui-canonical-digest`). Shared surface: `docs/ess-gaps.md`, where
`story:ess-ui-needs-per-way` appends requirements; this story only marks existing entries.

## Acceptance

Every one of the 18 gaps in `docs/ess-gaps.md` is marked closed there, naming the ESS release that
closed it and the uxui change that adopted it.

## Provenance

Migrated from `docs/ess-gaps.md` (lines 1-48) and `docs/roadmap.md` (step 2, lines 26-30).
`docs/ess-gaps.md` is not in Git yet (untracked on 2026-11-24); dates itself `Status: 2026-11-24`
(line 3).
