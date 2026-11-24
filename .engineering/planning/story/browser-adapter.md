---
format: aep.planning-md/3
id: story:browser-adapter
kind: story
status: implemented
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
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T15:24:07Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":2,"review_outcome":7}}}
- {from: "proposed", to: "active", at: "2026-11-24T15:24:07Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"test_result":2,"review_outcome":7}}}
- {from: "active", to: "implemented", at: "2026-11-24T15:24:07Z", actor: "human:operator", revision: 8, decided_on: {"recorded":{"test_result":2,"review_outcome":7}}}
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

## State on 2026-11-24

Exists: spike code in Node, `tools/scaffold/` (PR #1, PR #3). `cdp.mjs` drives Chromium over the
DevTools protocol; `probe.mjs` writes a `uxui-scaffold/1` JSON of a page (regions, sections, boxes);
stage 2 of `campaign.mjs` (and `stage2.mjs`, `sweep.mjs`) measures variants of the documentation
landing page in six environments and takes screenshots (`docs/campaign-2026-11-24-docs-landing.md`,
`docs/scaffold-spike.md`).

Missing: the evidence record of the acceptance, holding per declared node its box, computed styles
and accessibility-tree entry, a screenshot, and the adapter's name and version; and the adapter as
engine code rather than spike code (`docs/scaffold-spike.md`, "Language of the spike code").
