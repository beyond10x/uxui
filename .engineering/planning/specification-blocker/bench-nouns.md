---
format: aep.planning-md/3
id: specification-blocker:bench-nouns
kind: specification-blocker
status: open
title: the bench's records have no ESS home
relations:
- blocks: epic:bench
revision: 2
---
# Blocker: the bench's records have no ESS home

## What Is Blocked

Drafting five stories of `epic:bench`. Each introduces a typed record that no ESS document in this
repository declares, and the planning rule is that such a noun is modelled before stories are written
around it. The classification is kept here:

| story | acceptance | nouns | source |
|---|---|---|---|
| Studio bench and the loop's baseline | A bench of the studio app with 8 seeded copies and the clean original gives the loop's recall per defect class, false findings, cost and determinism. | Bench, SeededDefect, BenchRun | `docs/comparison.md:18-31, 59-60` |
| `ux-static@1` on the bench | `ux-static@1` runs on the same bench and reports the same measures for static defects. | BenchRun | `docs/comparison.md:61-62` |
| Layout model against the browser | For the same variants, the rank correlation per metric between the cell model and the browser is reported before any head-to-head result. | RankCorrelation | `docs/comparison.md:53-55` |
| Head-to-head on layout defects | Both tools run on the bench under a fixed budget, and the referee scores each tool's best variant: a browser measurement on held-out journeys and environments, and a blind pairwise preference by people (and, labelled as such, by a review agent) between the two proposals and the original, recorded as judgement records and never a gate input. | BenchRun, RefereeScore, JudgementRecord | `docs/comparison.md:33-51, 63` |
| Results as ratchet evidence | Each run writes ratchet evidence with the subject "tool", the bench version and the measures, and a new engine version may not lower recall on any defect class. | BenchRun | `docs/comparison.md:65-69` |

## What Would Clear It

An ESS document in this repository that declares the bench, a seeded defect (target, operator, node
path, the journey it breaks), a run with its measures, and the referee's score, passing
`ess specify validate`; the stories then cite it. `JudgementRecord` exists already
(`uxsearch.promotion`).

## Who Can Clear It

Whoever owns `ess/` once the hardening pass (`story:spec-hardening-pass`) has landed.

## What We Are Doing Meanwhile

`story:loop-round-on-studio` needs none of these nouns.
