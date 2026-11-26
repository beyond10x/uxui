# R-04 decision: take B? The live climb fits the viewport: two rows of two sections, the step table last

Finding: UXF-026. Target metric: `audit.pages_with_document_scroll` (on `/campaigns/climb/cp-01`:
`audit.cells_over_a_page_bar`).

A (now, "no"): the live climb as on `main` since R-03. Five sections stand under each other in the
main column, 1249 px of content in a viewport of 768 or 720 px; the header and the campaign's summary
stay in view while the page scrolls (R-03's B). The step table and "Frontier now" open below the fold,
and the step table pushes "Frontier now" down with every live step. Shot: `shots/decision-a.png`

B (change, "yes"): from 78rem of width the main column is two rows: the KPIs beside the chart of the
best value, the step table beside "Frontier now"; the side column is 11rem wide and the chart 7rem
high. The progress chart is not drawn there: its four series (evaluated, pruned, insufficient,
frontier size) are four columns of the step table. Under 78rem the columns stand as in A, with the
step table moved to the end. Rules of `themes/studio.css` under "The live climb fits the viewport";
the document's sections and layout are unchanged, so the terminal renderer draws what it drew.
Shot: `shots/decision-b.png`

Both shots are the same moment of the same card: `live-climb`, "frontier-now", at 1366x768 in the
light theme, after the driver revealed "Frontier now" (A scrolled 515 px, B not scrolled).

A is `before.round.json`: the document and theme of R-03 measured again in this round, with the
regression assets of this round. B is `round.json`. B was measured together with the other changes of
the round (the maps of UXF-005, the start form of UXF-023), so only the numbers of the live climb are
B's alone; of those, the summary also draws the budget as two fields (UXF-005), which does not change
whether the page fits.

| step of the order | A | B | separates? |
|---|---|---|---|
| 1 hard gates (§ 4) | gates 1 and 3 fail; gate 2 passes, 0 misses | gates 1 and 3 fail; gate 2 passes, 0 misses | no: B fails no gate that A passes |
| 2 open S1, S2 findings (`found_on`, § 5) | 0, 11 | 0, 10 | yes |
| 3 ratchet (§ 6) | the live climb 529 px past the viewport (4 of 4 cells over its page bar); 2 pages scroll; 1 look per run of `live-climb`; its layout shift 0.0009 to 0.013 | 0 px, 0 cells; 1 page scrolls; 0 looks; 0.0051 to 0.0075 in 5 runs; no key worse | yes |
| 4 survey (§ 3.1) | none: no person took part | none | no |
| 5 reviewers | none ran | none ran | no |

Reviewers:
- heuristic reviewer: no view, the review was not run in this round
- report reviewer: no view, the review was not run in this round

Trade-off: none measured. No key of the round is worse with B than with A on the same machine
(`round.json` against `before.round.json`; `triage.md`). What B costs and no metric counts: at the
gating viewports the live climb does not draw the progress chart, and its chart of the best value is
7rem high where it was 10rem; the campaign's summary is 11rem wide, so "layout model version" takes
two lines.

Decision id: R-04. Written beside no baseline value: the studio keeps no baseline file, and B makes no
key worse.

Recommendation: B, from step 2.

Default if unanswered by the next round: B, because the order recommends it and B makes no key worse.

Answer: none yet. B is built in the change of this round. An answer A takes it back: remove the rules
under "The live climb fits the viewport" from `themes/studio.css`; UXF-026 and UXF-022 are open again.
An answer B may take the progress chart out of the document as well (it is then gone from the
terminal drawing too, and `engine/search/tests/fixtures/studio.frontier.txt` is generated again).

## How the table was filled

- Step 1 is read as in R-02 and R-03: B fails no gate that A passes. Gate 1 fails in both, on other
  pages: the start form is 1077 px tall (UXF-012), and a variant and a comparison have scrolling
  elements (2 pages in both). On the live climb gate 1 fails with A and passes with B.
- Step 2 counts the records of `ess/ui/findings/` after this round. S2 with A: the 10 that are open
  in both (UXF-005, UXF-006, UXF-007, UXF-009, UXF-010, UXF-011, UXF-012, UXF-013, UXF-014, UXF-015)
  and UXF-026, which B closes. B also closes UXF-022 (S3, the step table pushes "Frontier now" down).
  UXF-023 (S3) closed in this round by a change that is not B, so neither version counts it.
- Step 3: there is no baseline file to ratchet against. The keys are the metrics of the two
  `round.json` files. The looks: with A the driver scrolls once per run to reach the step table; with B
  nothing it reveals is out of view.

## What B is, and what it is not

- It holds at the gating viewports and at the two measured ones that are 78rem wide or more
  (1366x657, 1536x754): the page is as tall as the viewport, and no element of it scrolls.
- It holds while the step table has six steps (the trial's last moment, `live-again`, at 1280x720).
  The table has no bound of its own (`max_rows: 200`): a campaign with more steps makes the page grow
  past the fold again, now below the step table only.
- At 960x1080 (measured, not gating) nothing changes but the order: the step table is last.
- The progress chart stays in the document. A person who wants the four series as lines finds them
  under 78rem of width only; at the gating sizes the step table gives the same numbers per step.
- "Frontier now" keeps its KPI values as JSON (UXF-005): it is 3/7 of the main column, wide enough for
  them without a scrolling table at 1280x720.
- Keyboard: the order of the DOM is unchanged; the step table has no control, so moving it last does
  not change the Tab order of anything that takes focus.
