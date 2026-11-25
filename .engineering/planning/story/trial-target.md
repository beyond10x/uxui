---
format: aep.planning-md/3
id: story:trial-target
kind: story
status: active
title: 'Trial target: an admin console page'
tags:
- roadmap-step-6
relations:
- decomposes: epic:targets
- depends_on: story:toy-climb
scope:
- confidence: inferred
  path: docs/targets.md
- confidence: inferred
  path: tools/scaffold/campaign.mjs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:16Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:16Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
---
# Story: Trial target: an admin console page

## Outcome

The engine is tried on a product UI it was not built around, and the generic findings come back
here.

## Context

Roadmap step 6 (`docs/roadmap.md:43-45`): one page or component of an admin console generated from
an `ess-ui/1` document, in a private repository whose fixtures make it I/O-free. This public
repository does not name it. Results stay in that repository; only generic findings come back here.

## Acceptance

A campaign of the engine on one page or component of the admin console, run in that repository on
its fixtures, ends with a ranked frontier.

## Provenance

Migrated from `docs/roadmap.md`, step 6, lines 43-45. First written 2026-11-24 · last touched
2026-11-24 · 1 git revision.
