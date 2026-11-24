---
format: aep.planning-md/3
id: story:ux-static-slice
kind: story
status: active
title: Thin vertical slice `ux-static@1`
tags:
- roadmap-step-4
relations:
- decomposes: epic:engine-slices
scope:
- confidence: inferred
  path: Cargo.toml
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T00:39:01Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-24T00:39:01Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
---
# Story: Thin vertical slice `ux-static@1`

## Outcome

The evidence contract with ratchet is proven on static metrics, before any browser or layout model.

## Context

Roadmap step 4 (`docs/roadmap.md:35-38`). Inputs: an `ess-ui/1` document, its `ess-ui-test/1`
journeys and a `uxui-declarations/1` file. Output: ratchet evidence records in ratchet's draft
shape.

Surface: not known yet. No Rust workspace exists in this tree; this story records `Cargo.toml` as an
inferred surface and stays out of any concurrent set with another Rust story until its code home is
named.

## Acceptance

Given an `ess-ui/1` document, its `ess-ui-test/1` journeys and a `uxui-declarations/1` file,
`ux-static@1` writes ratchet evidence records, in ratchet's draft shape, for required nodes present,
actions per journey and accessible names, without a browser or a layout model.

## Provenance

Migrated from `docs/roadmap.md`, step 4, lines 35-38. First written 2026-11-24 · last touched
2026-11-24 · 1 git revision.
