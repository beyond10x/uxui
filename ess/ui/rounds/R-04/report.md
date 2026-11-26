# R-04: the live climb fits the viewport, the maps read in words, the layout shift holds still

The three changes the owner ordered for this round (decision D20) are built. The live climb fits both
gating viewports: one page of ten scrolls where two did. No map is drawn as JSON any more, and no
block runs past its box (14 in R-03). The start form no longer moves as it loads, and the layout
shift, measured five times per round from now on, differs between runs by 0.0024 at most (0.0121 on
R-03's input). Gates 1 and 3 still fail. Three findings are closed: UXF-026, UXF-023 and UXF-022. None
is new. 17 findings are open, none of them S1.

No person and no paid level took part. Numbers are in `round.json`, R-03's input measured again in
`before.round.json`, the design decision in `decision.md`, the order of work in `triage.md`, the
findings in `../../findings/`.

## What changed

| # | finding | change | where |
|---|---|---|---|
| 1 | UXF-026 the live climb is taller than the viewport | from 78rem of width the main column is two rows: the KPIs beside the chart of the best value, the step table beside "Frontier now"; the side column 11rem, the chart 7rem high; the progress chart not drawn there; the step table last at every width. The yes side of `decision.md` | `themes/studio.css` |
| 2 | UXF-005 structured values as JSON, the maps | the budget, the first route of a variant and the diff of a promotion as one field per member (`budget.max_variants`, `first_route.origin`, `diff.changed`, ...) | `ess/ui/studio.ui.yaml` |
| 3 | UXF-023 the start form moves as it loads | the target holds its height from the first frame, the three fields with suggestions hold one row of them, a form's loading bar lies over its top edge | `themes/studio.css` |

UXF-023 was fixed in the page, not in the measurement: the ready condition and the settle time are
those of R-01 to R-03. What changed in the measurement is that it is taken five times
(`round.sh`, step 3a), so that a value that changes from run to run shows as such
(`page_checks.layout_shift.spread`).

The change of UXF-026 also closes UXF-022: the step table, the one section that grows while a
campaign runs, is the last one, so a new step moves nothing under it.

Regression assets added, all in the round:

- `studio.audit.json`, the set `page_bars`: the live climb's `document_overflow` (new in the audit
  cell: the document's height past the viewport's) and `scrollable_elements` at most 0, counted by
  `summary.mjs` into the new metric `audit.cells_over_a_page_bar` (UXF-026);
- `round.sh` runs the page checks four more times (`UXUI_ROUND_REPEATS`, `repeat/`), and
  `summary.mjs` writes every page's layout shift per run (`page_checks_repeat`) and the new metrics
  `page_checks.layout_shift.max_of_runs` and `page_checks.layout_shift.spread` (UXF-023, UXF-022);
- `studio.scenarios.json`: `live-climb` expects the budget as two fields and no `{"` in the summary
  (its expectation on the KPI values of "Frontier now", JSON in every round so far, is taken out:
  that is a list of records, below); `switch-axis`, `compare-and-judge` and `owner-trade-off` expect
  no `{"` in the record of a variant, of both sides of a comparison and of a promotion (UXF-005);
- `ess/ui/tests/studio.yaml`: "the budget reads as a map and the KPI values as a list" is now "the
  budget reads as its members and the KPI values as a list", and fails on R-03's document (UXF-005).

Outside the round's own files: the journey of that test in `ess/ui/studio.uxui.yaml` and its constant
in `engine/static/tests/main.rs` carry the new name and the new fields; the golden file
`engine/search/tests/fixtures/studio.frontier.txt` is generated again (the summary of the live climb
has one field more in the terminal too), and the test that read the ranking as a tie on `cut` now
reads it as domination (`docs/studio-climb.md` has the new ranking). `docs/ess-gaps.md` records the
five paths through an object and what would need a path through a list.

## What ran

As in R-03, one command: `sh ess/ui/rounds/round.sh R-04`, now with the page checks run five times.

