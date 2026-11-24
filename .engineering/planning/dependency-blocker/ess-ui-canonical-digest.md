---
format: aep.planning-md/3
id: dependency-blocker:ess-ui-canonical-digest
kind: dependency-blocker
status: open
title: ESS has no canonical form or digest of a UI state
relations:
- blocks: story:toy-climb
revision: 1
---
# Blocker: ESS has no canonical form or digest of a UI state

## What Is Blocked

`story:toy-climb`: "two routes to one document are one variant" and the evaluation cache rest on it
(`docs/ess-gaps.md:15-17`; `docs/roadmap.md:30`).

## What Would Clear It

An ESS release with a normative canonical byte form and digest of an `ess-ui/1` document and of a UI
state (gap 2, P0, G7; roadmap step 2 (b)).

## Who Can Clear It

The ESS maintainers; filed with them on 2026-11-24 (`docs/ess-gaps.md:3-4`).

## What We Are Doing Meanwhile

The specification states the invariant by construction (`ess/GAPS.md`, G7).
`story:track-ess-gaps` adopts the release that closes this.
