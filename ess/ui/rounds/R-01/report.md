# R-01: the first round on the studio

Gates 1, 2 and 3 fail. 5 of 11 scenarios complete in every run. 22 findings, 4 of them S1.
Nothing was changed in this round: it is the record the first change set starts from.

No person and no paid level took part. The round is the deterministic part of the loop: the page
checks, the render records, a scripted trial of the scenario cards, a page audit and a heuristic
review. Numbers are in `round.json`, the order of work in `triage.md`, the review in
`review-heuristics.md`, the findings in `../../findings/`.

## What ran

| step | command | result |
|---|---|---|
| generate | `ess generate ui --target react --path ess/ui/studio.ui.yaml --model ess --fixtures` | the React app on its fixtures |
| theme | `themes/studio.css` and `themes/theme-toggle.js` copied to `www/` and linked in the generated `www/index.html` | light and dark |
| build | `npm install`, `npm run build` in the generated project | `www/` |
| serve | the generated project's `esbuild --servedir=www --serve-fallback=www/index.html` on a loopback port | every page path answers |
| page checks | `ux-browser checks --config ess/ui/rounds/studio.checks.yaml --url <origin>` | 870 checks: 782 passed, 38 failed, 50 measured without a threshold |
| render records | `ux-browser render` per page, gating viewport and theme | 40 records and screenshots |
| trial | `node ess/ui/rounds/trial.mjs` on `studio.scenarios.json` | 11 scenarios, 44 runs, 142 screenshots |
| audit | the same script, per page | 40 audits |
| numbers | `node ess/ui/rounds/summary.mjs` | `round.json` |

All of it is one command: `sh ess/ui/rounds/round.sh R-01`. Versions and input digests are in
`versions.txt`.

## Adapter

| slot | value |
|---|---|
| scenario cards | `ess/ui/rounds/studio.scenarios.json`, 11 cards |
| anchor cards | `stop-campaign`, `owner-trade-off` |
| gating viewports | 1366x768, 1280x720 |
| measured only | 1366x657, 1536x754, 960x1080 |
| themes | light, dark |
| always visible | navigation, theme toggle, stop campaign, campaign state, gate |
| guidance cue | none |
| baseline | none: this is the first round |

Pages (10): the live climb, the frontier, the campaign list and the start form; the judgement
list; a promotion and the promotion list; the target catalog; a variant and a comparison of two.

## Hard gates

| gate | result | measured |
|---|---|---|
| 1 no scroll | fail | the document scrolls on 2 of 10 pages (the live climb 1283 px, the start form 1084 px, in 768); elements scroll on 3 more (a promotion, a variant, a comparison); 14 blocks overflow |
| 2 always visible | fail | 24 misses: Stop and the campaign state on the live climb at every viewport in both themes (20); the gate verdict on a promotion in the dark theme at both gating viewports (4) |
| 3 unmissable completion | fail | 6 of 11 scenarios are not completed; 2 runs fail outright (the axis switch at 1280x720). `completion_miss` and `false_complete` are not computed: there is no truth file |
| 4 screen-only user | not run | not built |
| 5 accessibility | not measured | axe-core is not installed. Of the keyboard: 0 misses, every Tab stop has a focus mark |
| 6 guidance cue | dropped | the studio has no guidance cue |

## Metrics

Lower is better for every one.

| metric | R-01 |
|---|---|
| `page_checks.page_scroll.total` | 0 |
| `page_checks.overflow.blocks` | 14 |
| `page_checks.always_visible.misses` | 24 |
| `page_checks.keyboard.misses` | 0 |
| `page_checks.layout_shift.max` | 0.0151 |
| `audit.pages_with_document_scroll` | 2 |
| `audit.pages_with_scrollable_element` | 3 |
| `audit.pages_without_h1` | 4 |
| `audit.text_under_14px.share` | 0.564 |
| `audit.text_low_contrast.runs_per_page.max` | 12 |
| `audit.controls_unnamed.max` | 1 |
| `audit.inputs_placeholder_only.pages` | 4 |
| `audit.tab_stops_unmarked.max` | 0 |
| `audit.pages_tab_starts_outside_navigation` | 10 |
| `trial.scenarios_not_completed` | 6 of 11 |
| `trial.runs_failed` | 2 of 44 |
| `trial.runs_expectation_missed` | 22 |
| `trial.looks.total` | 16 |
| `trial.actions.max` | 10 |

## Scenarios