| step | result |
|---|---|
| page checks, first run | 890 checks: 840 passed, 0 failed, 50 measured without a threshold |
| page checks, four more runs | the layout shift of every page per run (`page_checks_repeat`) |
| render records | 40 records and screenshots |
| trial | 11 scenarios, 44 runs, 144 screenshots |
| audit | 40 audits |

The same command ran on the document and theme of R-03 with this round's assets, as `R-04-red`: its
numbers are `before.round.json` (890 checks: 826 passed, 14 failed). A first run of R-04 is not kept:
the card `live-climb` expected the budget's seconds as "3600", and the React app draws "3,600"; the
card was corrected and the round ran again. Its numbers were the same except the layout shift of the
live climb, 0.0075 in all five runs (0.0051 to 0.0075 in the kept run).

## Where it was measured

On the machine of R-03, with Chrome for Testing 159.0.8154.6 (R-03: 158.0.8127.18) and the same ess
0.99.2, node, react, esbuild, typescript and ux-browser 0.11.0. Inter is installed and drawn, as in
R-03 (`versions.txt`). R-03's input measured again gives every count of R-03 and the same largest
layout shift, 0.0284.

## Adapter

As in R-03, with three additions: the page bars (the live climb: no document scroll, no scrolling
element), five runs of the page checks, and the new expectations above.

| slot | value |
|---|---|
| scenario cards | `ess/ui/rounds/studio.scenarios.json`, 11 cards |
| anchor cards | `stop-campaign`, `owner-trade-off` |
| gating viewports | 1366x768, 1280x720 |
| measured only | 1366x657, 1536x754, 960x1080 |
| themes | light, dark |
| always visible | navigation, theme toggle, stop campaign, campaign state, gate, owner action |
| audit bars | text under 14 px: 0; low-contrast text: 0 |
| audit floors | titles (`h1`): at least 1 per page |
| audit page bars | the live climb: `document_overflow` 0, `scrollable_elements` 0 (new) |
| page check runs | 5 (new) |
| notices | no notice contains "uxsearch." |
| guidance cue | none |
| baseline | none: R-03's `round.json` is what this round is read against |

## Hard gates

| gate | result | measured |
|---|---|---|
| 1 no scroll | fail | the document scrolls on 1 of 10 pages (the start form, 1077 px in 768; UXF-012); elements scroll on 2 (a variant and a comparison: their tables of JSON values); 0 blocks overflow. The live climb passes (it failed in R-03) |
| 2 always visible | pass | 0 misses |
| 3 unmissable completion | fail | 4 of 11 scenarios are not completed; no run fails outright |
| 4 screen-only user | not run | not built |
| 5 accessibility | not measured | axe-core is not installed. Keyboard: 0 misses, every Tab stop marked. Audit: 0 text runs under 4.5:1 |
| 6 guidance cue | dropped | the studio has no guidance cue |

## Metrics

Lower is better for every one. "Again" is R-03's document and theme measured in this round
(`before.round.json`), with this round's regression assets.

