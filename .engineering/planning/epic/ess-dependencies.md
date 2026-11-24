---
format: aep.planning-md/3
id: epic:ess-dependencies
kind: epic
status: draft
title: 'ESS dependencies: gaps uxui needs closed upstream'
tags:
- roadmap-step-2
revision: 2
---
# Epic: ESS dependencies: gaps uxui needs closed upstream

## Outcome

Every gap uxui found in ESS is closed upstream and adopted here, in the order that unblocks the
engine first.

## Why Now

The owner, 2026-11-24: "make sure that all ESS gaps are implemented ASAP" (`docs/ess-gaps.md:3-4`).
Gaps 1 and 2 decide whether exhaustive search is possible at all (`docs/roadmap.md:30`).

## Scope

The 18 gaps of `docs/ess-gaps.md` (P0: 3, P1: 6, P2: 9), filed with the ESS maintainers. The work
on them is in the ESS repository; the work here is tracking each release and adopting it
(`story:track-ess-gaps`). The two P0 gaps that block a story here are blockers:
`dependency-blocker:ess-ui-state-api` and `dependency-blocker:ess-ui-canonical-digest`.

## Out of Scope

Styling is not an ESS gap: themes, templates and the UX declarations belong to uxui
(`docs/ess-gaps.md:5-6`). That is why roadmap step 2 (e), "UX declarations as constructs (proposal
from `uxui-declarations/1`)" (`docs/roadmap.md:29`), is not among the 18 gaps: the later
`docs/ess-gaps.md:5-6` keeps the UX declarations in uxui, and this migration carries no story for
(e).

## Risks

Until a gap closes, the specification states its invariant only by construction (P2,
`docs/ess-gaps.md:38-48`).

## Done When

Every gap of `docs/ess-gaps.md` is closed by an ESS release and adopted here.

## Provenance

Migrated from `docs/ess-gaps.md` and `docs/roadmap.md` (step 2, lines 26-30).

- `docs/ess-gaps.md`: not in Git yet (untracked on 2026-11-24); dates itself `Status: 2026-11-24`
  (line 3).
- `docs/roadmap.md`: first written 2026-11-24 · last touched 2026-11-24 · 1 git revision.
