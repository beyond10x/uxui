---
format: aep.planning-md/3
id: story:check-tool-internals-research
kind: story
status: draft
title: 'Research: how deterministic UI check tools work inside'
tags:
- in-flight
relations:
- decomposes: epic:engine-slices
scope:
- confidence: cited
  path: research/2026-11-24-check-tool-internals.md
revision: 4
---
# Story: Research: how deterministic UI check tools work inside

## Outcome

For each deterministic UI check tool, uxui knows what owning that check on a declared model takes,
in line with the direction of building its own tooling (`AGENTS.md:30-33`).

## Context

Under way on 2026-11-24 in a separate worktree (`uxui-research`), as
`research/2026-11-24-check-tool-internals.md`. It goes past the earlier note
`research/2026-11-23-external-machine-checkable-ux.md`. No source in this tree states it as a plan
item; it is carried into `epic:engine-slices` from outside, and the acceptance is inferred from the
note's sections in that worktree.

## Acceptance

`research/2026-11-24-check-tool-internals.md` has, for each of axe-core, the Lighthouse accessibility
and best-practices audits, Playwright layout assertions, pixel diff, rrweb recording and replay, the
screen-only bot pattern, WCAG 2.x and APCA contrast, and tab order and focus checks, a section on how
it works inside and one on owning it on a declared model.

## Provenance

In flight outside this worktree; named by the brief that migrated this backlog, not by a document in
this tree. Acceptance: inferred.
