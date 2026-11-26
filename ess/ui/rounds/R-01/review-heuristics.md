# R-01 heuristic review

1 pass, 6 partial, 7 fail; 2 not observed.

Not the review § 8 of the skill asks for: this one was done by the session that ran the round,
which had seen the round's numbers, and not by a fresh agent. There are no recordings; the evidence
is the trial's screenshots (each 0.75 s after its moment), the render records and `round.json`.
Screenshots named `shots/...` are in this directory; the others are in the evidence directory the
round command writes (`target/loop-round/R-01/`).

## 1. Heuristics

| # | heuristic | verdict | evidence |
|---|---|---|---|
| 1 | One next thing | partial | The lists name one header action ("Pick a target to start"). The frontier offers four buttons of equal weight on every row: `shots/compare-and-judge.1-frontier.1366x768.light.png`. On a promotion the one action is out of view: `shots/owner-trade-off.1-gate-failed.1366x768.light.png`. Every form ends in "Save" |
| 2 | State in words | partial | States are words on badges (Running, Stopped, Proposed, fail) and the connection says "Live", "Reconnecting": `shots/stop-campaign.3-stopped.1366x768.dark.png`. Four pages do not say what they are (`round.json` audit, h1 0), and scrolled down the live climb shows no connection state: `shots/live-climb.5-reconnecting-seen-from-below.1366x768.dark.png` |
| 3 | Nothing required hidden | fail | The Owner's actions are outside the table frame: `shots/owner-trade-off.1-gate-failed.1366x768.light.png`. The switch action is clipped and covered: `shots/switch-axis.2-failed.1280x720.dark.png`. Stop and the campaign state scroll away (`round.json` page_check_misses, 20). The start form's button is below the fold (`round.json` trial "start-campaign", 1 look per run) |
| 4 | Stable layout | partial | After load the layout shift is at most 0.0151 (`round.json` page_checks). During a live climb the step table grows and moves "Frontier now" 78 px: `shots/live-climb.3-steps.1366x768.light.png` against `shots/live-climb.5-reconnecting-seen-from-below.1366x768.dark.png` |
| 5 | Provenance of every value | partial | The gate section says "Gate decision from ratchet" and shows the policy revision; the campaign shows its policy digest and layout model version. The charts have no value axis, the second no title; the step table and the chart disagree on step 4; three step times read the same: `shots/live-climb.3-steps.1366x768.light.png` |
| 6 | Errors cheap | fail | "Compare with picked" with nothing picked lands on Targets; the way back is 3 actions (`round.json` trial "compare-without-pick"). An empty start form is answered with "done": `shots/start-campaign.3-submitted-empty.1366x768.light.png`. Escape does not close a confirm; Cancel does, in 1 action (`round.json` trial "dismiss-confirm") |
| 7 | Words the user can say | fail | "uxsearch.search.StopCampaign done": `shots/stop-campaign.3-stopped.1366x768.dark.png`. Field names as labels, ids as values, `lower_is_better` on a badge, JSON as values: `shots/live-climb.1-loaded.1366x768.light.png` |
| 8 | The counterpart's time respected | not observed | |
| 9 | Keyboard and pointer parity | partial | Every control is a Tab stop with a focus mark (`round.json` metrics, `keyboard.misses` 0, unmarked stops 0). Stopping a campaign by keyboard completes in 10 key presses against 2 clicks. A dialog does not take focus, Escape does nothing, and focus is on the body after it closes (`round.json` trial moments, `focus_inside` false) |
| 10 | Recognition over recall | fail | The judgement drawer asks for two digests that are on the page behind it: `shots/compare-and-judge.4-judge-drawer.1366x768.dark.png`. A campaign names its target "tg-02". A variant page has no heading |
| 11 | Calm by default | pass | A notice appears only after a command and leaves after 5 s; the connection badge changes only with the connection; nothing moves without a cause: `shots/stop-campaign.3-stopped.1366x768.dark.png`, `shots/live-climb.1-loaded.1366x768.light.png`. The empty card and the empty labels of UXF-019 are the exceptions |
| 12 | One state vocabulary | partial | One badge shape for every state, and the same tones for pass and fail on every page. A rule's strength "hard" is red beside a green "pass": `shots/switch-axis.2-failed.1280x720.dark.png`; states are capitalised, verdicts are not |
| 13 | A wrong cue recovered in one action | not observed | |
| 14 | Unmistakable end state | fail | Stop ends well: the state reads "Stopped" and the action is gone, `shots/stop-campaign.3-stopped.1366x768.dark.png`. A sent axis switch can never be seen, its list is always empty (`round.json` trial "switch-axis", step 3). After five other commands nothing on the page changes (fixture mode): `shots/start-campaign.3-submitted-empty.1366x768.light.png` |
| 15 | Readable in every theme; key data at 14 px or more | fail | 56.4 % of the text runs are under 14 px, the state badges 11.4 px; light badges 3.69:1 to 4.07:1; dark group headings 4.33:1 (`round.json` audit). The multi-line field in the dark theme: `shots/compare-and-judge.4-judge-drawer.1366x768.dark.png` |
| 16 | Focus follows the work | fail | With a dialog open, focus is on the button behind it: `shots/stop-campaign.2-confirm.1366x768.light.png` and `round.json` trial moments. After it closes, and after every navigation, focus is on the body; a form does not focus its first field |

