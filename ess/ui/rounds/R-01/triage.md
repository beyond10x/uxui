# R-01 triage

Applied by rule (`plugin/skills/ui-iteration-loop/SKILL.md` § 3.3) to the 22 findings of the round.
Nothing was changed in this round; the change set below is what the next change would take.

## Caveats

- No person and no paid level took part: no recording, no questionnaire, no L4 replay, no L5 probe.
  "Two sources agree" therefore means two of: the ux-browser page checks, the trial, the page audit,
  the heuristic review.
- The heuristic review was done by the session that ran the round, after it had seen the numbers,
  not by a fresh agent. Its agreement with a metric is weaker than the skill assumes. Seven
  findings rest on it alone and carry confidence 1.
- The trial is a driver that names nodes by path, not the screen-only user of § 7. A control it
  found may still be one a person does not find.
- Frequency counts scenarios of this round: 11, each played at 2 viewports in 2 themes. Every
  scenario gave the same result in both themes; one (switch-axis) differs by viewport.
- axe-core did not run. Gate 5 is not measured; UXF-009, UXF-017 and UXF-018 are what the round's
  own audit could see of it.
- The app ran on its fixtures. Five of the six commands the round sends change nothing there
  (UXF-011), so no end state after them was seen.
- Severity is not defined in the skill. Read here as: S1 a hard gate fails or a scenario cannot be
  completed; S2 the person loses time or cannot tell the state; S3 the rest.
- Cost S, M or L is the session's reading of where the change lands: S a few lines of
  `ess/ui/studio.ui.yaml` or `themes/studio.css`; M a section or a page of the document reworked;
  L a change in ESS or its generated runtime.

## Did the last change sets move their metrics?

| change set | finding | target metric | before | after | moved? |
|---|---|---|---|---|---|
| none | | | | | this is the first round |

## Priorities

`priority = severity weight × frequency × confidence ÷ cost`, with S1 = 9, S2 = 3, S3 = 1 and
S = 1, M = 2, L = 4.

| id | title | class | severity | scenarios | confidence | cost | priority |
|---|---|---|---|---|---|---|---|
| UXF-017 | More than half of the text is under 14 px, state badges under 4.5:1 | layout | S2 | 11 of 11 | 2 | S | 6.00 |
| UXF-003 | Filters never match: search finds nothing, the axis-switch list is empty | engine | S1 | 2 of 11 | 2 | S | 3.27 |
| UXF-008 | Four pages show no title | copy | S2 | 5 of 11 | 2 | S | 2.73 |
| UXF-010 | A success notice names the command, not what happened | copy | S2 | 7 of 11 | 2 | M | 1.91 |
| UXF-004 | The live climb scrolls; Stop and the states leave the viewport | layout | S1 | 2 of 11 | 2 | M | 1.64 |
| UXF-015 | Labels are field names and values are ids | copy | S2 | 11 of 11 | 1 | M | 1.50 |
| UXF-001 | On a promotion the Owner's actions are out of view | layout | S1 | 1 of 11 | 2 | M | 0.82 |
| UXF-002 | "Switch to this value" is clipped and cannot be clicked at 1280x720 | layout | S1 | 1 of 11 | 2 | M | 0.82 |
| UXF-005 | Structured values are drawn as raw JSON | input | S2 | 6 of 11 | 2 | L | 0.82 |
| UXF-009 | Dialogs do not take focus, Escape does not close them | input | S2 | 6 of 11 | 2 | L | 0.82 |
| UXF-011 | In fixture mode five of six commands change nothing | input | S2 | 5 of 11 | 2 | L | 0.68 |
| UXF-014 | "Compare with picked" is offered with nothing picked | affordance | S2 | 1 of 11 | 2 | S | 0.55 |
| UXF-018 | The search fields and the document choice have no label | affordance | S3 | 5 of 11 | 2 | M | 0.45 |
| UXF-019 | Empty states say the wrong thing or nothing | copy | S3 | 5 of 11 | 1 | S | 0.45 |
| UXF-013 | Overlay forms are not filled from their row; every submit reads "Save" | affordance | S2 | 3 of 11 | 1 | M | 0.41 |
| UXF-006 | A live step that arrives as the climb opens is missing from the table | input | S2 | 1 of 11 | 2 | M | 0.27 |
| UXF-012 | The start form accepts an empty submit; its button is below the fold | affordance | S2 | 1 of 11 | 2 | M | 0.27 |
| UXF-020 | In the dark theme a multi-line field is drawn on an unthemed grey | layout | S3 | 2 of 11 | 1 | S | 0.18 |
| UXF-021 | A rule's strength "hard" is drawn like a failure | noise | S3 | 2 of 11 | 1 | S | 0.18 |
| UXF-022 | The step table grows and pushes "Frontier now" down | layout | S3 | 1 of 11 | 2 | M | 0.09 |
| UXF-007 | The progress chart has no title, no value axis, steps in arrival order | input | S2 | 1 of 11 | 1 | L | 0.07 |
| UXF-016 | Step times drop the seconds | copy | S3 | 1 of 11 | 1 | M | 0.05 |

