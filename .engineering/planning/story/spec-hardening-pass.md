---
format: aep.planning-md/3
id: story:spec-hardening-pass
kind: story
status: implemented
title: Specification hardening pass
tags:
- in-flight
- roadmap-step-1
relations:
- decomposes: epic:specification
scope:
- confidence: cited
  path: ess
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T00:03:19Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T00:03:19Z", actor: "human:operator", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-24T00:39:28Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
# Story: Specification hardening pass

## Outcome

The specification no longer admits the paths the independent review found: a frontier entry on empty
or partial evidence, a trade-off accepted on insufficient evidence, and an evaluation computed over
unevaluable states.

## Context

Under way on 2026-11-24 in this worktree (branch `spec/harden`): `ess/` holds uncommitted edits to
every domain, the studio document, its fixtures and tests. It feeds on
`reviews/2026-11-24-uxsearch-spec-review.md`: 37 findings (R1-R37; 3 blockers, 19 major, 15 minor,
line 8) and ten hardening targets where a planted defect would most likely survive (lines 416-440).
Round 3 lands after this pass (`docs/targets.md:87`).

## Acceptance

Every finding R1-R37 of `reviews/2026-11-24-uxsearch-spec-review.md` is either fixed in `ess/` or
listed in `ess/GAPS.md` under its id, and `ess specify validate --path ess` exits 0.

## Out of Scope

Scenarios for every outcome (`story:spec-round-2-scenarios`); the target adapter
(`story:spec-round-3-target-adapter`).

## Provenance

Migrated from `reviews/2026-11-24-uxsearch-spec-review.md` (verdict line 8, hardening targets lines
416-440). Not in Git yet (untracked on 2026-11-24); dates itself `Date: 2026-11-24` (line 3).
Status read from the worktree, not from the review: uncommitted edits under `ess/` on 2026-11-24.
