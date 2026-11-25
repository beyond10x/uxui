---
format: aep.planning-md/3
id: story:studio-climb
kind: story
status: active
title: Climb the studio
tags:
- roadmap-step-5
relations:
- decomposes: epic:engine-slices
- depends_on: story:toy-climb
- depends_on: story:studio-stock-generator
scope:
- confidence: cited
  path: ess/ui/studio.ui.yaml
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:15Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:15Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
---
# Story: Climb the studio

## Outcome

The engine climbs its own studio, the first target with real journeys.

## Context

Roadmap step 5, after the toy: "Then the studio, generated from `ess/ui/studio.ui.yaml`"
(`docs/roadmap.md:40`). The studio is also the bench's first target (`docs/comparison.md:22-23`).
Shared surface: it reads `ess/ui/studio.ui.yaml`, which `story:spec-hardening-pass` and
`story:studio-stock-generator` rewrite; it runs after the second (`depends_on`).

## Acceptance

The engine runs a campaign on `ess/ui/studio.ui.yaml` on layout model v0 and prints its ranked
frontier.

## Provenance

Migrated from `docs/roadmap.md`, step 5, line 40. First written 2026-11-24 · last touched 2026-11-24 ·
1 git revision.
