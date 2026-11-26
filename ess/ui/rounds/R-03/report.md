# R-03: the two open S1 findings, the page titles, and what the studio can do about two known gaps

Gate 2 passes for the first time. Gates 1 and 3 still fail. 7 of 11 scenarios complete in every run
(6 in R-02), and no run fails outright (2 in R-02). Three findings are closed: UXF-004, UXF-002 and
UXF-008. Three are new: UXF-024, UXF-025 and UXF-026. 20 findings are open, none of them S1.

The round was ordered with five changes. Three were built and closed their finding. One was built
for the part the studio can reach and its finding stays open (UXF-005). One could not be built in
the studio at all (UXF-010): its regression asset exists and fails.

No person and no paid level took part. Numbers are in `round.json`, R-02's input measured again in
`before.round.json`, the design decision in `decision.md`, the order of work in `triage.md`, the
findings in `../../findings/`.

## What changed

| # | finding | change | where |
|---|---|---|---|
| 1 | UXF-004 the live climb scrolls, Stop and the state leave the viewport | the header of a page stays at the top of the viewport and the narrow column stays under it while the main column scrolls: the yes side of `decision.md` | `themes/studio.css` |
| 2 | UXF-002 the switch action is clipped at 1280x720 | the action reads "Switch"; the first and the last cell of a table row have no outer padding | `ess/ui/studio.ui.yaml`, `themes/studio.css` |
| 3 | UXF-008 four pages show no title | `title: from_page` in the four headers | `ess/ui/studio.ui.yaml` |
| 4 | UXF-010 a notice names the command | none: the words are written by the generated runtime; the document has no key for them | |
| 5 | UXF-005 structured values as JSON | the decision of a promotion and the dialog that accepts its trade-off declare one field per member of each record; the theme does not draw the JSON fields there | `ess/ui/studio.ui.yaml`, `themes/studio.css` |

UXF-002 needed no second layout decision: a shorter label and 8 px less padding in the table, nothing
moved. The "Axes" table of a variant is 208 px in its 208 px at 1280x720.

Regression assets added, all in the round:

- `studio.audit.json`, the set `floors` (`h1` at least 1), counted by `summary.mjs` into the new
  metric `audit.cells_under_a_floor` (UXF-008);
- `studio.scenarios.json`, `notices.not_text` "uxsearch.", held on every notice of every card and
  counted by `trial.mjs` and `summary.mjs` into the new metric
  `trial.runs_notice_expectation_missed`; it does not change whether a run completed (UXF-010);
- `studio.scenarios.json`, two expectations of `owner-trade-off`: no `{"` in the section "decision"
  and in the record of the trade-off dialog (UXF-005).

UXF-004 and UXF-002 are held by assets that were there: the checks "stop campaign" and "campaign
state" of `studio.checks.yaml`, and the card `switch-axis`.

No test was added to `ess/ui/tests/studio.yaml`. The terminal renderer prints a title in every
header and prints a list of records as words already, so it cannot fail for UXF-008 or UXF-005; it
shows no notice (UXF-010) and has no viewport (UXF-004, UXF-002).

Also changed in the round's own files: `trial.mjs` judges an expectation on the whole text of its
node (its record keeps the first 200 characters, as before); `round.sh` writes into `versions.txt`
the font the machine resolves for each family the theme names; the stale `source` text of the card
`filter-targets` is corrected.

## What ran

| step | command | result |
|---|---|---|
| generate | `ess generate ui --target react --path ess/ui/studio.ui.yaml --model ess --fixtures` | the React app on its fixtures |
| theme | `themes/studio.css` and `themes/theme-toggle.js` copied to `www/` and linked in the generated `www/index.html` | light and dark |
| build | `npm install`, `npm run build` in the generated project | `www/` |
| serve | the generated project's `esbuild --servedir=www --serve-fallback=www/index.html` on a loopback port | every page path answers |
| page checks | `ux-browser checks --config ess/ui/rounds/studio.checks.yaml --url <origin>` | 890 checks: 826 passed, 14 failed, 50 measured without a threshold |
| render records | `ux-browser render` per page, gating viewport and theme | 40 records and screenshots |
| trial | `node ess/ui/rounds/trial.mjs` on `studio.scenarios.json` | 11 scenarios, 44 runs, 144 screenshots |
| audit | the same script, per page | 40 audits |
| numbers | `node ess/ui/rounds/summary.mjs` with `studio.audit.json` | `round.json` |

All of it is one command: `sh ess/ui/rounds/round.sh R-03`. Versions and input digests are in
`versions.txt`. The same command ran once before the changes, on the document and theme of R-02 with
the new regression assets, as `R-03-red`: its numbers are `before.round.json` (890 checks: 806
passed, 34 failed).

Two things about `before.round.json`:

