---
title: The loop
sidebar_position: 1
description: Rounds, levels, hard gates, findings and ratchets of the ui-iteration-loop skill.
lede: Two loops run side by side; a round with people at most twice a week, and Build and Gate on every merge, hermetic and without API keys.
source: plugin/skills/ui-iteration-loop/SKILL.md
---

# The loop

Two loops run side by side. The **round loop** runs all six steps with people, at most twice a
week. The **fast loop** runs only Build and Gate, on every merge, hermetic, with no API keys.

Four rules hold on the loop's edges: a gate pass lands; a gate fail is fixed and retried; every
finding becomes a test; the baseline never gets worse and gains one test per finding.

## A round

```text
 Trial ──▶ Measure ──▶ Triage ──▶ Design decision ──▶ Build ──▶ Gate
   ▲                                                              │
   └──────────── next round reports each change's metric ◀────────┘
```

| step | who | what comes out |
|---|---|---|
| Trial | people, or a driver bot | three recorded sessions from scenario cards: two fixed anchors, one rotating card; a 90-second questionnaire after each |
| Measure | a deterministic round command, then two fresh review agents | `report.md`, `round.json`, screenshots, a trend row, two review files |
| Triage | the agent, by rule | a change set of at most 5 changes, ordered by priority = severity × frequency × confidence ÷ cost; every S1 finding is in it |
| Design decision | a person | at most one yes-or-no question per round, with a default if unanswered |
| Build | the build agent | each finding's regression test written and run red, then the change makes it green |
| Gate | every merge | the hard gates and the ratchets below |

At most 5 changes per round, so a metric movement can be attributed to a change. A change that has
not moved its target metric after two rounds is kept with a written reason or reverted.

## Levels

| level | proves | runs |
|---|---|---|
| L0 engine scenarios | the logic behind the UI, and a guidance cue that follows its truth | every merge |
| L1 unit | rendering, moments, the metric functions | every merge |
| L2 layout contract | hard gates 1, 2, 3 and 5 at every gating viewport and theme, over every state change | every merge |
| L3 screen-only user | the bot completes every scenario | every merge |
| L4 replayed real sessions | verdicts and metrics over 3 replays, against the baseline | per round, paid |
| L5 discoverability probe | a vision model, given only the screenshot, says what to do next | per round, paid |
| L6 live rounds | people run the scenario cards | per round |
| L7 heuristic review | two fresh review agents | per round |
| L8 real users | five users per study, think-aloud | per milestone |

L0 to L3 are hermetic and gate every merge. L4 and L5 are paid and run only with an operator's
approval, never in a repository's offline gate.

## Gates

Every merge passes six hard gates:

1. **No scroll** at the gating viewports, in every theme, over every replay frame. On a docs site:
   no horizontal page scroll and no element running past its box.
2. **Always visible.** The adapter's always-visible elements are 100 % in the viewport, not clipped
   or covered, at every state change.
3. **Unmissable completion.** `completion_miss` = 0 and `false_complete` = 0 on every scenario.
4. **Screen-only user** completes every scenario to its truth, each fix in at most 2 actions.
5. **Accessibility.** axe-core reports 0 serious or critical violations in every theme; the task can
   be done by keyboard alone.
6. **Cue truth**, only with a guidance cue: never ahead of the truth, no flip-flops, `cue_accuracy`
   at least 95 %.

## Ratchets

The baseline is a committed `ux/baseline.json` with one value and tolerance per scenario and metric,
all lower-is-better. Counts have tolerance 0, steps 5 %. The ratchet command fails a merge on:

- a value worse than the baseline beyond its tolerance;
- an improvement not written into the baseline in the same merge;
- a missing value;
- a baseline made worse without a decision id.

## Findings

Every finding is a record `ux/findings/UXF-nnn.md` with a class, a severity (S1 to S3), the metric
it moves, its evidence and a **regression asset**: an engine scenario, a Playwright assertion or a
checklist line. No S1 or S2 finding closes without an asset that fails before the fix and passes
after. The same failure three times stops patching: diagnose first.

## The screen-only user

Deterministic code, no model. It uses only what is visible: controls fully in the viewport, not
covered at their centre, found by accessible role and name. A click or a key press is one action.
It follows the guidance cue, fixes what is flagged in at most 2 actions, steers a wrong cue back in
one, and presses the completion control, or the screen must say why it cannot. Its counts give the
actions per fix, the glance cost and the time to act in steps.

The full rules, including the adapter slots, the triage formula and the review agents, are in the
[skill reference](../reference/skill.md).
