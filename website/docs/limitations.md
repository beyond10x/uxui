---
title: Limitations
sidebar_position: 92
description: What the plugin's evidence does not show, what the engine's design does not settle, and the risks the roadmap names.
---

# Limitations

[Status](./status.md) lists what exists. This page lists what that does not establish. Its sources
are the risks in `docs/roadmap.md`, the critique `reviews/2026-11-23-ui-iteration-loop-critique.md`
and the round report `reviews/2026-11-23-r01-explainer-round.md`; the reviews are in the
[repository](https://github.com/beyond10x/uxui/tree/main/reviews).

## The engine is a first search on a toy page

What exists of the search engine is one search on layout model v0, the terminal renderer of ESS
driven in process: `ux-search climb` takes one page of an ESS example, writes every variant of two
axes (theme and section order), checks, opens and draws each at two sizes, and prints a ranked
frontier as text. Its terms are counts read from the drawing (check errors, regions below the fold,
regions cut by the frame, rows above the primary action); themes tie, because the terminal renderer
reads no theme. `ux-search cost` measures what one evaluation costs per UI state; its numbers are
not measured yet.

One campaign has run in a browser: the engine's first on the browser adapter, over this site's
landing page (144 variants, 864 renders, a frontier of 16; the page as shipped ranks 35th). A
person's pick promoted as a pull request does not exist yet.

Everything else about the engine on this site is a design. No search explores UI states: each
variant is its page as opened. No climb reaches the studio. The ESS
specification is valid and compiled, but has no scenarios. The numbers in the design (evaluation
counts, seconds per stage) rest on an assumed 50 µs per layout call and one measurement on a loaded
host; the roadmap's step 3 replaces them.

## The plugin has been applied once

The skill has run one round, on its own explainer page (R-01), with the hermetic levels and two AI
reviewers only: no person, no questionnaire, no keyed replay. What that round showed:

- **Trade-offs are blocked.** A navigation the reviewers asked for cut one value from 23 to 0 and
  made 20 of 168 gating values worse. It landed only through a decision id: the ratchet treats a
  better page as a regression to excuse.
- **A metric measured the walk, not the page.** `reading_load` ("words in the end viewport") moved
  by +2 to +80 in 8 walks because the walk stopped scrolling elsewhere. The engine's catalogue
  replaces it with a count at a fixed point.
- **One rule was counted many times.** An accessibility rule was counted once per journey, 12 times.
- **The apparatus is larger than the page.** The harness of tools, scenarios and tests was larger
  than the page it measured; browser runs took up to 1.8 minutes and the fast loop once ran 182 s
  against a 120 s budget.

## What the loop's metrics cannot see

- Every metric is a proxy. Lower-is-better values reward removing content; nothing measures
  comprehension, correct content, trust or aesthetics. A page stating wrong facts passes every gate.
- axe-core finds only the machine-checkable share of accessibility issues. 0 violations is not
  "accessible": there is no screen-reader run in the loop.
- Headless Chromium with its fonts is the only renderer; other engines, zoom and real DPI are not
  measured.
- The screen-only user finds controls by accessible role and name and is told its target. It tests
  the accessibility tree, not perception.
- Scenarios, truths and the bot are written by the same team that fixes the UI, so the measuring
  stick is not independent of what it measures. Two AI reviewers of one model family count as
  "confidence 2" in the triage formula although their blind spots correlate.
- With 3 sessions per round, metric movements between rounds are noise; the triage formula is
  judgement with arithmetic around it.

## What the engine's design leaves open

- **Cells, not pixels.** Layout model v0 is ESS's terminal renderer. Overflow, clipping and density
  are coarse until a pixel model exists.
- **Exhaustive search may be impossible.** It depends on ESS exposing the actions available in a UI
  state and a state that can be forked or replayed cheaply, and on the measured cost per call.
- **Taste stays with people.** The metrics rank what is declared. The original loop's own data shows
  that hermetic metrics did not separate 28 design variants. The engine shows the rest side by side
  and never claims that a frontier variant looks better.
- **Ratchet is in design.** Trade-offs, result scope, exact versus sampled values and finding
  identity are open questions sent to ratchet; the preset family is a draft.

## Copies of the skill

Two older copies of the skill exist outside this repository. Until they point here, they drift.

## Privacy

Recording real users captures page text and input. The skill allows only a structure-only mode with
every text and input masked; a keyed probe that sends screenshots to a model is operator-only and
paid per round.