- its font lines were added to its versions after it ran, from the same machine; `round.sh` wrote
  them itself only from the next run on;
- the expectation on the trade-off dialog was written after it ran. Its red is the card played
  alone on the same build (`../../findings/UXF-005.md`), so `before.round.json` shows 8 expectations
  of `owner-trade-off` and `round.json` shows 9.

## Where it was measured

On the machine of R-02, with the same browser build (Chrome for Testing 158.0.8127.18) and the same
ess, node, react, esbuild and typescript. ux-browser is 0.11.0 (R-02: 0.10.0). R-02's input measured
again gives every count and every page height of R-02; only the layout shift differs (below).

The fonts, new in `versions.txt`: the theme asks for Inter, then "Segoe UI", then `system-ui`. On
this machine Inter is installed and is what the browser draws; the two others resolve to Noto Sans.
R-02's report named the fonts as the likely cause of the pixel differences between its machine and
R-01's. The line is there now for the next machine to compare; it was not compared in this round.
The repository holds no font file.

## Adapter

| slot | value |
|---|---|
| scenario cards | `ess/ui/rounds/studio.scenarios.json`, 11 cards |
| anchor cards | `stop-campaign`, `owner-trade-off` |
| gating viewports | 1366x768, 1280x720 |
| measured only | 1366x657, 1536x754, 960x1080 |
| themes | light, dark |
| always visible | navigation, theme toggle, stop campaign, campaign state, gate, owner action |
| audit bars | text under 14 px: 0 runs per page; low-contrast text: 0 runs per page |
| audit floors | titles (`h1`): at least 1 per page (new) |
| notices | no notice contains "uxsearch." (new) |
| guidance cue | none |
| baseline | none: the studio keeps no baseline file; R-02's `round.json` is what this round is read against |

Pages (10), as in R-01.

## Hard gates

| gate | result | measured |
|---|---|---|
| 1 no scroll | fail | the document scrolls on 2 of 10 pages (the live climb 1249 px, the start form 1077 px, in 768); elements scroll on 2 more (a variant, a comparison); 14 blocks overflow |
| 2 always visible | pass | 0 misses (20 in R-02): Stop and the campaign state stay in view on the live climb at every viewport in both themes |
| 3 unmissable completion | fail | 4 of 11 scenarios are not completed; no run fails outright. `completion_miss` and `false_complete` are not computed: there is no truth file |
| 4 screen-only user | not run | not built |
| 5 accessibility | not measured | axe-core is not installed. Of the keyboard: 0 misses, every Tab stop has a focus mark. Of the audit: 0 text runs under 4.5:1 |
| 6 guidance cue | dropped | the studio has no guidance cue |

Gate 2 passes and gate 1 does not: the live climb is as tall as before. What is left of UXF-004 is
UXF-026.

## Metrics

Lower is better for every one. "Again" is R-02's document and theme measured in this round
(`before.round.json`), with the regression assets of this round.

| metric | R-02 | again | R-03 | against R-02 |
|---|---|---|---|---|
| `page_checks.page_scroll.total` | 0 | 0 | 0 | equal |
| `page_checks.overflow.blocks` | 14 | 14 | 14 | equal |
| `page_checks.always_visible.misses` | 20 | 20 | 0 | better |
| `page_checks.keyboard.misses` | 0 | 0 | 0 | equal |
| `page_checks.layout_shift.max` | 0.0176 | 0.0229 | 0.0284 | worse by 0.0108; see Layout shift |
| `audit.pages_with_document_scroll` | 2 | 2 | 2 | equal |
| `audit.pages_with_scrollable_element` | 2 | 2 | 2 | equal |
| `audit.pages_without_h1` | 4 | 4 | 0 | better |
| `audit.text_under_14px.share` | 0 | 0 | 0 | equal |
| `audit.text_low_contrast.runs_per_page.max` | 0 | 0 | 0 | equal |
| `audit.controls_unnamed.max` | 1 | 1 | 1 | equal |
| `audit.inputs_placeholder_only.pages` | 4 | 4 | 4 | equal |
| `audit.tab_stops_unmarked.max` | 0 | 0 | 0 | equal |
| `audit.pages_tab_starts_outside_navigation` | 10 | 10 | 10 | equal |
| `audit.cells_over_a_bar` | 0 of 80 | 0 of 80 | 0 of 80 | equal |
| `audit.cells_under_a_floor` (new) | not counted | 16 of 40 | 0 of 40 | |
| `trial.scenarios_not_completed` | 5 of 11 | 5 of 11 | 4 of 11 | better |
| `trial.runs_failed` | 2 of 44 | 2 of 44 | 0 of 44 | better |
| `trial.runs_expectation_missed` | 16 | 16 | 16 | equal |
| `trial.runs_notice_expectation_missed` (new) | not counted | 26 of 44 | 28 of 44 | |
| `trial.looks.total` | 12 | 12 | 8 | better |
| `trial.actions.max` | 10 | 10 | 10 | equal |

