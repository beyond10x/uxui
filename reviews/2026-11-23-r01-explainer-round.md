# R-01 on the explainer page: what one round of the skill showed

2026-11-23. One round of `ui-iteration-loop` applied to a one-file explainer page of the skill itself. The
harness, round dir and page are in the session scratch directory (not in this repository); numbers below are
from its `rounds/R-01/` files, its two review files and the build agent's report.

## The round

- Measure: 304 Playwright tests; at the gating viewports (1440x900, 1280x800) 20 of 74 gate tests red.
- Findings: 5 from the round command (3 S1: SVG steps, div tiles, unlabelled inputs), 7 from the heuristic
  reviewer (6 pass, 9 partial, 1 fail of 16 heuristics), 1 duplicate from the report reviewer.
- Triage: 5 changes (the 3 S1s, narrow layout, in-page navigation via decision D1, default yes).
- Build: failing-first per finding; after it every gate green at gating and measured-only viewports (372
  passed); 6 findings closed, 6 open.
- Time: browser runs took 1.8 min and 50.8 s; queue waits for the shared build pool took 30, 12, 18 and 20 min.

## Evidence for items of the critique (`2026-11-23-ui-iteration-loop-critique.md`)

- Item 6/8 (ratchet blocks trade-offs): the navigation the reviewers asked for made 20 of 168 gating values
  worse (a link click is one action more per pointer scenario; 12 links add Tab stops), while `looks` fell
  23 → 0. It landed only through a decision id. The ratchet treats a better page as a regression to excuse.
- Item 1 (Goodhart, proxy metrics): `reading_load` is "words in the end viewport"; it moved +2 to +80 in 8
  walks only because the walk stopped scrolling elsewhere.
- Item 18 (triage formula): UXF-007, the demo prints a false verdict, ranked 0.50 and stayed out of the set;
  the formula has no term for a false statement on the page.
- Item 10 (the stick is not independent): the build raised 4 scenarios' action bars to fit the change, with
  the decision cited. Legitimate here; nothing stops it otherwise.
- Item 13/14 (AI judges AI): the independent report reviewer found a real tool defect (T1: an undone task
  counted as no miss; true misses 12, reported 2) and an empty first ratchet pass (T5). The two reviewers
  overlapped on one finding (no navigation), which the formula counted as confidence 2.
- Item 3 (axe share): axe was 0 after the fix, but the heuristic reviewer still found text under 14 px, red
  used for both "gate" and "failure", and undefined shorthand: none machine-checked.
- Item 22 (cost): the harness (tools, lib, tests, scenarios) is larger than the page it measured.

## Open after R-01

- Findings UXF-004, 006, 007, 010, 011, 012; tool defects T2-T4, T6-T8, T10.
- The fast loop ran 182 s against the 120 s budget once, at load 85-90 on 20 CPUs (50.8 s on a quieter run).
- No skip link before the 12 navigation links; nothing measures it.
