# R-03 decision: take B? The page header and the narrow column stay in view while a page scrolls

Finding: UXF-004. Target metric: `page_checks.always_visible.misses` on `/campaigns/climb/cp-01`.

A (now, "no"): the live climb is 1249 px of content and scrolls as one sheet. Scrolled to the step
table or to "Frontier now", the header with Stop and the connection state, and the summary with the
campaign state, are out of view. Shot: `shots/decision-no.png`

B (change, "yes"): the header of a page (its title, the connection state, its actions) stays at the
top of the viewport, and the narrow column beside the main column stays under it; the main column
scrolls past both. A rule of `themes/studio.css` for every page with a header; the document's
layout is unchanged. Shot: `shots/decision-yes.png`

Both shots are the same moment of the same card: `live-climb`, "reconnecting seen from below", at
1366x768 in the light theme, the page scrolled 515 px.

A is `before.round.json`: the document and theme of R-02 measured again in this round, with the
regression assets of this round. B is `round.json`. B was measured together with the other changes
of the round (UXF-002, UXF-008, UXF-005), so only the numbers of the live climb are B's alone.

| step of the order | A | B | separates? |
|---|---|---|---|
| 1 hard gates (§ 4) | gates 1, 2 and 3 fail; of gate 2, 20 misses, all on this page | gates 1 and 3 fail; gate 2 passes, 0 misses | no: B fails no gate that A passes |
| 2 open S1, S2 findings (`found_on`, § 5) | 1, 10 | 0, 11 | yes |
| 3 ratchet (§ 6) | 20 misses (Stop 10, campaign state 10); 2 looks per run of `live-climb` | 0 misses; 1 look per run; no count worse | yes |
| 4 survey (§ 3.1) | none: no person took part | none | no |
| 5 reviewers | none ran | none ran | no |

Reviewers:
- heuristic reviewer: no view, the review was not run in this round
- report reviewer: no view, the review was not run in this round

Trade-off: none measured. No count of the round is worse with B than with A on the same machine
(`report.md`, Metrics). `page_checks.layout_shift.max` is the start form's value in both and changes
from run to run on unchanged input (UXF-023; `report.md`, Layout shift). What B costs and no metric
counts: on a page that scrolls, the header takes 58 px of the viewport for good, and on the live
climb the summary takes the left third of the width at every scroll position.

Decision id: R-03. Written beside no baseline value: the studio keeps no baseline file, and B makes
no count worse.

Recommendation: B, from step 2.

Default if unanswered by the next round: B, because the order recommends it and B makes no count
worse.

Answer: none yet. B is built in the change of this round. An answer A takes it back: remove the
rules of `.ui-header` and `.ui-layout-columns > .ui-column-narrow` under "A page that scrolls keeps
its header in view" from `themes/studio.css`; UXF-004 is open again and UXF-026 is closed with it.

## How the table was filled

- Step 1 is read as in R-02: B fails no gate that A passes. B passes one gate that A fails. Gate 1
  fails in both: with B the live climb is as tall as with A (1249 px in 768), and the page still
  scrolls. B does not fit the climb into the viewport; it keeps in view what the document declares
  `always` visible.
- Step 2 counts the records of `ess/ui/findings/` after this round. S1 with A: UXF-004. With B it
  is closed, and what is left of it is UXF-026 (S2): the step table and "Frontier now" still open
  below the fold. S2 with A: the 10 that are open in both (UXF-005, UXF-006, UXF-007, UXF-009,
  UXF-010, UXF-011, UXF-012, UXF-013, UXF-014, UXF-015). UXF-002 and UXF-008 closed in this round by
  changes that are not B, so neither version counts them.
- Step 3: there is no baseline file to ratchet against. The keys are the metrics of the two
  `round.json` files. The 20 misses of A are one per unit: 5 viewports, 2 themes, 2 items, at the
  bottom of the page. The looks: with A the driver scrolls once to the step table and once back to
  the header to read the connection state; with B the header is in view and it scrolls once.

## What B is, and what it is not

- The header is opaque and reaches over the padding of the page, so what scrolls passes under it.
  The page's background glow is fixed to the viewport, so that the header carries the same picture
  without a seam.
- The rule holds for every page with a header. On the eight pages that fit the gating viewports
  nothing moves. The start form scrolls too (1077 px): its header, which holds only the title,
  stays in view.
- The narrow column stays only where columns stand beside each other. Under 768 px of width the
  generated stylesheet puts the columns under each other; there only the header stays, and the
  campaign state scrolls away. No measured viewport is that narrow (the narrowest is 960x1080).
- A narrow column taller than the viewport would keep its lower part out of reach while the main
  column scrolls. The two narrow columns of the studio are 367 px (the live climb) and 592 px (a
  variant) tall; the smallest measured viewport is 657 px high and leaves 595 px under the header.
- Keyboard: `scroll-padding-top` on the root keeps a focused control from landing under the
  header. The page checks find 0 keyboard misses with B, as with A.