| metric | R-03 | again | R-04 | against R-03 |
|---|---|---|---|---|
| `page_checks.page_scroll.total` | 0 | 0 | 0 | equal |
| `page_checks.overflow.blocks` | 14 | 14 | 0 | better |
| `page_checks.always_visible.misses` | 0 | 0 | 0 | equal |
| `page_checks.keyboard.misses` | 0 | 0 | 0 | equal |
| `page_checks.layout_shift.max` | 0.0284 | 0.0284 | 0.0075 | better |
| `page_checks.layout_shift.max_of_runs` (new) | not counted | 0.0284 | 0.0075 | |
| `page_checks.layout_shift.spread` (new) | not counted | 0.0121 | 0.0024 | |
| `audit.pages_with_document_scroll` | 2 | 2 | 1 | better |
| `audit.pages_with_scrollable_element` | 2 | 2 | 2 | equal |
| `audit.pages_without_h1` | 0 | 0 | 0 | equal |
| `audit.text_under_14px.share` | 0 | 0 | 0 | equal |
| `audit.text_low_contrast.runs_per_page.max` | 0 | 0 | 0 | equal |
| `audit.controls_unnamed.max` | 1 | 1 | 1 | equal |
| `audit.inputs_placeholder_only.pages` | 4 | 4 | 4 | equal |
| `audit.tab_stops_unmarked.max` | 0 | 0 | 0 | equal |
| `audit.pages_tab_starts_outside_navigation` | 10 | 10 | 10 | equal |
| `audit.cells_over_a_bar` | 0 of 80 | 0 of 80 | 0 of 80 | equal |
| `audit.cells_under_a_floor` | 0 of 40 | 0 of 40 | 0 of 40 | equal |
| `audit.cells_over_a_page_bar` (new) | not counted | 4 of 8 | 0 of 8 | |
| `trial.scenarios_not_completed` | 4 of 11 | 6 of 11 | 4 of 11 | equal |
| `trial.runs_failed` | 0 of 44 | 0 of 44 | 0 of 44 | equal |
| `trial.runs_expectation_missed` | 16 | 24 | 16 | equal |
| `trial.runs_notice_expectation_missed` | 28 of 44 | 28 of 44 | 28 of 44 | equal |
| `trial.looks.total` | 8 | 8 | 4 | better |
| `trial.actions.max` | 10 | 10 | 10 | equal |

No metric is worse than R-03. "Again" misses more expectations than R-03 because it holds R-03's
document to this round's new ones: `switch-axis` and `compare-and-judge` miss their no-JSON
expectations there, and `live-climb` its three on the budget.

## Layout shift

The value `round.json` keeps is the first of five runs; all five per page are in
`page_checks_repeat`.

| input | the start form, 5 runs | the live climb, 5 runs |
|---|---|---|
| R-03's document and theme | 0.0284, 0.0176, 0.0176, 0.0176, 0.0284 | 0.013, 0.0009, 0.0009, 0.013, 0.013 |
| R-04 | 0, 0, 0, 0, 0 | 0.0075, 0.0064, 0.0075, 0.0051, 0.0075 |

The start form moved three times as it loaded (its record, UXF-023, has the sources); none of them is
left. The live climb's 0.013 was the step table growing by a live step at 960x1080 and moving
"Frontier now" (UXF-022); that viewport reads 0.0004 now. What is left on the live climb, at
1366x768, 1280x720 and 1366x657, is the head of the step table: "frontier size" breaks onto a second
line once the rows arrive and the table sets its widths, and the rows move 17 px down. It is the
largest value of the round and is 0.0024 apart over five runs, under the tolerance of 0.01.

## Scenarios

| scenario | card | R-03 | R-04 | what happened in R-04 |
|---|---|---|---|---|
| stop-campaign | anchor | completed | completed | 2 actions |
| stop-campaign-keyboard | rotating | completed | completed | 10 key presses |
| start-campaign | rotating | completed | completed | 10 actions, 1 look for the button below the fold (UXF-012) |
| compare-and-judge | rotating | completed | completed | 4 actions; both sides of the comparison name their route in words |
| propose-promotion | rotating | completed | completed | 2 actions |
| filter-targets | rotating | completed | completed | 4 actions |
| switch-axis | rotating | completed | completed | 1 action; the variant names its route in words |
| owner-trade-off | anchor | missed | missed | 9 of 10 expectations met; the state stays "Proposed" (fixture mode, UXF-011) |
| live-climb | rotating | missed, 1 look | missed, 0 looks | 5 of 6 expectations met: the step table shows steps 1, 2, 3 and 5 (UXF-006); the budget reads in words; everything the card reveals is in view |
| compare-without-pick | rotating | missed | missed | lands on the target catalog |
| dismiss-confirm | rotating | missed | missed | Escape does not close the dialog; Cancel does |

