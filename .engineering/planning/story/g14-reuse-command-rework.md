---
format: aep.planning-md/3
id: story:g14-reuse-command-rework
kind: story
status: active
title: Decide the reuse-command rework of the uxsearch guards (G14)
summary: Keep or drop the reuse-command rework, by synthesis time on the ESS release with the G14 fix
relations:
- decomposes: epic:ess-dependencies
scope:
- confidence: inferred
  path: ess/GAPS.md
- confidence: cited
  path: ess/components.yaml
- confidence: cited
  path: ess/domains/evaluation.yaml
- confidence: cited
  path: ess/domains/journeys.yaml
- confidence: cited
  path: ess/domains/promotion.yaml
- confidence: cited
  path: ess/scenarios/browser-target-from-patch-to-promotion.yaml
- confidence: cited
  path: ess/scenarios/evaluation-key-matches-its-inputs.yaml
- confidence: cited
  path: website/docs/reference/specification.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:14Z", actor: "human:operator", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:14Z", actor: "human:operator", revision: 4}
---
# Story: Decide the reuse-command rework of the uxsearch guards (G14)

## Outcome

The uxsearch specification either takes the reuse-command rework on this branch or drops it, decided by
measured conformance synthesis time on the ESS release that ships the G14 fix.

## Context

Conformance synthesis of the hardened specification takes about 1000 s on ESS 0.98.0 (ess/GAPS.md G14).
ESS diagnosed the cause as exponential work in row-set selectors and put the fix in its P0 wave.

The rework on branch `spec/g14-reuse-commands` removes the row-set collision selectors from the request
commands instead: a stored key is never requested again (`already-requested` refuses with
`EvaluationExists` or the state-space equivalent), and two new commands, `ReuseEvaluation` and
`ReuseStateSpace`, answer from a stored key or refuse a collision. Files: `ess/components.yaml`,
`ess/domains/evaluation.yaml`, `ess/domains/journeys.yaml`, `ess/domains/promotion.yaml`,
`ess/scenarios/evaluation-key-matches-its-inputs.yaml`. `ess specify validate`: valid, 10 scenarios.

## Acceptance

- Synthesis time of `main` and of this branch is measured on the ESS release with the G14 fix, both
  numbers in this story.
- If `main` synthesizes in under 60 s, the branch is closed unmerged and this story records why.
- Otherwise the branch merges after the hardening scenarios still catch every planted defect
  (24 of 24 today).

## Decision on 2026-11-24

Measured on the shared build node with the ESS release of that day: conformance synthesis of this
branch 11 s; of main about 1000 s (`ess/GAPS.md` G14). No ESS release fixes G14 yet. Taken: merge the
rework now (reversible by a revert). When ESS ships the G14 fix, synthesis of the guard form is measured
again and the form with the clearer specification is kept.