## 2. Findings filed

| id | title | severity | class | scenarios | cost |
|---|---|---|---|---|---|
| UXF-001 | On a promotion the Owner's actions are out of view, and reaching them hides the gate verdict | S1 | layout | owner-trade-off | M |
| UXF-002 | On a variant "Switch to this value" is clipped and cannot be clicked at 1280x720 | S1 | layout | switch-axis | M |
| UXF-003 | Filters never match: the search of four lists finds nothing, the axis-switch list is always empty | S1 | engine | filter-targets, switch-axis | S |
| UXF-004 | The live climb scrolls, and Stop, the campaign state and the connection state leave the viewport | S1 | layout | live-climb, start-campaign | M |
| UXF-005 | Structured values are drawn as raw JSON | S2 | input | live-climb, owner-trade-off, switch-axis, compare-and-judge, compare-without-pick, propose-promotion | L |
| UXF-006 | A live step that arrives as the climb opens is missing from the step table | S2 | input | live-climb | M |
| UXF-007 | The progress chart has no title and no value axis, and its steps run in arrival order | S2 | input | live-climb | L |
| UXF-008 | Four pages show no title | S2 | copy | live-climb, switch-axis, compare-and-judge, compare-without-pick, propose-promotion | S |
| UXF-009 | Dialogs and drawers do not take focus, Escape does not close them, focus is lost when they close | S2 | input | stop-campaign, stop-campaign-keyboard, dismiss-confirm, owner-trade-off, compare-and-judge, propose-promotion | L |
| UXF-010 | A success notice names the command, not what happened | S2 | copy | stop-campaign, stop-campaign-keyboard, owner-trade-off, switch-axis, start-campaign, compare-and-judge, propose-promotion | M |
| UXF-011 | In fixture mode five of six commands change nothing | S2 | input | owner-trade-off, switch-axis, start-campaign, compare-and-judge, propose-promotion | L |
| UXF-012 | The start form accepts an empty submit, marks nothing as required, keeps its button below the fold | S2 | affordance | start-campaign | M |
| UXF-013 | Overlay forms are not filled from the row they were opened on, and every submit reads "Save" | S2 | affordance | compare-and-judge, propose-promotion, owner-trade-off | M |
| UXF-014 | "Compare with picked" is offered with nothing picked and lands on Targets without a word | S2 | affordance | compare-without-pick | S |
| UXF-015 | Labels are field names and values are ids | S2 | copy | all 11 | M |
| UXF-016 | Step times drop the seconds | S3 | copy | live-climb | M |
| UXF-017 | More than half of the text is under 14 px, and state badges are under 4.5:1 in the light theme | S2 | layout | all 11 | S |
| UXF-018 | The search fields and the catalog's document choice have no label | S3 | affordance | stop-campaign, stop-campaign-keyboard, dismiss-confirm, start-campaign, filter-targets | M |
| UXF-019 | Empty states say the wrong thing or nothing | S3 | copy | filter-targets, start-campaign, switch-axis, owner-trade-off, live-climb | S |
| UXF-020 | In the dark theme a multi-line text field is drawn on an unthemed grey | S3 | layout | compare-and-judge, propose-promotion | S |
| UXF-021 | Badge colours are not one vocabulary: a rule's strength "hard" is drawn like a failure | S3 | noise | switch-axis, compare-and-judge | S |
| UXF-022 | The step table grows and pushes "Frontier now" down while it is read | S3 | layout | live-climb | M |

UXF-001 to UXF-006, UXF-009, UXF-011, UXF-012, UXF-014, UXF-017 and UXF-018 were found by a
measurement of the round and are confirmed here; the other ten come from this review.

## 3. Decision

no decision this round

## 4. Not observed

- 8, the counterpart's time respected. The studio is worked by one person at a time and no scenario
  of this round has someone waiting on another. A card where a campaign waits for the Owner's
  decision on a promotion would show it.
- 13, a wrong cue recovered in one action. The studio has no guidance cue (the adapter's slot is
  "none"). It applies once a page names a next step.
