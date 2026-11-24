---
format: aep.planning-md/3
id: epic:specification
kind: epic
status: draft
title: 'ESS specification `uxsearch`: hardening, round 2, round 3'
tags:
- roadmap-step-1
revision: 3
---
# Epic: ESS specification `uxsearch`: hardening, round 2, round 3

## Outcome

The specification `uxsearch` in `ess/` is sound enough to generate from: the independent review's
findings are closed or recorded as gaps, every outcome has a scenario, and the specification
describes targets that are not `ess-ui/1` documents.

## Why Now

Specification first, code second (`docs/roadmap.md:3-4`). Round 2 unblocks generating anything from
the specification (`docs/roadmap.md:23-25`). The review of 2026-11-24 found 3 blockers, 19 major and
15 minor findings, three of which break the invariant that insufficient evidence never passes
(`reviews/2026-11-24-uxsearch-spec-review.md:8-13`).

## Scope

`ess/`: the domains, the studio `ess-ui/1` document with its fixtures and tests, and `ess/GAPS.md`.
Stories, in order: `story:spec-hardening-pass`, `story:spec-round-2-scenarios`,
`story:spec-round-3-target-adapter`.

## Out of Scope

What ESS itself cannot express goes to `epic:ess-dependencies`. Styling is solved in `themes/`,
never by extending `ess-ui/1` (`AGENTS.md:35-38`).

## Risks

- Layout model v0 has cells, not pixels: overflow, clipping and density are coarse until a pixel
  model exists (`docs/roadmap.md:61`).
- The metrics rank what is declared; aesthetics and focus stay judgement (`docs/roadmap.md:63-64`).

## Done When

`ess specify validate --path ess` exits 0 with no unscenarioed outcome, and the target adapter of
`docs/targets.md:77-85` is part of the specification.

## Provenance

Migrated from `docs/roadmap.md` (step 1, lines 23-25), `docs/targets.md` (lines 77-87) and
`reviews/2026-11-24-uxsearch-spec-review.md`.

- `docs/roadmap.md`: first written 2026-11-24 · last touched 2026-11-24 · 1 git revision; its
  working copy carries uncommitted edits of 2026-11-24, and the line numbers cited are of that copy.
- `docs/targets.md` and the review: not in Git yet (untracked on 2026-11-24). The targets document
  dates itself `Status: design, 2026-11-24` (line 3); the review `Date: 2026-11-24` (line 3).
