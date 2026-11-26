# R-02: three changes of the first change set

Gates 1, 2 and 3 still fail. 6 of 11 scenarios complete in every run (5 in R-01). Three findings
are closed: UXF-003, UXF-017 and UXF-001. One is new: UXF-023. 20 findings are open, 2 of them S1.

The round built three of the five changes R-01's triage named, each with a regression asset that
failed before the change and passes after it, and measured the studio again. UXF-004 and UXF-002
were not built: they wait for the design decision of a later round (R-01's triage, rule 6).

No person and no paid level took part. Numbers are in `round.json`, the same input as R-01 measured
again in `before.round.json`, the design decision in `decision.md`, the order of work in
`triage.md`, the findings in `../../findings/`.

## What changed

| # | finding | change | where |
|---|---|---|---|
| 1 | UXF-003 filters never match | the five filters name the row's field: `row.<field>` | `ess/ui/studio.ui.yaml` |
| 2 | UXF-017 small text, badge contrast | one size of 14.06 px for all small text; the text colour of a toned badge per theme; tighter table cells and page padding so that the pages still fit | `themes/studio.css` |
| 3 | UXF-001 the Owner's actions out of view | the gate decision is a list row, its fields in columns and its actions beside the verdict: the yes side of `decision.md` | `ess/ui/studio.ui.yaml`, `themes/studio.css` |

Regression assets added: two expectations in `ess/ui/tests/studio.yaml`; the check "owner action"
in `studio.checks.yaml` and the node "always owner action" in `studio.render.yaml`; the audit bars
`studio.audit.json`, counted by `summary.mjs` into the new metric `audit.cells_over_a_bar`.

## What ran

| step | command | result |
|---|---|---|
| generate | `ess generate ui --target react --path ess/ui/studio.ui.yaml --model ess --fixtures` | the React app on its fixtures |
| theme | `themes/studio.css` and `themes/theme-toggle.js` copied to `www/` and linked in the generated `www/index.html` | light and dark |
| build | `npm install`, `npm run build` in the generated project | `www/` |
| serve | the generated project's `esbuild --servedir=www --serve-fallback=www/index.html` on a loopback port | every page path answers |
| page checks | `ux-browser checks --config ess/ui/rounds/studio.checks.yaml --url <origin>` | 890 checks: 806 passed, 34 failed, 50 measured without a threshold |
| render records | `ux-browser render` per page, gating viewport and theme | 40 records and screenshots |
| trial | `node ess/ui/rounds/trial.mjs` on `studio.scenarios.json` | 11 scenarios, 44 runs, 142 screenshots |
| audit | the same script, per page | 40 audits |
| numbers | `node ess/ui/rounds/summary.mjs` with `studio.audit.json` | `round.json` |

All of it is one command: `sh ess/ui/rounds/round.sh R-02`. Versions and input digests are in
`versions.txt`. The same command ran once before the changes, on the document and theme of R-01 with
the new regression assets, as `R-02-red`: its numbers are `before.round.json`.

## Where it was measured

R-01 ran as one job of a build pool, on the pool's machine. From R-02 on, `round.sh` gives only its
two compiling steps to the pool and measures on the machine that started it. The browser is the same
build (`versions.txt`: Chrome for Testing 158.0.8127.18), and so are ess, node, react, esbuild and
typescript.

The two machines do not draw the same pixels. `before.round.json` is R-01's input measured on the
machine of R-02, and it differs from R-01's `round.json` in these values and in no count:

| value | R-01 | the same input, measured again |
|---|---|---|
| height of the live climb | 1283 px | 1258 px |
| height of the start form | 1084 px | 1044 px |
| right edge of the overflowing budget, live climb | 598 px | 607 px |
| right edge of the overflowing route, a variant | 493 px | 499 px |
| `page_checks.layout_shift.max` | 0.0151 | 0.026 |

The likely cause is the fonts each machine has: the theme asks for Inter and falls back to the
system's face. That was not checked on the pool's machine. So every metric below has three columns,
and "worse" is read against both of the first two.

## Adapter

| slot | value |
|---|---|
| scenario cards | `ess/ui/rounds/studio.scenarios.json`, 11 cards |
| anchor cards | `stop-campaign`, `owner-trade-off` |
| gating viewports | 1366x768, 1280x720 |
| measured only | 1366x657, 1536x754, 960x1080 |
| themes | light, dark |
| always visible | navigation, theme toggle, stop campaign, campaign state, gate, owner action (new) |
| audit bars | text under 14 px: 0 runs per page; low-contrast text: 0 runs per page (new) |
| guidance cue | none |
| baseline | none: the studio keeps no baseline file; R-01's `round.json` is what this round is read against |

Pages (10), as in R-01.

## Hard gates

| gate | result | measured |
|---|---|---|
| 1 no scroll | fail | the document scrolls on 2 of 10 pages (the live climb 1249 px, the start form 1077 px, in 768); elements scroll on 2 more (a variant, a comparison; a promotion no longer); 14 blocks overflow |
| 2 always visible | fail | 20 misses: Stop and the campaign state on the live climb at every viewport in both themes. A promotion: 0 (4 in R-01; 20 with the check "owner action" on R-01's input) |
| 3 unmissable completion | fail | 5 of 11 scenarios are not completed; 2 runs fail outright (the axis switch at 1280x720). `completion_miss` and `false_complete` are not computed: there is no truth file |
| 4 screen-only user | not run | not built |
| 5 accessibility | not measured | axe-core is not installed. Of the keyboard: 0 misses, every Tab stop has a focus mark. Of the audit: 0 text runs under 4.5:1 |
| 6 guidance cue | dropped | the studio has no guidance cue |

## Metrics

Lower is better for every one. "Again" is R-01's document and theme measured in this round
(`before.round.json`), with the regression assets of this round.

| metric | R-01 | again | R-02 | against R-01 |
|---|---|---|---|---|
| `page_checks.page_scroll.total` | 0 | 0 | 0 | equal |
| `page_checks.overflow.blocks` | 14 | 14 | 14 | equal |
| `page_checks.always_visible.misses` | 24 | 40 | 20 | better |
| `page_checks.keyboard.misses` | 0 | 0 | 0 | equal |
| `page_checks.layout_shift.max` | 0.0151 | 0.026 | 0.0176 | worse by 0.0025; see Layout shift |
| `audit.pages_with_document_scroll` | 2 | 2 | 2 | equal |
| `audit.pages_with_scrollable_element` | 3 | 3 | 2 | better |
| `audit.pages_without_h1` | 4 | 4 | 4 | equal |
| `audit.text_under_14px.share` | 0.564 | 0.564 | 0 | better |
| `audit.text_low_contrast.runs_per_page.max` | 12 | 12 | 0 | better |
| `audit.controls_unnamed.max` | 1 | 1 | 1 | equal |
| `audit.inputs_placeholder_only.pages` | 4 | 4 | 4 | equal |
| `audit.tab_stops_unmarked.max` | 0 | 0 | 0 | equal |
| `audit.pages_tab_starts_outside_navigation` | 10 | 10 | 10 | equal |
| `audit.cells_over_a_bar` (new) | not counted | 74 of 80 | 0 of 80 | |
| `trial.scenarios_not_completed` | 6 of 11 | 6 of 11 | 5 of 11 | better |
| `trial.runs_failed` | 2 of 44 | 2 of 44 | 2 of 44 | equal |
| `trial.runs_expectation_missed` | 22 | 22 | 16 | better |
| `trial.looks.total` | 16 | 16 | 12 | better |
| `trial.actions.max` | 10 | 10 | 10 | equal |

`page_checks.always_visible.misses` counts one item more than in R-01, the owner action. With R-01's
items alone the three columns read 24, 24 and 20.

One metric is worse than R-01 recorded it: `page_checks.layout_shift.max`. Two values that no metric
counts moved the wrong way against "again": the start form is 33 px taller (1044 to 1077 px; it
scrolled before and scrolls now), and a variant at 1280x720 has little room left since its "Axis
switches" table shows a row (its document is 720 px in 720; the free room was not measured).

## Layout shift

`page_checks.layout_shift.max` is, in all three columns, the shift of the start form after it
reports ready. That value is not stable (UXF-023): the form is moved by content that arrives within
about 25 ms of the ready mark, sometimes before it and sometimes after.

| input | measured, per run of the page checks |
|---|---|
| R-01 as recorded | 0.0151 |
| R-01's document and theme, again | 0.026 |
| R-01's theme, 4 more runs of this page | 0.0129, 0.0208, 0.0208, 0.0129 |
| R-02 | 0.0176 |
| R-02's theme, 5 more runs | 0.0284, 0.0176, 0.0284, 0.003, 0.0284 |

Read: R-02's value lies inside the range of R-01's input on the same machine (0.0129 to 0.026), and
0.0025 above the one value R-01 recorded on the other machine. The change of UXF-017 does make the
late content taller, and the largest value seen with it is 0.0024 above the largest seen without it
(0.0284 against 0.026). Both differences are under the skill's tolerance for layout shift, 0.01.
The nine extra runs are measurements of this round and are not in `round.json`.

## Scenarios

The result is the same in both themes; one scenario differs by viewport.

| scenario | card | R-01 | R-02 | what happened in R-02 |
|---|---|---|---|---|
| stop-campaign | anchor | completed | completed | 2 actions |
| stop-campaign-keyboard | rotating | completed | completed | 10 key presses |
| start-campaign | rotating | completed | completed | 10 actions, 1 look for the button below the fold |
| compare-and-judge | rotating | completed | completed | 4 actions |
| propose-promotion | rotating | completed | completed | 2 actions |
| filter-targets | rotating | missed | completed | "form" lists "Settings form"; 4 actions |
| owner-trade-off | anchor | missed, 1 look | missed, 0 looks | verdict and actions are in one view; the state stays "Proposed" (fixture mode, UXF-011) |
| switch-axis | rotating | missed; failed at 1280x720 | completed at 1366x768; failed at 1280x720 | the switch list shows its row; at 1280x720 the action is still covered and cannot be clicked (UXF-002) |
| live-climb | rotating | missed | missed | as in R-01: steps 1, 2, 3 and 5; budget and KPI values are JSON; 2 looks |
| compare-without-pick | rotating | missed | missed | lands on the target catalog |
| dismiss-confirm | rotating | missed | missed | Escape does not close the dialog; Cancel does |

## The moments to look at

| shot | what it shows | findings |
|---|---|---|
| `shots/decision-no.png` | R-01's input: a failed gate, the Owner's actions outside the table frame | UXF-001 |
| `shots/decision-yes.png` | the verdict, what gets worse and better, and the actions in one view | UXF-001 closed; UXF-005 |
| `shots/owner-trade-off.2-dialog.1366x768.light.png` | the dialog over the decision; the verdict stays in view behind it | UXF-013 |
| `shots/owner-trade-off.1-gate-failed.1280x720.dark.png` | the same page in the dark theme at the smaller gating viewport | UXF-017 closed |
| `shots/filter-targets.1-filtered.1366x768.light.png` | a search that finds its row | UXF-003 closed |
| `shots/filter-targets.2-no-match.1366x768.light.png` | a search without a match, and the wrong empty text | UXF-019 |
| `shots/switch-axis.3-switches.1366x768.light.png` | the switch list with a row; the notice says the switch was not applied | UXF-003 closed; UXF-010, UXF-011 |
| `shots/switch-axis.2-failed.1280x720.dark.png` | "Switch to this value" still clipped and covered | UXF-002 |
| `shots/live-climb.1-loaded.1366x768.light.png` | badges and labels at 14 px; no title, values as JSON | UXF-017 closed; UXF-005, UXF-008 |
| `shots/live-climb.5-reconnecting-seen-from-below.1366x768.dark.png` | scrolled down: no Stop, no state, no connection state | UXF-004 |
| `shots/start-campaign.2-form.1366x768.light.png` | the start form once loaded: the target above it and the suggestions arrived after the first paint | UXF-023; UXF-012 |

## Findings

23 records: 3 closed in this round, 20 open (2 S1, 11 S2, 7 S3).

| id | change | regression asset | before | after |
|---|---|---|---|---|
| UXF-003 | closed | 2 expectations in `ess/ui/tests/studio.yaml`; `filter-targets`, `switch-axis` step 3 | `5 passed, 2 failed`; missed in 8 of 8 runs | `7 passed, 0 failed`; met in 8 of 8 |
| UXF-017 | closed | the audit bars of `studio.audit.json` | 74 of 80 cells over a bar | 0 of 80 |
| UXF-001 | closed by the yes side of `decision.md` | the checks "gate" and "owner action" | 20 misses on the promotion | 0 |
| UXF-023 | new, S3, open | to add | | |

Known losses stay as R-01 named them: UXF-005, UXF-006, UXF-007, UXF-009, UXF-011. UXF-023 is a
candidate for the list: its cause is in the stock generator.

## Design decision

`decision.md`: the gate decision of a promotion as a card (B) against the table row (A).
Recommendation B from step 2 of the order, default B, no answer yet. B is built.

## Questionnaire and replay

None: no person took part, and levels L4, L5, L6 and L8 did not run.

## Missing inputs

- The same fonts on every machine that measures, or the fonts served with the app: without them a
  round on another machine is not comparable in its pixel values.
- A truth file for the scenarios, so that `completion_miss` and `false_complete` can be computed.
- A served backend or richer fixtures: five of the six commands the round sends change nothing in
  fixture mode, so no end state after them was seen.
- axe-core, for gate 5.
- The screen-only user, for gate 4.

## Not run

| part | why |
|---|---|
| the heuristic review | not repeated in this round; R-01's verdicts stand except where a finding closed |
| axe-core (gate 5) | not installed |
| the screen-only user (gate 4) | not built |
| L4 replay, L5 probe, L6, L8 | paid model levels, excluded from this round |
| fresh review agents and `review-report.md` | none ran; `decision.md` records "no view" for both reviewers |
| UXF-004, UXF-002 | in the change set, not built: each needs a design decision, and a round takes one |
| ratchet against a baseline file | none kept; the metrics table above is the comparison |
