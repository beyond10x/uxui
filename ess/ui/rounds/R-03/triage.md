# R-03 triage

Applied by rule (`plugin/skills/ui-iteration-loop/SKILL.md` § 3.3) to the 20 findings open after the
round. Four of the five changes ordered for this round were built; the change set below is what the
next round would take.

## Caveats

- No person and no paid level took part: no recording, no questionnaire, no L4 replay, no L5 probe.
  "Two sources agree" means two of: the ux-browser page checks, the trial, the page audit, the
  heuristic review of R-01.
- The heuristic review was not repeated. Severity, frequency, confidence and cost of the 17 findings
  that R-02 left open and this round did not touch are R-01's and R-02's, unchanged. A frequency may
  be lower now: a page that shows its title or a decision in words was not looked at again for the
  other findings it carried.
- The three new findings were read from the trial's screenshots and numbers by the session that
  built the changes, not by a reviewer.
- R-03 was measured with the browser build of R-02 and ux-browser 0.11.0 (R-02: 0.10.0). R-02's
  input was measured again (`before.round.json`): every count and every page height is R-02's; the
  layout shift is not (`report.md`, Layout shift). "Before" below is R-02's value.
- The trial is a driver that names nodes by path, not the screen-only user of § 7.
- axe-core did not run. The app ran on its fixtures: five of the six commands the round sends change
  nothing there (UXF-011).
- Severity and cost are read as in R-01: S1 a hard gate fails or a scenario cannot be completed;
  S2 the person loses time or cannot tell the state; S3 the rest. S a few lines of
  `ess/ui/studio.ui.yaml` or `themes/studio.css`; M a section or a page of the document reworked;
  L a change in ESS or its generated runtime. One reading is new and the owner's to overturn:
  UXF-026 is S2 although gate 1 fails on its page (its record says why).

## Did the last change sets move their metrics?

| change set | finding | target metric | before | after | moved? |
|---|---|---|---|---|---|
| R-01's and R-02's | UXF-004 the live climb scrolls | `page_checks.always_visible.misses` on the live climb | 20 | 0 | yes |
| R-01's and R-02's | UXF-002 the switch action is clipped | `trial.runs_failed`; scrollable elements on a variant | 2; 3 | 0; 2 | yes |
| R-02's | UXF-008 four pages show no title | `audit.pages_without_h1` | 4 | 0 | yes |
| R-02's | UXF-010 a notice names the command | `trial.runs_notice_expectation_missed` (new) | 26 of 44 | 28 of 44 | no: the studio cannot build it |
| R-02's | UXF-015 labels are field names | none yet | | | not built: the round was ordered with UXF-005 in its place |
| this round's order | UXF-005 structured values as JSON | the two expectations of `owner-trade-off` on the decision and its dialog; `page_checks.overflow.blocks` | missed in 4 of 4 runs; 14 | met in 4 of 4; 14 | in part |

The round's totals: `trial.scenarios_not_completed` 5 to 4, `trial.runs_failed` 2 to 0,
`trial.looks.total` 12 to 8, `page_checks.always_visible.misses` 20 to 0, `audit.pages_without_h1`
4 to 0. `trial.runs_expectation_missed` stays 16: the four cards that miss an expectation miss it
for a known loss or for UXF-014. One metric is over R-02's value: `page_checks.layout_shift.max`,
0.0176 to 0.0284, the start form's value, which changes from run to run on the same input
(UXF-023; `report.md`, Layout shift).

UXF-010: its 28 is 2 over the 26 measured before the changes. No notice changed. The two runs are
`switch-axis` at 1280x720, which did not reach its command before UXF-002 closed.

Rule 4 (a change that has not moved its metric after two rounds is reviewed): no built change is
without movement. UXF-010 was looked into and cannot be built in the studio's document or theme
with ess 0.99.2; it moves to the known losses. UXF-015 has not been built; R-04 is its second round
in a set.

## Priorities

`priority = severity weight × frequency × confidence ÷ cost`, with S1 = 9, S2 = 3, S3 = 1 and
S = 1, M = 2, L = 4.

