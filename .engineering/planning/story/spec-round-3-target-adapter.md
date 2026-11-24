---
format: aep.planning-md/3
id: story:spec-round-3-target-adapter
kind: story
status: implemented
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
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T13:14:48Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-24T13:14:48Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-24T13:14:48Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
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

## Result

The acceptance scenario is `ess/scenarios/browser-target-from-patch-to-promotion.yaml`. Before the
constructs existed, `ess specify validate --path ess` refused it with 27 scenario refusals (no
`adapter` on `BaseDocument`, `Target` or `LayoutModel`, no `browser` input on `DeclareTarget`, no
`patches` on `AddAxisValue`, no `script` on `DeclareJourney`, no `RecordSourceDiff` and no
`SourceDiffsOfPromotion`). With them it answers `uxsearch v1 — 10 file(s), 12 scenario(s), valid`.

What changed, per domain of `docs/targets.md:77-85`:

- `design`: `AdapterKind` (`ess-ui`, `browser`); a base document and a target name their adapter; a
  browser target holds its base URL and build command (`BrowserTarget`); an axis value holds
  `SourcePatch`es (a JSON pointer or a CSS variable in a named file) beside operator applications.
- `journeys`: a journey has either a model (`test_file`, `test_name`) or a `script` of steps.
- `evaluation`: `LayoutModel` is one adapter's renderer and names its adapter, so the key, which
  names the renderer version, holds the adapter and its version.
- `promotion`: `SourceDiff`, one per source file, recorded by `RecordSourceDiff`.

The funnel's lift and binding table, out of scope above, were taken into this round at the
operator's brief (`specification-blocker:funnel-nouns`, cleared): scenario
`ess/scenarios/lift-must-reproduce-the-component.yaml`.

One existing scenario changed: `evaluation-key-matches-its-inputs.yaml` gained `adapter: ess-ui` on
its arranged layout model, the field every renderer now carries. The other nine are unchanged and
valid. Open questions are the `UNMAPPED:` markers in the changed domain files.
