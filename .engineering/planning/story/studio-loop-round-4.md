---
format: aep.planning-md/3
id: story:studio-loop-round-4
kind: story
status: implemented
title: 'Studio loop round 4: the live climb fits the viewport, structured values as text, a stable layout shift'
summary: Fix UXF-026 and UXF-005 and make the layout shift of UXF-023 stable on the studio, and measure round R-04 against R-03
relations:
- decomposes: epic:bench
- depends_on: story:studio-loop-round-3
scope:
- confidence: cited
  path: docs/ess-gaps.md
- confidence: cited
  path: docs/own-generator.md
- confidence: cited
  path: docs/studio-climb.md
- confidence: cited
  path: engine/search/tests
- confidence: cited
  path: ess/ui/findings
- confidence: cited
  path: ess/ui/rounds
- confidence: cited
  path: ess/ui/studio.ui.yaml
- confidence: cited
  path: ess/ui/studio.uxui.yaml
- confidence: cited
  path: ess/ui/tests/studio.yaml
- confidence: cited
  path: generators/own
- confidence: cited
  path: themes/studio.css
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T19:42:29Z", actor: "human:operator", revision: 3, correlation: "D20"}
- {from: "proposed", to: "active", at: "2026-11-26T19:42:29Z", actor: "human:operator", revision: 4, correlation: "D20"}
- {from: "active", to: "implemented", at: "2026-11-26T20:20:16Z", actor: "human:operator", revision: 8, decided_on: {"recorded":{"test_result":2}}}
---
# Story: Studio loop round 4: the live climb fits the viewport, structured values as text, a stable layout shift

## Outcome

The live climb fits the gating viewports, the maps of the studio read as words, the layout shift of
the start form is one number over repeated runs, and round R-04 shows by how much each target metric
moved against R-03.

## Context

Round R-03 (`ess/ui/rounds/R-03/`, `story:studio-loop-round-3`) left 20 findings open, none of them S1;
gates 1 and 3 fail. The owner's decision D20 of 2026-11-26 (taken on its default A) orders round R-04
on the studio with three findings, in this order:

- UXF-026 (S2): the live climb is 1249 px tall in a viewport of 768 or 720 px, so gate 1 fails on
  it and the step table and "Frontier now" open below the fold. Fitting it is a layout change: the
  design decision of this round (`decision.md` of R-04), its default built.
- UXF-005 (S2): structured values are drawn as raw JSON. R-03 built the studio's handling for the
  lists of a promotion's decision; R-03's triage names the next part, the maps (the budget of a
  campaign, the first route of a variant, the diff of a promotion), as one field per member. The
  lists of records that are left need a field path through a list, which ESS has not decided on
  (LR-D1, `docs/ess-gaps.md`): the studio adds no such path, and what the round would need is
  recorded there.
- UXF-023 (S3): the layout shift of the start form changes from run to run on unchanged input
  (0.003 to 0.0284 over R-02 and R-03), so "no metric worse" cannot be held for it. The round makes
  repeated runs agree, by fixing the measurement or the page, and says which.

R-03's decision (`ess/ui/rounds/R-03/decision.md`) waits for the owner and is not answered here.

Shared surface: `ess/ui/studio.ui.yaml`, `ess/ui/tests/studio.yaml`, `ess/ui/studio.uxui.yaml`,
`themes/studio.css`, `ess/ui/rounds/`, `ess/ui/findings/`, `docs/ess-gaps.md`. The terminal drawing of
the live climb feeds the golden file `engine/search/tests/fixtures/studio.frontier.txt`: a change of
that page's sections or fields generates it again with the command its test names.

## Acceptance

`sh ess/ui/rounds/round.sh R-04` is on record in `ess/ui/rounds/R-04/`, and against R-03:

- the live climb fits both gating viewports in both themes: no document scroll and no scrolling
  element on it, held by a page bar of `ess/ui/rounds/studio.audit.json` that is missed before the
  change; `audit.pages_with_document_scroll` 1 (2 in R-03) (UXF-026);
- `page_checks.overflow.blocks` is 0 (14 in R-03): no map is drawn as JSON, held by expectations of
  the scenario cards that are missed before the change (UXF-005);
- the layout shift after ready is measured in 5 runs of the page checks, and no page's values
  differ between runs by more than the skill's tolerance of 0.01
  (`page_checks.layout_shift.spread`); `report.md` says whether the page or the measurement was
  fixed (UXF-023);
- `decision.md` of R-04 compares A and B in R-03's form, with a recommendation, a written default and
  "Answer: none yet"; the default is what is built;
- no metric of `ess/ui/rounds/R-03/round.json` is worse beyond the skill's tolerance;
- each closed finding names a regression asset that failed before the change and passes after it,
  and `ess/ui/rounds/R-04/triage.md` has the table "did the last change sets move their metrics?".

## Provenance

Drafted on 2026-11-26 from the owner's decision D20 (ordered round R-04 with UXF-026, UXF-005 and
UXF-023; taken on its default A) and `ess/ui/rounds/R-03/triage.md`.

## Decision on 2026-11-26

Round R-04 is on record in `ess/ui/rounds/R-04/` (`round.json`; R-03's input measured again with this
round's assets in `before.round.json`). Against the acceptance:

- the live climb fits both gating viewports in both themes: met. The page bars of the live climb in
  `ess/ui/rounds/studio.audit.json` (`document_overflow` 0, `scrollable_elements` 0) are missed in 4
  of 4 cells on R-03's input and in 0 after (`audit.cells_over_a_page_bar`);
  `audit.pages_with_document_scroll` 1 (2 in R-03) (UXF-026, closed).
- `page_checks.overflow.blocks` 0 (14 in R-03): met. The new no-JSON expectations of `live-climb`,
  `switch-axis`, `compare-and-judge` and `owner-trade-off` are missed on R-03's input and met in 4 of
  4 runs after; the terminal test "the budget reads as its members" fails on R-03's document
  (UXF-005, open for the lists of records, which need a field path through a list: none added,
  `docs/ess-gaps.md`).
  The paths through an object reach the repository's own generator too (`generators/own`): it reads
  the member and checks the first member against the view, held by the test "a path through an
  object binds through its first member"; `docs/own-generator.md` names the difference to ESS.
- the layout shift in 5 runs, no page's values more than 0.01 apart: met,
  `page_checks.layout_shift.spread` 0.0024 (0.0121 on R-03's input); the start form 0 in 5 of 5 runs.
  `ess/ui/rounds/R-04/report.md` says the page was fixed, not the measurement (added after the
  round's merge; until then this line was not met as written). The statement is also in
  `ess/ui/findings/UXF-023.md` ("Closed in R-04: the page was fixed, and the measure is taken five
  times") and in `triage.md`: the page was fixed (UXF-023, closed).
- `decision.md` compares A and B in R-03's form, recommendation B, default B, "Answer: none yet";
  B is built: met.
- no metric of R-03 worse beyond the tolerance: met. Every count is equal or better;
  `page_checks.layout_shift.max` 0.0284 to 0.0075.
- each closed finding names a regression asset that failed before and passes after (UXF-026,
  UXF-023, UXF-022), and `triage.md` has the table "did the last change sets move their metrics?":
  met.

The story is closed on that reading. What it leaves: UXF-005 for the lists of records; gate 1 on the
start form (UXF-012) and on the tables of a variant and a comparison (UXF-005); the answers to the
decisions of R-03 and R-04.