## Known losses (rule 5)

Five findings are caused by input the studio does not control: the stock generator's rendering and
runtime, and fixture mode. They are recorded and not put into a change set; only the studio's
handling of them is.

| id | cause | the studio's handling |
|---|---|---|
| UXF-005 | `as: map` and `as: list` are drawn as JSON by the React target (`docs/ess-gaps.md`, "A list of records as text") | keep such values out of table columns: part of UXF-001 and UXF-002 |
| UXF-006 | a live event before a section's first answer; not diagnosed | none in this set; diagnose first |
| UXF-007 | the stock chart: order (`docs/ess-gaps.md`, item 6), no value axis | title the chart: a line of the document, outside this set |
| UXF-009 | the generated overlays have no focus or Escape handling | none in the document |
| UXF-011 | fixture mode applies one of six commands | the round's: a served backend or richer fixtures before gate 3 can be measured |

## The change set

At most 5 changes. Rule 1 puts every S1 in; rule 2 fills the last place by priority.

| # | finding | by rule | target metric | now | regression asset |
|---|---|---|---|---|---|
| 1 | UXF-003 filters never match | 1 (S1) | `trial.scenarios_not_completed`: `filter-targets` and the row expectation of `switch-axis` | missed in 8 of 8 runs | an ess-ui-test on the axis-switch rows and on a search (to add); the two trial scenarios |
| 2 | UXF-004 the live climb scrolls | 1 (S1) | `page_checks.always_visible.misses` on the live climb | 20 | the checks "stop campaign" and "campaign state" of `studio.checks.yaml` |
| 3 | UXF-001 the Owner's actions are out of view | 1 (S1) | `page_checks.always_visible.misses` on a promotion; looks of `owner-trade-off` | 4; 1 per run | the check "gate" of `studio.checks.yaml`; one more for the action (to add) |
| 4 | UXF-002 the switch action is clipped | 1 (S1) | `trial.runs_failed` | 2 | the trial scenario `switch-axis` at 1280x720 |
| 5 | UXF-017 small text and badge contrast | 2 (priority 6.00) | `audit.text_under_14px.share`; `audit.text_low_contrast.runs_per_page.max` | 0.564; 12 | the page audit |

Next by priority, outside the set: UXF-008 (2.73), UXF-010 (1.91), UXF-015 (1.50).

Rule 6, design decisions: changes 2, 3 and 4 are layout changes, and § 3.4 takes one design
decision per round. Default: the decision of the next round goes to UXF-001, the finding on an
anchor card; UXF-004 and UXF-002 stay in the set and wait their turn, and block no other change.
Changes 1 and 5 need no decision: a corrected expression and theme values. The owner may order it
otherwise.

The change set is not yet a task in the planning store: this round's story ends at the record.