| id | title | class | severity | scenarios | confidence | cost | priority |
|---|---|---|---|---|---|---|---|
| UXF-015 | Labels are field names and values are ids | copy | S2 | 11 of 11 | 1 | M | 1.50 |
| UXF-010 | A success notice names the command, not what happened | copy | S2 | 7 of 11 | 2 | L | 0.95 |
| UXF-005 | Structured values are drawn as raw JSON | input | S2 | 6 of 11 | 2 | L | 0.82 |
| UXF-009 | Dialogs do not take focus, Escape does not close them | input | S2 | 6 of 11 | 2 | L | 0.82 |
| UXF-011 | In fixture mode five of six commands change nothing | input | S2 | 5 of 11 | 2 | L | 0.68 |
| UXF-014 | "Compare with picked" is offered with nothing picked | affordance | S2 | 1 of 11 | 2 | S | 0.55 |
| UXF-018 | The search fields and the document choice have no label | affordance | S3 | 5 of 11 | 2 | M | 0.45 |
| UXF-019 | Empty states say the wrong thing or nothing | copy | S3 | 5 of 11 | 1 | S | 0.45 |
| UXF-013 | Overlay forms are not filled from their row; every submit reads "Save" | affordance | S2 | 3 of 11 | 1 | M | 0.41 |
| UXF-006 | A live step that arrives as the climb opens is missing from the table | input | S2 | 1 of 11 | 2 | M | 0.27 |
| UXF-012 | The start form accepts an empty submit; its button is below the fold | affordance | S2 | 1 of 11 | 2 | M | 0.27 |
| UXF-026 | The live climb is taller than the viewport (new) | layout | S2 | 1 of 11 | 2 | M | 0.27 |
| UXF-020 | In the dark theme a multi-line field is drawn on an unthemed grey | layout | S3 | 2 of 11 | 1 | S | 0.18 |
| UXF-021 | A rule's strength "hard" is drawn like a failure | noise | S3 | 2 of 11 | 1 | S | 0.18 |
| UXF-022 | The step table grows and pushes "Frontier now" down | layout | S3 | 1 of 11 | 2 | M | 0.09 |
| UXF-024 | A notice lies over the action of the last table row (new) | layout | S3 | 1 of 11 | 1 | S | 0.09 |
| UXF-007 | The progress chart has no title, no value axis, steps in arrival order | input | S2 | 1 of 11 | 1 | L | 0.07 |
| UXF-016 | Step times drop the seconds | copy | S3 | 1 of 11 | 1 | M | 0.05 |
| UXF-023 | The start form moves as it loads; its layout shift changes from run to run | input | S3 | 1 of 11 | 2 | L | 0.05 |
| UXF-025 | A requirement and its verdict are paired only by place (new) | copy | S3 | 1 of 11 | 1 | L | 0.02 |

Changed against R-02's table: UXF-010 from cost M (1.91) to cost L (0.95), now that the cause is
read. Closed in this round and out of the table: UXF-004, UXF-002, UXF-008.

## Known losses (rule 5)

Six: the five of R-01 (UXF-005, UXF-006, UXF-007, UXF-009, UXF-011) and, new, UXF-010. This round
met three of them:

| id | where the round met it | the studio's handling |
|---|---|---|
| UXF-005 | the decision of a promotion and the dialog that accepts its trade-off | built in this round: one field per member of each record, the JSON fields not drawn in the React app. Every other structured value is JSON as before |
| UXF-010 | every notice of the seven cards that send a command | none possible: the words are the generated runtime's. The round now counts them |
| UXF-011 | `owner-trade-off` stays missed: the state does not change after the accept | none; the round's: a served backend or richer fixtures |

Candidates, as their records say: UXF-023 (the suggestions of a form field have no element until
their read answers) and UXF-025 (the generator has no drawing of a list of records but JSON).

## The change set

At most 5 changes. Rule 1 puts every S1 in: none is open. Rule 2 fills the places by priority; of a
known loss only the studio's handling can be a change (rule 5), so UXF-010, UXF-009 and UXF-011
take no place.

| # | finding | by rule | target metric | now | regression asset |
|---|---|---|---|---|---|
| 1 | UXF-015 labels are field names | 2 (priority 1.50) | none yet | | a heuristic checklist line (its record) |
| 2 | UXF-005, its handling for maps: the budget, the first route and the diff as one field per member | 2 (priority 0.82) and 5 | `page_checks.overflow.blocks` | 14 | the page check `layout.overflow@1` on the live climb and on a variant; `live-climb`, its expectation on the budget |
| 3 | UXF-014 "Compare with picked" with nothing picked | 2 (priority 0.55) | `trial.scenarios_not_completed` | 4 | the trial scenario `compare-without-pick` |
| 4 | UXF-018 search fields without a label | 2 (priority 0.45) | `audit.inputs_placeholder_only.pages` | 4 | the page audit; a bar in `studio.audit.json` (to add) |
| 5 | UXF-019 empty states say the wrong thing | 2 (priority 0.45) | none yet | | to add: an expectation in `filter-targets` at the moment "no-match" |

Next by priority, outside the set: UXF-013 (0.41), UXF-012 (0.27), UXF-026 (0.27).

Change 2 in detail. All 14 overflowing blocks are maps drawn as JSON: the budget on the live climb
(10 units) and the first route of a variant (4 units). Both renderers read a member through an
object (`budget.max_variants`), so the fields can be declared once and need nothing of the theme,
unlike the lists of this round. It changes what the live climb draws: the test "the budget reads as
a map and the KPI values as a list" and the card `live-climb` state the budget as one text and are
restated with it. The terminal drawing of that page is also what the golden file
`engine/search/tests/fixtures/studio.frontier.txt` is computed from: one field more in the summary
can change its numbers, and the file is then generated again with the command its test names.

Rule 6, design decisions: none of the five is a layout change. Three findings outside the set need
one when their turn comes: UXF-026 (fit the climb into the viewport), UXF-024 (where notices are
drawn) and UXF-012 (the start form's button below the fold).

Open from this round: the answer to `decision.md` (default B, built). Open from R-02: the answer to
`../R-02/decision.md` (default B, built).

The change set is not yet a task in the planning store: this round's story ends at the record.
