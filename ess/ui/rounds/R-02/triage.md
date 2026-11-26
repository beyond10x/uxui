# R-02 triage

Applied by rule (`plugin/skills/ui-iteration-loop/SKILL.md` § 3.3) to the 20 findings open after the
round. Three changes of R-01's change set were built in this round; the change set below is what
the next round would take.

## Caveats

- No person and no paid level took part: no recording, no questionnaire, no L4 replay, no L5 probe.
  "Two sources agree" means two of: the ux-browser page checks, the trial, the page audit, the
  heuristic review of R-01.
- The heuristic review was not repeated. Severity, frequency, confidence and cost of the 19 findings
  that R-01 left open are R-01's, unchanged. A frequency may be lower now: a scenario that no longer
  shows an empty list or a small badge was not looked at again for the other findings it carried.
- R-02 was measured on another machine than R-01, with the same browser build. R-01's input was
  measured again on it (`before.round.json`): every count is equal, pixel values differ by up to
  40 px, and the layout shift differs by more than its tolerance (`report.md`, "Where it was
  measured"). "Before" below is R-01's value; where the value measured again differs, it follows
  in brackets.
- The trial is a driver that names nodes by path, not the screen-only user of § 7.
- axe-core did not run. The app ran on its fixtures: five of the six commands the round sends change
  nothing there (UXF-011).
- Severity and cost are read as in R-01: S1 a hard gate fails or a scenario cannot be completed;
  S2 the person loses time or cannot tell the state; S3 the rest. S a few lines of
  `ess/ui/studio.ui.yaml` or `themes/studio.css`; M a section or a page of the document reworked;
  L a change in ESS or its generated runtime.

## Did the last change sets move their metrics?

| change set | finding | target metric | before | after | moved? |
|---|---|---|---|---|---|
| R-01's | UXF-003 filters never match | `trial.scenarios_not_completed`: `filter-targets` and the row expectation of `switch-axis` | missed in 8 of 8 runs | met in 8 of 8; `filter-targets` completed in 4 of 4 | yes |
| R-01's | UXF-004 the live climb scrolls | `page_checks.always_visible.misses` on the live climb | 20 | 20 | not built |
| R-01's | UXF-001 the Owner's actions are out of view | `page_checks.always_visible.misses` on a promotion; looks of `owner-trade-off` | 4 (20 with the check for the action); 1 per run | 0; 0 per run | yes |
| R-01's | UXF-002 the switch action is clipped | `trial.runs_failed` | 2 | 2 | not built |
| R-01's | UXF-017 small text and badge contrast | `audit.text_under_14px.share`; `audit.text_low_contrast.runs_per_page.max` | 0.564; 12 | 0; 0 | yes |

The round's totals: `trial.scenarios_not_completed` 6 to 5, `trial.runs_expectation_missed` 22 to 16,
`trial.looks.total` 16 to 12, `page_checks.always_visible.misses` 24 to 20,
`audit.pages_with_scrollable_element` 3 to 2. One metric is worse than R-01 recorded it:
`page_checks.layout_shift.max`, 0.0151 to 0.0176 (0.026 when R-01's input is measured again). Its
value changes from run to run on the same input: UXF-023.

Rule 4 (a change that has not moved its metric after two rounds is reviewed): no change has had two
rounds. UXF-004 and UXF-002 were not built; this is their first round in a set.

## Priorities

`priority = severity weight × frequency × confidence ÷ cost`, with S1 = 9, S2 = 3, S3 = 1 and
S = 1, M = 2, L = 4.

| id | title | class | severity | scenarios | confidence | cost | priority |
|---|---|---|---|---|---|---|---|
| UXF-008 | Four pages show no title | copy | S2 | 5 of 11 | 2 | S | 2.73 |
| UXF-010 | A success notice names the command, not what happened | copy | S2 | 7 of 11 | 2 | M | 1.91 |
| UXF-004 | The live climb scrolls; Stop and the states leave the viewport | layout | S1 | 2 of 11 | 2 | M | 1.64 |
| UXF-015 | Labels are field names and values are ids | copy | S2 | 11 of 11 | 1 | M | 1.50 |
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
| UXF-023 | The start form moves as it loads; its layout shift changes from run to run | input | S3 | 1 of 11 | 2 | L | 0.05 |

Closed in this round and out of the table: UXF-003, UXF-017, UXF-001.

## Known losses (rule 5)

The five of R-01 stay: UXF-005, UXF-006, UXF-007, UXF-009, UXF-011. This round met two of them
again and did not work around either:

| id | where the round met it | the studio's handling |
|---|---|---|
| UXF-005 | the decision card of a promotion still shows three values as JSON, now broken at the column's end | none in this round; the card shows them, it does not hide them |
| UXF-011 | `owner-trade-off` stays missed: the state does not change after the accept | none; the round's: a served backend or richer fixtures |

Candidate: UXF-023. The suggestions of a form field have no element until their read answers
(`TagsInput` of the generated runtime), so the theme cannot keep their place. The studio's handling
would be a change of its own document: the target beside the form.

## The change set

At most 5 changes. Rule 1 puts every S1 in; rule 2 fills the other places by priority.

| # | finding | by rule | target metric | now | regression asset |
|---|---|---|---|---|---|
| 1 | UXF-004 the live climb scrolls | 1 (S1) | `page_checks.always_visible.misses` on the live climb | 20 | the checks "stop campaign" and "campaign state" of `studio.checks.yaml` |
| 2 | UXF-002 the switch action is clipped | 1 (S1) | `trial.runs_failed`; `audit.pages_with_scrollable_element` | 2; 2 | the trial scenario `switch-axis` at 1280x720 |
| 3 | UXF-008 four pages show no title | 2 (priority 2.73) | `audit.pages_without_h1` | 4 | the page audit; a bar `h1` in `studio.audit.json` (to add: the bars file counts "at most", this one is "at least") |
| 4 | UXF-010 a notice names the command | 2 (priority 1.91) | none yet | | to add: an expectation in the scenario cards that no notice contains "uxsearch." |
| 5 | UXF-015 labels are field names | 2 (priority 1.50) | none yet | | a heuristic checklist line (its record) |

Next by priority, outside the set: UXF-014 (0.55), UXF-018 (0.45), UXF-019 (0.45).

Rule 6, design decisions: changes 1 and 2 are layout changes, and § 3.4 takes one design decision
per round. Default: the decision of the next round goes to UXF-004, the one with the higher priority
and the 20 gate misses; UXF-002 stays in the set and waits its turn. Changes 3, 4 and 5 need no
decision: titles and wording in the document. The owner may order it otherwise.

Open from this round: the answer to `decision.md` (default B, built).

The change set is not yet a task in the planning store: this round's story ends at the record.