`trial.runs_notice_expectation_missed` is 2 higher after the changes than before them, and no
notice changed: the two runs are `switch-axis` at 1280x720, which failed before they reached their
command and now show its notice.

A variant has 2 scrolling elements per cell where it had 3: its "Axes" table no longer scrolls.
The count of pages with a scrolling element is 2 as before, since the page still has two.

One metric is worse than R-02 recorded it: `page_checks.layout_shift.max`.

## Layout shift

`page_checks.layout_shift.max` is, in all three columns, the shift of the start form after it
reports ready. That value is not stable (UXF-023). R-03's value is 0.0108 over R-02's, 0.0008 past
the skill's tolerance of 0.01. As it is the only value over R-02, the page checks of that page ran
4 more times; and they had run 4 more times on an earlier build of this round, which differs from
the final one only in the trade-off dialog of a promotion.

| input | measured, per run of the page checks |
|---|---|
| R-02 as recorded | 0.0176 |
| R-02's theme, 5 more runs in R-02 | 0.0284, 0.0176, 0.0284, 0.003, 0.0284 |
| R-02's document and theme, again | 0.0229 |
| R-03 | 0.0284 |
| R-03's theme, 4 more runs | 0.0157, 0.0284, 0.0143, 0.0155 |
| R-03's theme, the earlier build | 0.0284 |
| R-03's theme, the earlier build, 4 more runs | 0.0176, 0.0062, 0.0176, 0.0284 |

Read: on R-02's theme the value was seen from 0.003 to 0.0284 in 7 runs; on R-03's theme from
0.0062 to 0.0284 in 10 runs. The largest value is the same, 0.0284, and R-02 measured it three
times in its own extra runs. The round's change did not raise what this page can reach; the one
run that `round.json` keeps drew the largest value where R-02's drew a middle one. The runs beside
`round.json` are measurements of this round and are not in it.

Per viewport in R-03: 1366x768 0.0125, 1280x720 0.0057, 1366x657 0.0111, 1536x754 0.0155,
960x1080 0.0284.

A second page has a value: the live climb, 0.013 at 960x1080 and under 0.001 at the four other
viewports. R-02 recorded 0.0009 for it; R-02's input measured again gives 0.0125, at the same
viewport. So it is there without this round's theme, and it is not the metric's maximum. It is
recorded in UXF-023.

## Scenarios

The result is the same in both themes and, from this round on, at both viewports.

| scenario | card | R-02 | R-03 | what happened in R-03 |
|---|---|---|---|---|
| stop-campaign | anchor | completed | completed | 2 actions |
| stop-campaign-keyboard | rotating | completed | completed | 10 key presses |
| start-campaign | rotating | completed | completed | 10 actions, 1 look for the button below the fold |
| compare-and-judge | rotating | completed | completed | 4 actions |
| propose-promotion | rotating | completed | completed | 2 actions |
| filter-targets | rotating | completed | completed | 4 actions |
| switch-axis | rotating | completed at 1366x768; failed at 1280x720 | completed | 1 action at both viewports: the click on "Switch" lands |
| owner-trade-off | anchor | missed | missed | 8 of 9 expectations met: the decision and the dialog read in words; the state stays "Proposed" (fixture mode, UXF-011) |
| live-climb | rotating | missed, 2 looks | missed, 1 look | as in R-02: steps 1, 2, 3 and 5; budget and KPI values are JSON. Stop and the state need no look back |
| compare-without-pick | rotating | missed | missed | lands on the target catalog |
| dismiss-confirm | rotating | missed | missed | Escape does not close the dialog; Cancel does |

The seven cards that send a command show a notice with the command's name in it in every run.

## The moments to look at

| shot | what it shows | findings |
|---|---|---|
| `shots/decision-no.png` | R-02's input: the live climb scrolled 515 px, no Stop, no state, no connection state | UXF-004 |
| `shots/decision-yes.png` | the same moment: the header and the summary stay, the step table scrolls past them | UXF-004 closed; UXF-026 |
| `shots/live-climb.5-reconnecting-seen-from-below.1366x768.dark.png` | the same in the dark theme, "reconnecting" readable from below | UXF-004 closed |
| `shots/live-climb.1-loaded.1366x768.light.png` | the live climb with its title; the budget still JSON and past its card | UXF-008 closed; UXF-005 |
| `shots/compare-and-judge.1-frontier.1366x768.light.png` | the frontier with its title | UXF-008 closed |
| `shots/compare-and-judge.3-compare.1366x768.light.png` | the comparison with its title; which two variants it compares stands only in the fields | UXF-008 closed; UXF-015 |
| `shots/switch-axis.2-switch-sent.1280x720.dark.png` | the smaller gating viewport after the click on "Switch"; the notice lies over the last row's action | UXF-002 closed; UXF-024, UXF-010 |
| `shots/owner-trade-off.1-gate-failed.1366x768.light.png` | the decision in words: what gets worse and better with both values, each requirement with its verdict; the diff above it still JSON | UXF-005; UXF-025 |
| `shots/owner-trade-off.2-dialog.1366x768.light.png` | the dialog names what gets worse and better in words | UXF-005; UXF-013 |
| `shots/stop-campaign.3-stopped.1366x768.dark.png` | the notice after Stop: "uxsearch.search.StopCampaign done" | UXF-010 |
| `shots/start-campaign.2-form.1366x768.light.png` | the start form once loaded | UXF-023; UXF-012 |

