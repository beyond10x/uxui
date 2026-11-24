---
format: aep.planning-md/3
id: dependency-blocker:ess-ui-state-api
kind: dependency-blocker
status: open
title: ESS has no UI state API
relations:
- blocks: story:toy-climb
revision: 2
---
# Blocker: ESS has no UI state API

## What Is Blocked

`story:toy-climb`: exhaustive search over a UI state space is impossible without it
(`docs/ess-gaps.md:12-14`; `docs/roadmap.md:30, 62`).

## What Would Clear It

An ESS release that gives, for a UI state of an `ess-ui/1` document, the list of actions available in
it and a cloneable state or a replay-free way to reach it (gap 1, P0, of `docs/ess-gaps.md`; roadmap
step 2 (a)).

## Who Can Clear It

The ESS maintainers; filed with them on 2026-11-24 (`docs/ess-gaps.md:3-4`).

## What We Are Doing Meanwhile

`story:cost-spike` and `story:ux-static-slice` need neither the state API nor a digest.
`story:track-ess-gaps` adopts the release that closes this.
