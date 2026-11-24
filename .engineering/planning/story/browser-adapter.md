---
format: aep.planning-md/3
id: story:browser-adapter
kind: story
status: draft
title: Browser adapter
tags:
- roadmap-step-5
relations:
- decomposes: epic:engine-slices
- depends_on: story:spec-round-3-target-adapter
- depends_on: story:docs-site
scope:
- confidence: inferred
  path: Cargo.toml
revision: 4
---
# Story: Browser adapter

## Outcome

The engine can evaluate any page with a URL in real pixels, with a headless browser as its renderer.

## Context

Roadmap step 5 (`docs/roadmap.md:41-42`); designed in `docs/targets.md:23-26`: materialize writes the
variant's files and builds or serves them; render gives boxes, computed styles, the accessibility
tree and a screenshot at each viewport and mode. Built before the cell model is extended, because it
reuses the original loop's proven measurements and gives the bench a referee
(`docs/targets.md:91-93`). Needs the adapter in the specification
(`story:spec-round-3-target-adapter`) and, for its acceptance, the documentation landing page
(`story:docs-site`).

Surface: not known yet; records `Cargo.toml` as an inferred surface until the engine's code home is
named.

## Acceptance

Run on the documentation landing page at one declared viewport and mode, the browser adapter writes
one evidence record holding, for each declared node, its box, its computed styles and its
accessibility-tree entry, plus a screenshot, and naming the adapter and its version.

## Provenance

Migrated from `docs/roadmap.md` (lines 41-42, an uncommitted edit of 2026-11-24) and
`docs/targets.md` (lines 23-30; untracked on 2026-11-24, `Status: design, 2026-11-24`).
