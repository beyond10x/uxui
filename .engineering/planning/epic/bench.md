---
format: aep.planning-md/3
id: epic:bench
kind: epic
status: draft
title: 'The bench: the engine against the loop it replaces'
revision: 2
---
# Epic: The bench: the engine against the loop it replaces

## Outcome

A bench of targets with planted defects says, per defect class, whether the engine finds and fixes
more real UX defects than the `ui-iteration-loop` skill, for less time and money, judged by a referee
neither tool optimises against (`docs/comparison.md:9-16`).

## Why Now

The loop runs today, so its baseline can be measured before any engine slice exists
(`docs/comparison.md:57-60`).

## Scope

`docs/comparison.md`: the bench (lines 18-31), the measures (33-42), the referee (44-55), results as
ratchet evidence (65-69), dogfooding on the studio (71-75).

## Out of Scope

Held until the bench's nouns have an ESS home (`specification-blocker:bench-nouns`): the seeded
bench and the loop's baseline on it, `ux-static@1` on the bench, the layout model's rank correlation
against the browser, the head-to-head, and results as ratchet evidence.

## Risks

The original loop's own data shows hermetic metrics did not separate 28 design variants
(`docs/roadmap.md:63-64`).

## Done When

Each run of either tool writes ratchet evidence with the bench version and the measures, and a new
engine version may not lower recall on any defect class (`docs/comparison.md:67-69`).

## Provenance

Migrated from `docs/comparison.md`. Not in Git yet (untracked on 2026-11-24); dates itself `Status:
design, 2026-11-24` (line 3).
