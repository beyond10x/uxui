# R-02 decision: take B? The gate decision of a promotion as one card, its actions beside the verdict

Finding: UXF-001. Target metric: `page_checks.always_visible.misses` on `/promotions/detail/pr-02`.

A (now, "no"): the decision stays one table row of six columns, 1797 px wide in a frame of 1042 px;
the Owner's actions are outside the frame until it is scrolled, and then the verdict is.
Shot: `shots/decision-no.png`

B (change, "yes"): the decision is a list row (`style: list` on the section "decision" of
`promotions.detail`): labelled fields in columns, "Accept", "Accept trade-off" and "Reject" beside
the verdict. Shot: `shots/decision-yes.png`

A is `before.round.json`: the document and theme of R-01 measured again in this round, with the
check "owner action" added. B is `round.json`. B was measured together with the two other changes of
the round (UXF-003, UXF-017), so only the numbers of the promotion page are B's alone.

| step of the order | A | B | separates? |
|---|---|---|---|
| 1 hard gates (§ 4) | gates 1, 2 and 3 fail; of gate 2, 20 misses on this page | gates 1, 2 and 3 fail, for what other pages show; of gate 2, 0 misses on this page | no: B fails no gate that A passes |
| 2 open S1, S2 findings (`found_on`, § 5) | 3, 11 | 2, 11 | yes |
| 3 ratchet (§ 6) | 20 misses (verdict 4, action 16); 1 look per run of `owner-trade-off`; 1 scrollable element | 0 misses; 0 looks; 0 scrollable elements; no key worse | yes |
| 4 survey (§ 3.1) | none: no person took part | none | no |
| 5 reviewers | none ran | none ran | no |

Reviewers:
- heuristic reviewer: no view, the review was not run in this round
- report reviewer: no view, the review was not run in this round

Trade-off: none measured. No key of the round is worse with B than with A on the same machine
(`report.md`, Metrics). Not measured: how the three structured values read. In A each is one line of
JSON, cut off by the frame; in B each is JSON broken where its column ends (UXF-005, a known loss in
both).

Decision id: R-02. Written beside no baseline value: the studio keeps no baseline file yet, and B
makes no key worse.

Recommendation: B, from step 2.

Default if unanswered by the next round: B, because the order recommends it and B makes no key
worse. The round was ordered with yes as its default.

Answer: B, the owner, 2026-11-26 (D13). B is built in the change of this round and stays. An answer
A would have taken it back: remove `style: list` from the section, and UXF-001 is open again.

## How the table was filled

- Step 1 reads "B fails one: A" for an A that passed its gates. Here A fails three, and B fails the
  same three by the numbers of other pages (the live climb, the start form, a variant). The step is
  read as: B fails no gate that A passes.
- Step 2 counts the records of `ess/ui/findings/` after this round. Open with A: S1 UXF-001, UXF-002,
  UXF-004; S2 the 11 that R-01 left open besides UXF-017. With B, UXF-001 is closed.
- Step 3: there is no baseline file to ratchet against. The keys are the metrics of the two
  `round.json` files.
