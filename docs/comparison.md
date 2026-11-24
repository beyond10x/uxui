# Comparing the engine with the loop it replaces

Status: design, 2026-11-24. The engine exists as a specification only; this document says how it will be
measured against the `ui-iteration-loop` skill (the "loop") once the first slices of the roadmap exist, and
what can be measured before that.

## The question

Does the engine find and fix more real UX defects than the loop, for less time and money, with results that
hold up outside the engine's own metrics? Three ways the comparison can lie, and the guard against each:

| lie | guard |
|---|---|
| each tool is judged by its own metrics (the engine wins on cell metrics, the loop on its browser metrics) | a held-out referee neither tool optimises against |
| the targets favour one tool (an `ess-ui/1` page suits the engine, a hand-built page suits the loop) | one source per target: the `ess-ui/1` document, the browser app generated from it by `ess generate ui --target react` |
| one tool got more budget | a fixed budget per run: wall time, model tokens, browser minutes, person minutes, all recorded |

## The bench

A **bench** is a set of targets with known defects, the same for both tools.

- **Targets**: `ess-ui/1` documents with fixtures, at several scopes (a component, a page, a composition). The
  first is the studio itself (`ess/ui/studio.ui.yaml`); further ones come from ESS's examples and, later, the
  trial target of the roadmap. The loop runs on the React app generated from each document, so both tools see
  the same UI.
- **Seeded defects**: each target gets copies with one planted defect each, by a mutation operator run in
  reverse: a primary action moved below the fold, a required node hidden behind a tab, a label without an
  accessible name, a journey made 3 steps longer, a long fixture value that overflows, a second primary action,
  an overlay that traps focus, a contrast pair under 4.5:1. Each seed is a record: target, operator, node path,
  the journey it breaks. Ground truth comes from the seeding, not from either tool.
- **Clean controls**: the unseeded targets, to count false findings.

## What is measured per run

| measure | what it says |
|---|---|
| seeds found / seeds planted (recall), per defect class | which defects each tool can see at all |
| findings on clean controls (false findings) | noise a person has to triage |
| time and cost to the first true finding, and to the end of the budget | speed |
| states reached per journey | coverage: the loop's bot walks scripted scenarios, the engine enumerates |
| same run twice, same findings (determinism) | whether a result can be gated on |
| referee score of the best variant each tool proposes after the budget | whether the fixes make the UI better |

## The referee

The referee scores the final UI of each tool and is used by neither during the run:

1. A browser measurement of the generated app (scroll, visibility, axe-core, task completion by a screen-only
   bot) on **held-out journeys and environments** that neither tool's run used.
2. A blind pairwise preference by people (and, labelled as such, by a review agent) between the two proposals
   and the original, recorded as judgement records. Never a gate input.

The engine's cell-based layout model is checked against (1) on its own: for the same variants, does the cell
metric rank them as the browser does (rank correlation per metric)? A low correlation is a finding against the
layout model, reported before any head-to-head result, because every engine number rests on it.

## What can be measured now

- The loop runs today. A first bench with the studio app and 8 seeded copies gives the loop's recall, false
  findings, cost and determinism: the baseline any engine slice has to beat.
- `ux-static@1` (roadmap step 4) runs on the same bench as soon as it exists: static defects (missing names,
  journey length, two primary actions) are measurable without a layout model.
- The head-to-head on layout defects waits for the layout model (roadmap steps 3 and 5).

## Results are ratchet evidence

Each run writes ratchet evidence records with the subject "tool" (loop or engine, with version), the bench
version and the measures above. The bench is a ratchet assessment plan; a new engine version may not lower
recall on any defect class or raise false findings. The comparison then keeps running after it is won.

## Dogfooding

The first target is the engine's own studio: its `ess-ui/1` document is generated into a React app on fixtures
(`ess generate ui --target react --model ess --fixtures`). The loop runs a round on it now; the engine runs on
it as soon as a slice exists. Findings on the studio go into `ess/ui/` like any other target's.
