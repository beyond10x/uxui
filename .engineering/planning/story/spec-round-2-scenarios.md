---
format: aep.planning-md/3
id: story:spec-round-2-scenarios
kind: story
status: active
title: 'Specification round 2: a scenario for every outcome'
tags:
- roadmap-step-1
relations:
- decomposes: epic:specification
- depends_on: story:spec-hardening-pass
scope:
- confidence: cited
  path: ess/GAPS.md
- confidence: cited
  path: ess/domains
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:15Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:15Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"review_outcome":3}}}
---
# Story: Specification round 2: a scenario for every outcome

## Outcome

Anything generated from the specification is held to scenarios that cover every outcome, and no
unresolved UNMAPPED marker hides in the domains.

## Context

Roadmap step 1. On 2026-11-24 the specification had 156 outcomes without a scenario, 13 UNMAPPED
markers and 11 ESS gaps (`docs/roadmap.md:10-11`). The independent review the step asks for
(design, scope, acceptance) is `reviews/2026-11-24-uxsearch-spec-review.md`; its findings are
`story:spec-hardening-pass`. Unblocks generating anything from the specification
(`docs/roadmap.md:25`).

Shared surface: the domain files of `ess/domains/`, which `story:spec-round-3-target-adapter` also
changes. Round 3 runs after this story (its `depends_on`), so the scenarios cover the v1 outcome set
and round 3 brings scenarios for the outcomes it adds.

## Acceptance

`ess specify validate --path ess --format json` reports 0 unscenarioed outcomes, and every UNMAPPED
marker left in `ess/` is listed in `ess/GAPS.md`.

## Provenance

Migrated from `docs/roadmap.md`, step 1, lines 23-25 ("Scenarios for the 156 outcomes; resolve the 13
UNMAPPED markers or move each to `ess/GAPS.md`"). First written 2026-11-24 · last touched 2026-11-24
· 1 git revision.

## State on 2026-11-24

Exists: ten authored scenarios in `ess/scenarios/` (PR #1, `story:spec-hardening-pass`). The
hardening report (`reviews/2026-11-24-uxsearch-hardening.md`, "Completeness") counts, after the
hardening, 258 unscenarioed constructs (156 before) and 160 of 264 outcomes without a synthesized
scenario. 14 UNMAPPED markers remain in `ess/domains/` (design 4, search 5, metrics 2, promotion 1,
ratchet 1, rules 1); `ess/GAPS.md` mentions UNMAPPED at 5 places.

Missing: 0 unscenarioed outcomes in `ess specify validate --path ess --format json`, and each of the
14 markers listed in `ess/GAPS.md`. Not run for this note: the validate run was stopped after 5
minutes in the build pool; the counts above are the report's.
