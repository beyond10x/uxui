---
title: Design and state spaces
sidebar_position: 3
description: The two spaces the engine searches and evaluates, their sizes in a worked example, the layout-model contract and the search strategies.
lede: A variant comes from the design space, a UI state from the state space; evaluation crosses them with the environments, so each space is reduced on its own first.
source: docs/state-space.md; docs/concept.md
---

# Design and state spaces

:::caution[Specification only]

These are design numbers from `docs/state-space.md` and `docs/concept.md`. The time estimates rest
on an assumed 50 µs per layout call that nothing has measured yet.

:::

## Two spaces

| space | element | made of | who chooses |
|---|---|---|---|
| design space | a variant | axes × mutation operators applied to the base document | the search |
| UI state space | a UI state | fixtures × actions × overlays × focus × virtual time, per journey | the user, simulated |

Evaluation crosses both with the **environments** (viewport, theme mode, input mode, text scale,
reduced motion). Sizes multiply, so each space is reduced on its own first.

## Axes

An **axis** is one dimension of variation with a finite set of values. Its kinds:

| kind | varies |
|---|---|
| `theme` | a token set |
| `template` | a shell, page or section template |
| `layout` | region placement, section order, density |
| `component` | an alternative composite for the same data, for example table or cards |
| `interaction` | for example an inline confirm or an overlay; save on change or on submit |
| `copy` | label variants |

A **variant** is the base document plus a choice per axis and a sequence of operator applications.
It is identified by the digest of its canonical form, so two routes to the same document are one
variant.

## A worked example

A ticket desk page: a filter bar, a list of tickets, the selected ticket's detail, a drawer to edit
it and a confirm before closing it. Nine axes:

| quantity | value |
|---|---|
| product of the axis value counts | 11,664 |
| valid variants, after operator preconditions | 5,184 |
| after the static hard rule "a command that ends a lifecycle has a confirm" | 3,456 |
| environments: 3 viewports × 2 theme modes × 2 input modes × 2 text scales | 24 |
| UI states per variant and environment, over three journeys | 1,501 |
| (variant, environment, state) evaluations | 124,498,944 |
| layout calls, after merging inputs that do not move a box | 20,749,824 |

The same collection searched alone as a component has 144 variants, 97 states and 16 environments:
223,488 evaluations, small enough to search exhaustively.

## Search in stages

| stage | evaluates | ticket page |
|---|---|---|
| 1 static | all valid variants, on the document alone | 5,184 checks, 1,728 pruned |
| 2 journey paths | the states on each journey's shortest path | 345,600 layout calls |
| 3 full state space | the frontier only | about 450,000 layout calls for 50 variants |

Stage 3 can change the frontier: a variant can fail a hard rule in a state no journey path visits.

| strategy | use when |
|---|---|
| exhaustive | the layout calls fit the call budget |
| beam | structural moves of unbounded depth dominate |
| hill climbing with restarts | one scalar KPI over a large axis product |
| Pareto evolution | several KPIs, a large space, a frontier wanted |

Every strategy takes a seed: the same seed and inputs give the same frontier and the same climb, so
the studio can replay it.

## The layout model

A deterministic function from (canonical variant, UI state, environment, font-metrics table, model
version) to boxes, visible fractions, laid-out text, resolved style, an accessibility node per
element, the document extents and the focus order. The same inputs give byte-identical output: no
clock, no randomness, no host fonts. A construct the model cannot lay out is refused by node path,
and every metric that reads it becomes insufficient evidence.

Version 0 is ESS's terminal renderer: a deterministic cell grid, enough for in-view, overflow,
truncation and the action graph. It has no pixels, no tokens, no accessibility tree and no Tab focus
model; a pixel model comes later.

## What ESS still lacks

To be raised with the ESS maintainers, in the order the engine needs them: the actions available in
a UI state and a state that can be forked; a canonical digest of a UI state and of an `ess-ui/1`
document; a per-step trace in the test report; rendering one component alone; and declarations of
required and always-visible nodes, the completion signal and a guidance cue. The first two decide
whether exhaustive search is possible at all.