## The moments to look at

| shot | what it shows | findings |
|---|---|---|
| `shots/decision-a.png` | R-03's input: the live climb scrolled 515 px to reach "Frontier now" | UXF-026 |
| `shots/decision-b.png` | the same moment: nothing scrolled, the KPIs and the best value above, the steps and the frontier below | UXF-026 closed |
| `shots/live-climb.1-loaded.1366x768.light.png` | the live climb as it opens; the budget in two fields | UXF-026, UXF-005 |
| `shots/live-climb.7-live-again.1280x720.light.png` | the smaller gating viewport with six steps in the table, the trial's last moment: still no scroll | UXF-026, UXF-022 closed |
| `shots/live-climb.5-reconnecting-seen-from-below.1366x768.dark.png` | the dark theme; the KPI values of "Frontier now" still JSON | UXF-005 |
| `shots/switch-axis.1-variant.1366x768.light.png` | a variant: "Reached by" and "In design space" in words; the findings and evaluation values still JSON, in tables that scroll | UXF-005 |
| `shots/compare-and-judge.3-compare.1366x768.light.png` | both sides of a comparison name their route in words | UXF-005 |
| `shots/owner-trade-off.1-gate-failed.1366x768.light.png` | the promotion's summary names its changed nodes in words | UXF-005 |
| `shots/start-campaign.2-form.1366x768.light.png` | the start form once loaded, in the same place as while loading | UXF-023 closed; UXF-012 |

## Findings

26 records: 9 closed (3 of them in this round), 17 open (0 S1, 10 S2, 7 S3).

| id | change | regression asset | before | after |
|---|---|---|---|---|
| UXF-026 | closed by the yes side of `decision.md` | the page bars of the live climb | 4 of 4 cells over them | 0 |
| UXF-023 | closed | the page checks in 5 runs, `page_checks.layout_shift.spread` | the start form 0.0176 to 0.0284; spread 0.0121 | the start form 0 in 5 of 5; spread 0.0024 |
| UXF-022 | closed by the yes side of `decision.md` | the page checks in 5 runs, the live climb at 960x1080 | 0.013 in 3 of 5 runs | 0.0004 |
| UXF-005 | open; the maps are built | the new expectations of four cards; the terminal test; `page_checks.overflow.blocks` | missed in 4 of 4 runs each; failed; 14 | met in 4 of 4; passes; 0 |

UXF-005 stays open for the lists of records: the KPI values of a frontier entry, the values of an
evaluation and the findings of a rule verdict are JSON, and they keep a variant's and a comparison's
tables scrolling (gate 1). Each needs a field path through a list (decision LR-D1 of ESS); the round
adds none (`docs/ess-gaps.md`).

Known losses: UXF-005 (the lists), UXF-006, UXF-007, UXF-009, UXF-010, UXF-011.

## Design decision

`decision.md`: the live climb fits the viewport (B) against the page that scrolls with its header in
view (A). Recommendation B from step 2 of the order, default B, no answer yet. B is built. The answer
to `../R-03/decision.md` is open as well; its B is built since R-03.

## Questionnaire and replay

None: no person took part, and levels L4, L5, L6 and L8 did not run.

## Missing inputs

As in R-03: a truth file for the scenarios, a served backend or richer fixtures, axe-core, the
screen-only user; from ESS a notice text per command and outcome, and the drawing of a list of records
in words (or a field path through a list).

## Not run

| part | why |
|---|---|
| the heuristic review | not repeated in this round; the verdicts of R-01 stand except where a finding closed |
| axe-core (gate 5) | not installed |
| the screen-only user (gate 4) | not built |
| L4 replay, L5 probe, L6, L8 | paid model levels, excluded from this round |
| fresh review agents and `review-report.md` | none ran; `decision.md` records "no view" for both reviewers |
| ratchet against a baseline file | none kept; the metrics table above is the comparison |
