---
format: aep.planning-md/3
id: story:docs-site
kind: story
status: implemented
title: The documentation website
tags:
- in-flight
relations:
- decomposes: epic:targets
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: README.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: website
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T10:49:00Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"review_outcome":3,"verification":1}}}
- {from: "proposed", to: "active", at: "2026-11-24T10:49:00Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"review_outcome":3,"verification":1}}}
- {from: "active", to: "implemented", at: "2026-11-24T10:50:06Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
# Story: The documentation website

## Outcome

uxui has a public documentation site, the first real target of the engine.

## Context

Under way on 2026-11-24 in a separate worktree (`uxui-docs-site`): a Docusaurus site under
`website/` with its landing page generated from `website/product.json` (`b10x-product-landing/1`)
and styled by CSS (`docs/targets.md:5-6`). Nothing in this tree states the site as a plan item; the
acceptance is inferred from `docs/targets.md:5-6`.

Shared surface: that worktree also edits `Taskfile.yml`, `AGENTS.md` and `README.md`;
`story:own-generator-smallest` edits `Taskfile.yml` and `AGENTS.md` in its worktree too.

## Acceptance

`website/` builds into a static site whose landing page shows the content of `website/product.json`.

## Provenance

In flight outside this worktree; described in `docs/targets.md:5-6` (untracked on 2026-11-24,
`Status: design, 2026-11-24`). Acceptance: inferred. Shared paths read from the worktrees' status.
