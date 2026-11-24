---
format: aep.planning-md/3
id: story:scaffold-spike
kind: story
status: implemented
title: 'Scaffold spike: lift a page section into an `ess-ui/1` document'
tags:
- in-flight
relations:
- decomposes: epic:engine-slices
- depends_on: story:docs-site
scope:
- confidence: cited
  path: tools/scaffold
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T10:50:06Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-24T10:50:06Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-24T10:50:06Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
# Story: Scaffold spike: lift a page section into an `ess-ui/1` document

## Outcome

It is known whether a section of a real page can be lifted into an intermediate `ess-ui/1` document
(the scaffold) that the engine can search without a browser.

## Context

Under way on 2026-11-24 in a separate worktree (`uxui-scaffold`), whose untracked `tools/scaffold/`
holds the spike's code. The funnel's stage 0 (`docs/targets.md:40-42`); "when the scaffold moves,
the page moves" (`docs/targets.md:47-49`). It reads the documentation landing page that
`story:docs-site` produces. The spike's own acceptance is not written down anywhere in this tree:
the statement below is inferred from the lift-fidelity check (`docs/targets.md:53-54`).

## Acceptance

A report states, for one section of the documentation landing page, whether a lifted `ess-ui/1`
document passes `ess ui check` and, rendered, reproduces the section's nodes, order, actions and
names as the page shows them.

## Out of Scope

The lift stage as a product with a binding table (`specification-blocker:funnel-nouns`).

## Provenance

Migrated from `docs/targets.md`, the funnel, lines 32-56. Not in Git yet (untracked on 2026-11-24);
dates itself `Status: design, 2026-11-24` (line 3). Acceptance: inferred. Surface `tools/scaffold/`:
read from the spike's worktree, not from a source document.