The result is the same in both themes; one scenario differs by viewport.

| scenario | card | result | what happened |
|---|---|---|---|
| stop-campaign | anchor | completed | 2 actions; the state reads "Stopped" and the action is gone |
| stop-campaign-keyboard | rotating | completed | 10 key presses |
| start-campaign | rotating | completed | 10 actions, 1 look for the button below the fold; nothing changes after it (fixture mode) |
| compare-and-judge | rotating | completed | 4 actions |
| propose-promotion | rotating | completed | 2 actions |
| owner-trade-off | anchor | missed | the state stays "Proposed"; 1 look for the action |
| live-climb | rotating | missed | the step table shows steps 1, 2, 3 and 5; budget and KPI values are JSON; 2 looks |
| switch-axis | rotating | missed; failed at 1280x720 | the switch list is empty; at 1280x720 the action is covered and cannot be clicked |
| compare-without-pick | rotating | missed | lands on the target catalog |
| filter-targets | rotating | missed | 0 rows for a search that names a listed target |
| dismiss-confirm | rotating | missed | Escape does not close the dialog; Cancel does |

## The worst moments

| shot | what it shows | findings |
|---|---|---|
| `shots/owner-trade-off.1-gate-failed.1366x768.light.png` | a failed gate, and the Owner's actions outside the table frame | UXF-001, UXF-017 |
| `shots/owner-trade-off.2-dialog.1366x768.light.png` | the action reached; the gate verdict is no longer in view | UXF-001, UXF-013 |
| `shots/switch-axis.2-failed.1280x720.dark.png` | "Switch to this value" clipped and covered | UXF-002, UXF-021 |
| `shots/filter-targets.1-filtered.1366x768.light.png` | a search that finds nothing, and the wrong empty text | UXF-003, UXF-018, UXF-019 |
| `shots/live-climb.1-loaded.1366x768.light.png` | no title, and values as JSON | UXF-005, UXF-008 |
| `shots/live-climb.3-steps.1366x768.light.png` | a step missing from the table; the chart; three equal times | UXF-006, UXF-007, UXF-016 |
| `shots/live-climb.5-reconnecting-seen-from-below.1366x768.dark.png` | scrolled down: no Stop, no state, no connection state | UXF-004, UXF-022 |
| `shots/start-campaign.3-submitted-empty.1366x768.light.png` | an empty form answered with "done" | UXF-011, UXF-012 |
| `shots/compare-and-judge.4-judge-drawer.1366x768.dark.png` | a drawer that asks for what is on the page behind it; the grey field | UXF-013, UXF-020 |
| `shots/compare-and-judge.1-frontier.1366x768.light.png` | four equal buttons per row; field names and ids | UXF-014, UXF-015 |
| `shots/stop-campaign.2-confirm.1366x768.light.png` | a dialog open, focus on the button behind it | UXF-009 |
| `shots/stop-campaign.3-stopped.1366x768.dark.png` | the notice names the command | UXF-010 |

## Findings

22 findings: 4 S1, 12 S2, 6 S3. By class: layout 6, copy 5, input 5, affordance 4, engine 1,
noise 1. Each has a record in `../../findings/` with its evidence and the regression asset it
needs.

Severity is not defined in the skill. Read here as: S1 a hard gate fails or a scenario cannot be
completed; S2 the person loses time or cannot tell the state; S3 the rest.

The change set the triage names: UXF-003, UXF-004, UXF-001, UXF-002 (every S1) and UXF-017 (by
priority). Five findings are known losses, caused by the stock generator or by fixture mode:
UXF-005, UXF-006, UXF-007, UXF-009, UXF-011.

## Questionnaire and replay

None: no person took part, and levels L4, L5, L6 and L8 did not run.

## Missing inputs

- A truth file for the scenarios, so that `completion_miss` and `false_complete` can be computed.
- A served backend or richer fixtures: five of the six commands the round sends change nothing in
  fixture mode, so no end state after them was seen.
- axe-core, for gate 5.
- The screen-only user, for gate 4.

## Not run

| part | why |
|---|---|
| axe-core (gate 5) | not installed |
| the screen-only user (gate 4) | not built |
| L4 replay, L5 probe, L6, L8 | paid model levels, excluded from this round |
| fresh review agents and `review-report.md` | the heuristic review was done by the agent that ran the round, after it had seen the numbers |
| design decision, Build, Gate | this round ends at the record |
| ratchet against a baseline | no baseline yet; `round.json` of this round is the first |
