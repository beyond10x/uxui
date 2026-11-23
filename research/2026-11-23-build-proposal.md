# uxui: what to build next, and how it sits on ratchet

2026-11-23. Proposal for discussion. Sources: ratchet `docs/design.md` and `vision.md` (scaffold worktree,
design phase), this repo's research and reviews, the R-01 round on the explainer page.

## Layers

| layer | owns | format |
|---|---|---|
| ESS | what the UI is: nodes and stable paths (`ess-ui/1`), behaviour tests (`ess-ui-test/1`) | ESS |
| ratchet | generic quality: requirement, assessment plan, evidence record, gate, protected reference, ratchet policy, "insufficient evidence", policy from a trusted revision | `ratchet/1` |
| uxui | what UX quality means for a UI and how to measure it: a preset family, assessors that emit ratchet evidence, the round procedure | ratchet presets + assessors + skill |

uxui does not build a baseline file, a ratchet or a gate evaluator. The 7 copies in the org
(modelspec, slot x2, runbook, conductor x2, substrate) each have one; ratchet replaces all of them.

## What uxui ships

1. **Preset family `web-ux@1`** (ratchet presets), each bound to `ess-ui/1` node paths, not CSS:
   - `no-overflow` (no horizontal document scroll, no element past its box), absolute limit 0
   - `always-visible` (declared node paths 100 % in viewport, not covered) at every state change
   - `completion-visible` (`completion_miss` = 0, `false_complete` = 0)
   - `screen-only-completes` (a deterministic walker finishes every journey by role and name)
   - `keyboard-parity` (every journey also by keyboard alone)
   - `a11y-rules` (ACT-style rule outcomes; `cantTell` maps to ratchet's "insufficient evidence", never pass)
   - `task-effort` (actions per journey, glance cost) with a protected reference
   - `content-true` (rendered text of declared nodes equals the declared source; closes the "false
     statement" gap that triage could not rank)
2. **Assessors** that return ratchet evidence records:
   - `ux-static@1`: reads the `ess-ui/1` document only. Shortest action path per journey, keyboard
     reachability, accessible names. No browser, deterministic, no noise. Runs first.
   - `ux-walk@1`: runs `ess-ui-test/1` journeys in a browser at declared viewports and themes, records
     geometry, focus, actions and rule outcomes per step. Observation conditions (engine, fonts, viewport,
     queue time excluded) go into the evidence, so ratchet can refuse incomparable runs.
3. **The skill**, thinned to the round: observe, file findings, turn each finding into a ratchet
   requirement (its assessment is the regression test), build failing-first, `ratchet evaluate --gate merge`,
   `ratchet reference propose` for improvements. Triage and design decisions stay judgement, labelled so.

## How ratchet closes items of the critique

| critique item | ratchet mechanism |
|---|---|
| 7 lucky noise becomes the floor | the reference stays fixed between explicit adoptions; adoption needs repeated comparable evidence |
| 15 agent edits its own gate | policy comes from a trusted revision; a threshold change is a policy change |
| 4 one renderer, fonts | observation conditions and resolved target in every evidence record; incompatible = insufficient |
| R-01 queue time in wall time | the same: queue wait is not run time, an incomparable run is insufficient, not red |
| 3 axe "needs review" | "insufficient evidence" is never a pass |
| R-01 axe counted 12 times | findings keep stable identity (rule + node path), "no new findings" by identity, not counts |

## Gaps to raise with ratchet (seen in R-01)

1. **Trade-offs.** The navigation made 20 values worse and 1 much better; it passed only by a decision id.
   Ratchet has per-requirement references; it needs a declared trade-off (requirement group, or an
   authorized reference change that names the gain it buys).
2. **Scope of a result.** A page-level result (an a11y rule) must not be summed per journey.
3. **Noisy measures.** Median-of-N and a declared sample count for timing measures, kept apart from exact counts.
4. **Finding identity for UI**: rule id + `ess-ui` node path + viewport/theme.

## Order (goal first)

1. Agree the split with the ratchet session: `web-ux@1` presets + `ux-walk` evidence as ratchet's
   "ESS UI behavior" integration (its first milestone, design.md § 7). uxui is its first consumer.
2. `ux-static@1` on an `ess-ui/1` document (no browser): cheap, deterministic, proves the evidence shape.
3. `ux-walk@1` on one real consumer: migrate one of the 7 copies (smallest first; candidates `runbook/ui/ux`
   or `slot/ui/ux`) and delete its private baseline and ratchet.
4. Thin the skill to the round on top.

## Zero dependencies, staged

- Now: Playwright + axe-core inside `ux-walk` only, behind the evidence format.
- Next: own ACT rules over the accessibility tree (replaces axe; the rule set is small and declared).
- Later: own Chrome DevTools Protocol driver in Rust (replaces Playwright). The evidence format does not
  change, so gates and references survive each swap; a swap is a new evidence series per ratchet's rule.

## Not machine-checkable, kept as judgement

Aesthetics, focus, "one emphasis per region" beyond proxies, comprehension. The colleague's own data: the
loop climbs effort metrics but does not improve design. These stay with people and review agents; their
output enters only as findings that become requirements.
