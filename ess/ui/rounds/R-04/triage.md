# R-04 triage

Applied by rule (`plugin/skills/ui-iteration-loop/SKILL.md` § 3.3) to the 17 findings open after the
round. The three changes ordered for this round (the owner's decision D20) were built; the change set
below is what the next round would take.

## Caveats

- No person and no paid level took part: no recording, no questionnaire, no L4 replay, no L5 probe.
  "Two sources agree" means two of: the ux-browser page checks, the trial, the page audit, the
  heuristic review of R-01.
- The heuristic review was not repeated. Severity, frequency, confidence and cost of the findings this
  round did not touch are those of R-01 to R-03, unchanged.
- R-04 was measured with Chrome for Testing 159.0.8154.6 (R-03: 158.0.8127.18) and the same ess,
  node, react, esbuild, typescript and ux-browser 0.11.0. R-03's input was measured again on it
  (`before.round.json`): every count of R-03 is equal, and so is the layout shift of the first run,
  0.0284. "Before" below is R-03's value, or the value measured again where R-03 had no such key.
- The trial is a driver that names nodes by path, not the screen-only user of § 7.
- axe-core did not run. The app ran on its fixtures: five of the six commands the round sends change
  nothing there (UXF-011).
- Severity and cost are read as in R-01 to R-03.

## Did the last change sets move their metrics?

| change set | finding | target metric | before | after | moved? |
|---|---|---|---|---|---|
| R-03's built changes | UXF-004 the live climb scrolls, Stop leaves the viewport | `page_checks.always_visible.misses` | 0 | 0 | held |
| R-03's built changes | UXF-002 the switch action is clipped | `trial.runs_failed` | 0 | 0 | held |
| R-03's built changes | UXF-008 four pages show no title | `audit.pages_without_h1` | 0 | 0 | held |
| R-03's built changes | UXF-005, the decision of a promotion | `owner-trade-off` steps 4 and 10 | met in 4 of 4 | met in 4 of 4 | held |
| R-03's built changes | UXF-010 a notice names the command (known loss) | `trial.runs_notice_expectation_missed` | 28 of 44 | 28 of 44 | no: the studio cannot build it |
| this round's order (D20) | UXF-026 the live climb is taller than the viewport | `audit.cells_over_a_page_bar` (new); `audit.pages_with_document_scroll` | 4; 2 | 0; 1 | yes |
| this round's order (D20) | UXF-005, the maps | `page_checks.overflow.blocks`; the new no-JSON expectations | 14; missed in 4 of 4 runs each | 0; met in 4 of 4 | yes |
| this round's order (D20) | UXF-023 the start form moves; its layout shift changes from run to run | `page_checks.layout_shift.spread` (new); `page_checks.layout_shift.max` | 0.0121; 0.0284 | 0.0024; 0.0075 | yes |
| R-03's change set | UXF-015 labels are field names | none yet | | | not built: D20 ordered this round's set |
| R-03's change set | UXF-014, UXF-018, UXF-019 | as R-03's triage names them | | | not built: D20 ordered this round's set |

The round's totals against R-03: `page_checks.overflow.blocks` 14 to 0,
`page_checks.layout_shift.max` 0.0284 to 0.0075, `audit.pages_with_document_scroll` 2 to 1,
`trial.looks.total` 8 to 4. Every other count is equal. `trial.scenarios_not_completed` is 4 as in
R-03; on R-03's input with this round's assets it is 6 (the new expectations of `switch-axis` and
`compare-and-judge` miss there).

Also closed by the yes side of `decision.md`: UXF-022 (S3, the step table pushes "Frontier now" down),
its value at 960x1080 0.013 in 3 of 5 runs before, 0.0004 after.

Rule 4 (a change that has not moved its metric after two rounds is reviewed): no built change is
without movement. UXF-010 stays a known loss. UXF-015 has been in two change sets and was not built in
either, both times because an order of the owner set the round; it has not failed to move, it has not
been tried.

## Priorities

`priority = severity weight × frequency × confidence ÷ cost`, with S1 = 9, S2 = 3, S3 = 1 and
S = 1, M = 2, L = 4.

| id | title | class | severity | scenarios | confidence | cost | priority |
|---|---|---|---|---|---|---|---|
| UXF-015 | Labels are field names and values are ids | copy | S2 | 11 of 11 | 1 | M | 1.50 |
| UXF-010 | A success notice names the command, not what happened | copy | S2 | 7 of 11 | 2 | L | 0.95 |
| UXF-005 | Structured values are drawn as raw JSON (the lists of records) | input | S2 | 6 of 11 | 2 | L | 0.82 |
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
| UXF-024 | A notice lies over the action of the last table row | layout | S3 | 1 of 11 | 1 | S | 0.09 |
| UXF-007 | The progress chart has no title, no value axis, steps in arrival order | input | S2 | 1 of 11 | 1 | L | 0.07 |
| UXF-016 | Step times drop the seconds | copy | S3 | 1 of 11 | 1 | M | 0.05 |
| UXF-025 | A requirement and its verdict are paired only by place | copy | S3 | 1 of 11 | 1 | L | 0.02 |

Changed against R-03's table: closed and out of it, UXF-026, UXF-023 and UXF-022. UXF-005 keeps its
priority: the maps are words now, the lists of records on the same six scenarios are not. UXF-007 is
unchanged: the progress chart is not drawn at the gating viewports with B, but it is in the document
and drawn under 78rem of width.

## Known losses (rule 5)

Six, as after R-03: UXF-005 (now the lists of records alone), UXF-006, UXF-007, UXF-009, UXF-010,
UXF-011. What the studio can still do about UXF-005 needs a field path through a list (LR-D1,
`docs/ess-gaps.md`); the round adds none.

## The change set

At most 5 changes. Rule 1 puts every S1 in: none is open. Rule 2 fills the places by priority; of a
known loss only the studio's handling can be a change (rule 5), and UXF-010, UXF-005, UXF-009 and
UXF-011 have none left that the document or the theme can build.

| # | finding | by rule | target metric | now | regression asset |
|---|---|---|---|---|---|
| 1 | UXF-015 labels are field names | 2 (priority 1.50) | none yet | | a heuristic checklist line (its record) |
| 2 | UXF-014 "Compare with picked" with nothing picked | 2 (priority 0.55) | `trial.scenarios_not_completed` | 4 | the trial scenario `compare-without-pick` |
| 3 | UXF-018 search fields without a label | 2 (priority 0.45) | `audit.inputs_placeholder_only.pages` | 4 | the page audit; a bar in `studio.audit.json` (to add) |
| 4 | UXF-019 empty states say the wrong thing | 2 (priority 0.45) | none yet | | to add: an expectation in `filter-targets` at the moment "no-match" |
| 5 | UXF-013 overlay forms not filled from their row | 2 (priority 0.41) | none yet | | to add: expectations on the propose and judge forms |

Next by priority, outside the set: UXF-006 and UXF-012 (0.27).

Rule 6, design decisions: none of the five is a layout change. UXF-012 (the start form's button below
the fold, now the one page with a document scroll) and UXF-024 (where notices are drawn) need one when
their turn comes.

Open from this round: the answer to `decision.md` (default B, built). Open from R-03: the answer to
`../R-03/decision.md` (default B, built).

The change set is not yet a task in the planning store: this round's story ends at the record.
