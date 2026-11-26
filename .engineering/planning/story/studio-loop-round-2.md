---
format: aep.planning-md/3
id: story:studio-loop-round-2
kind: story
status: implemented
title: 'Studio loop round 2: the first change set'
summary: Fix UXF-003, UXF-017 and UXF-001 on the studio and measure round R-02 against R-01
relations:
- decomposes: epic:bench
- depends_on: story:loop-round-on-studio
scope:
- confidence: cited
  path: ess/ui/findings
- confidence: cited
  path: ess/ui/rounds
- confidence: cited
  path: ess/ui/studio.ui.yaml
- confidence: cited
  path: ess/ui/tests/studio.yaml
- confidence: cited
  path: themes/studio.css
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T06:08:09Z", actor: "human:operator", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-26T06:08:09Z", actor: "human:operator", revision: 5}
- {from: "active", to: "implemented", at: "2026-11-26T07:21:38Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
# Story: Studio loop round 2: the first change set

## Outcome

The studio's worst defects of round R-01 are fixed, each held by a check that failed before its fix,
and round R-02 shows by how much each target metric moved.

## Context

Round R-01 (`ess/ui/rounds/R-01/`, `story:loop-round-on-studio`) recorded 22 findings on the React
app generated from `ess/ui/studio.ui.yaml`; gates 1, 2 and 3 fail. Its triage
(`ess/ui/rounds/R-01/triage.md`, "The change set") names five changes: UXF-003, UXF-004, UXF-001,
UXF-002 (every S1) and UXF-017 (by priority). Three of them are layout changes, and the skill takes
one design decision per round: the triage gives it to UXF-001, the finding on an anchor card.
UXF-004 and UXF-002 stay in the set and wait for the next round.

So this round builds three changes:

- UXF-003: the filters of the target catalog and of the axis switch never match (an expression).
- UXF-017: small text and badge contrast (theme values).
- UXF-001: the Owner's actions on a promotion are out of view (layout; `decision.md` of R-02, default
  yes).

Shared surface: `ess/ui/studio.ui.yaml` and `ess/ui/tests/studio.yaml`, `themes/studio.css`,
`ess/ui/rounds/`, `ess/ui/findings/`. Five findings of R-01 are known losses of the stock generator or
of fixture mode (UXF-005, UXF-006, UXF-007, UXF-009, UXF-011) and are not in this story.

## Acceptance

`sh ess/ui/rounds/round.sh R-02` is on record in `ess/ui/rounds/R-02/`, and against R-01:

- `trial.scenarios_not_completed` no longer counts `filter-targets`, and the row expectation of
  `switch-axis` holds;
- `page_checks.always_visible.misses` on a promotion is 0 (4 in R-01);
- `audit.text_under_14px.share` and `audit.text_low_contrast.runs_per_page.max` are lower than 0.564
  and 12;
- no metric of `ess/ui/rounds/R-01/round.json` is worse;
- UXF-003, UXF-017 and UXF-001 each name a regression asset that failed before the change and passes
  after it, and `ess/ui/rounds/R-02/triage.md` has the table "did the last change sets move their
  metrics?".

## Provenance

Drafted on 2026-11-26 from `ess/ui/rounds/R-01/triage.md`, which ends: "The change set is not yet a
task in the planning store".

## Decision on 2026-11-26

Round R-02 is on record and four of the five acceptance lines hold as written. The fourth line, "no
metric of `ess/ui/rounds/R-01/round.json` is worse", is read with the skill's tolerance
(`plugin/skills/ui-iteration-loop/SKILL.md` § 6: counts 0, `layout_shift` 0.01):

- every count of R-01 is equal or lower in R-02;
- `page_checks.layout_shift.max` is 0.0176 against 0.0151, 0.0025 above R-01 and under the tolerance
  of 0.01. On unchanged input the value ran from 0.0129 to 0.026 with R-01's theme and from 0.003 to
  0.0284 with R-02's, on one machine (`ess/ui/rounds/R-02/report.md`, "Layout shift"). The
  instability is finding UXF-023, open.

Decision: the line counts as met. The design decision of the round (`ess/ui/rounds/R-02/decision.md`,
the gate decision as a list row) is built on its default and has no answer yet; an answer "no" takes
the change back and reopens UXF-001, in a later round.