## Findings

26 records: 6 closed (3 of them in this round), 20 open (0 S1, 11 S2, 9 S3).

| id | change | regression asset | before | after |
|---|---|---|---|---|
| UXF-004 | closed by the yes side of `decision.md` | the checks "stop campaign" and "campaign state" | 20 misses on the live climb | 0 |
| UXF-002 | closed | the card `switch-axis` | failed in 2 of 4 runs | completed in 4 of 4 |
| UXF-008 | closed | the floor `h1` of `studio.audit.json` | 16 of 40 cells under it | 0 of 40 |
| UXF-010 | open; known loss from this round on | `notices.not_text` of `studio.scenarios.json` | missed in 26 of 44 runs | missed in 28 of 44 |
| UXF-005 | open; built for one page | `owner-trade-off`, two expectations | missed in 4 of 4 runs each | met in 4 of 4 |
| UXF-023 | open; measured again | to add | | |
| UXF-024 | new, S3, open | to add | | |
| UXF-025 | new, S3, open | to add | | |
| UXF-026 | new, S2, open | the page audit, `audit.pages_with_document_scroll` | | 2 |

Known losses: UXF-005, UXF-006, UXF-007, UXF-009, UXF-011 as R-01 named them, and UXF-010.

What the studio's document and theme cannot do with ess 0.99.2, as found in this round:

- Notices. An action, a confirm and a form cannot name the text of the notice their command
  answers with. The generated runtime writes `<command> done`, `sent`, `failed` and "done in
  fixture mode" from the command's qualified name.
- Lists of records. No `as` draws a list of records, or a map, as words in the generated app. A
  field path with a dot reads a member of every entry there, and the terminal renderer prints
  nothing for the same path through a list; the path form is not in the format's reference. The
  decision of a promotion stands on it all the same, and the theme hides the JSON fields beside it.

`docs/ess-gaps.md` has the second as "A list of records as text", without the path form, and does
not have the first; the round does not edit that file.

One file outside the round still names what the React app no longer draws: the journey of
`ess/ui/studio.uxui.yaml` requires the nodes `columns/losses` and `columns/gains` of the decision.
Their words are at `columns/worse` and `columns/better` now. The nodes are still declared and the
terminal renderer still draws them; no step of the repository's gate fails for it.

## Design decision

`decision.md`: the page header and the narrow column stay in view while a page scrolls (B) against
the page that scrolls as one sheet (A). Recommendation B from step 2 of the order, default B, no
answer yet. B is built.

The answer to `../R-02/decision.md` is open as well; its B is built since R-02.

## Questionnaire and replay

None: no person took part, and levels L4, L5, L6 and L8 did not run.

## Missing inputs

- The fonts of the pool's machine, to compare with the three lines of `versions.txt`; or the fonts
  served with the app.
- A truth file for the scenarios, so that `completion_miss` and `false_complete` can be computed.
- A served backend or richer fixtures: five of the six commands the round sends change nothing in
  fixture mode, so no end state after them was seen.
- axe-core, for gate 5.
- The screen-only user, for gate 4.
- From ESS: a notice text per command and outcome, and one drawing of a list of records that both
  renderers share.

## Not run

| part | why |
|---|---|
| the heuristic review | not repeated in this round; R-01's verdicts stand except where a finding closed |
| axe-core (gate 5) | not installed |
| the screen-only user (gate 4) | not built |
| L4 replay, L5 probe, L6, L8 | paid model levels, excluded from this round |
| fresh review agents and `review-report.md` | none ran; `decision.md` records "no view" for both reviewers |
| UXF-010 | ordered, not built: neither the document nor the theme can set a notice's words |
| UXF-005 for the maps and the other lists | not built: the budget's texts are held as one field by a test and a card, and the terminal drawing of the live climb feeds a golden file of `engine/search`; `triage.md` names it as the next change |
| ratchet against a baseline file | none kept; the metrics table above is the comparison |
