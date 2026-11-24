---
format: aep.planning-md/3
id: epic:engine-slices
kind: epic
status: draft
title: 'Engine slices: cost spike, `ux-static@1`, first climbs, browser adapter, funnel'
tags:
- roadmap-step-3
revision: 4
---
# Epic: Engine slices: cost spike, `ux-static@1`, first climbs, browser adapter, funnel

## Outcome

The UX search engine exists as code in thin vertical slices: its per-state cost is measured, its
static metrics write ratchet evidence, it climbs a toy page and the studio on layout model v0, and a
browser adapter lets it climb any page with a URL.

## Why Now

The first code is a thin vertical slice that measures the assumption the whole design rests on
(`docs/roadmap.md:3-4`): `docs/state-space.md` assumes 50 µs per layout call and nothing has
measured it (`docs/roadmap.md:31-34`).

## Scope

Roadmap steps 3 to 5 (`docs/roadmap.md:31-42`), the target adapter and the funnel of
`docs/targets.md:9-56`. Code in Rust, no third-party dependency beyond ESS (`docs/roadmap.md:33-34`;
`AGENTS.md:30-33`). Decisions already taken: layout model v0 is ESS's terminal renderer, UX
declarations live in a `uxui-declarations/1` file (`docs/roadmap.md:15-16`); contrast is APCA beside
WCAG 2.x in an own implementation of the published formula (`docs/roadmap.md:17-19`).

`story:check-tool-internals-research` is carried in from outside the migrated documents: it is under
way in its own worktree and serves the direction of owning every tool (`AGENTS.md:30-33`); no
sentence of the roadmap asks for it.

No Rust workspace exists in this tree yet. The root `Cargo.toml` is being introduced by
`story:own-generator-smallest` in its worktree, so every engine story records `Cargo.toml` as an
inferred surface until its code home is named, and `story:cost-spike`, the first of them, waits for
that story.

## Out of Scope

The documentation site and the trial target as targets (`epic:targets`); ratchet's gates
(`epic:ratchet-integration`). Held until their nouns have an ESS home
(`specification-blocker:funnel-nouns`): funnel stage 0 (lift) as a product, stage 2 (project the top
variants onto the real page through the binding table) and stage 3 (judge side-by-side screenshots of
stage 2's variants) (`docs/targets.md:40-45`).

## Risks

- No study shows a computed metric orders close variants of one design as people do; picking the
  best of thousands is best-of-n optimisation. The roadmap's counter, which every campaign story
  here carries: tie bands, the Pareto frontier, prefer the smallest change, and a small human
  calibration set per campaign before a ranking is trusted (`docs/roadmap.md:56-60`).
- Exhaustive search depends on ESS's UI state API and canonical digest (`docs/roadmap.md:30, 62`;
  `dependency-blocker:ess-ui-state-api`, `dependency-blocker:ess-ui-canonical-digest`) and on the
  cost measured by `story:cost-spike`.
- Layout model v0 has cells, not pixels (`docs/roadmap.md:61`); a low stage agreement shows the cell
  model misleads on a target (`docs/targets.md:55-56`).

## Done When

The engine climbs the studio on layout model v0 and a page with a URL on the browser adapter, each
writing evidence that names its adapter and version (`docs/targets.md:28-30`).

## Provenance

Migrated from `docs/roadmap.md` (steps 3-5, lines 31-42; risks, lines 56-62) and `docs/targets.md`
(lines 9-56).

- `docs/roadmap.md`: first written 2026-11-24 · last touched 2026-11-24 · 1 git revision; lines
  17-19, 41-42 and 56-60 are uncommitted edits of 2026-11-24.
- `docs/targets.md`: not in Git yet (untracked on 2026-11-24); dates itself `Status: design,
  2026-11-24` (line 3).
