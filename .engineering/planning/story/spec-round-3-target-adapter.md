---
format: aep.planning-md/3
id: story:spec-round-3-target-adapter
kind: story
status: draft
title: 'Specification round 3: the target adapter'
tags:
- roadmap-step-5
relations:
- decomposes: epic:specification
- depends_on: story:spec-hardening-pass
- depends_on: story:spec-round-2-scenarios
scope:
- confidence: cited
  path: ess/domains
revision: 3
---
# Story: Specification round 3: the target adapter

## Outcome

The specification describes targets that are not `ess-ui/1` documents, such as this repository's
documentation site, so the engine can search them.

## Context

`uxsearch` v1 assumes every target is an `ess-ui/1` document on a layout model with no I/O
(`docs/targets.md:3-7`). The target adapter (materialize, render, enumerate; `ess-ui` and `browser`)
is designed in `docs/targets.md:9-30`; the changes per domain are `docs/targets.md:77-85` (`design`,
`journeys`, `evaluation`, `promotion`). It lands after the hardening pass (`docs/targets.md:87`) and
after round 2, whose scenarios cover the same domain files.

## Acceptance

A scenario in `ess/` declares a `Target` with the `browser` adapter (base URL and build command),
varies it by an axis value that patches a named source file, walks a journey of scripted steps,
evaluates it under a key that includes the adapter and its version, and promotes it as a diff per
source file, and `ess specify validate --path ess` exits 0 with that scenario present.

## Out of Scope

The funnel's lift stage and binding table (`specification-blocker:funnel-nouns`).

## Provenance

Migrated from `docs/targets.md`, "What changes in the specification", lines 77-87. Not in Git yet
(untracked on 2026-11-24); dates itself `Status: design, 2026-11-24` (line 3).
