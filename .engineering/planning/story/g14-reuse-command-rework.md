---
format: aep.planning-md/3
id: story:g14-reuse-command-rework
kind: story
status: implemented
title: Decide the reuse-command rework of the uxsearch guards (G14)
summary: Keep or drop the reuse-command rework, by synthesis time on the ESS release with the G14 fix
relations:
- decomposes: epic:ess-dependencies
scope:
- confidence: inferred
  path: docs/ess-gaps.md
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
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:14Z", actor: "human:operator", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:14Z", actor: "human:operator", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-26T06:46:46Z", actor: "human:operator", revision: 9, decided_on: {"recorded":{"test_result":1}}}
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

## Decision on 2026-11-26

The reuse-command form stays. Measured on 2026-11-26, it synthesizes in under 60 s, the line the
acceptance set.

What changed since the acceptance was written: the branch `spec/g14-reuse-commands` is no longer a
candidate. It was merged by pull request 5 (commit b8d940e), so `main` is the reuse-command form, the
case "the branch is closed unmerged" cannot happen, and the open question became the reverse one:
whether to go back to the guard form. The guard form exists only as the commit before that merge,
with 10 scenarios; no commit holds today's specification (50 scenarios, rounds 2 and 3) in guard
form, so there is no second tree to time against. No ESS release names a fix for G14: the releases
0.99.3 and 0.99.5 name synthesis speed-ups of their own, neither names row-set guards.

Measured: `ess verify conform synthesize --path ess --scenarios ess --suite-format 5`, the released
binaries of ESS 0.99.2 and 0.99.5, one host with 20 CPUs.

| tree | ESS | wall | CPU (user) | host load |
|---|---|---|---|---|
| `main` before G20 was closed (347a098) | 0.99.2 | 23.6 s, 23.7 s | not taken | 9.5, 6.7 |
| the same | 0.99.5 | 24.2 s, 23.2 s | not taken | 5.7, 5.1 |
| the same | 0.99.5 | 48.4 s, 32.3 s | 43.3 s, 31.6 s | 63.8, 43.8 |
| after G20 was closed (17c06a6) | 0.99.5 | 34.6 s, 33.0 s | 33.1 s, 32.2 s | 56.8, 36.1 |
| the same | 0.99.2 | 95.9 s, 89.2 s | not taken | 75.5, 73.9 |
| the same | 0.99.5 | 80.4 s, 85.0 s | not taken | 84.6, 90.1 |

The last two rows are wall time on a host loaded to four times its CPUs and say nothing about the
60 s line; the rows with CPU time were taken alternating between the two trees and show the same
time before and after the ten row sets of G20. The earlier numbers (2 s before the hardening,
about 1000 s and 325 s for the guard form, 11 s and 16 s for the reuse form) are in `ess/GAPS.md`,
G14.

Decision: no return to the guard form. A later change that takes synthesis over 60 s on an idle host
is a defect of that change (`ess/GAPS.md`, G14, keeps the times).
