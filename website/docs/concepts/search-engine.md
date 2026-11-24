---
title: The search engine
sidebar_position: 2
description: How the specified UX search engine generates variants, evaluates them without I/O, searches, and promotes a variant through ratchet.
lede: Generate the design variants of an ess-ui/1 document by rule, evaluate each against every reachable UI state with no I/O, and search for the best compositions under declared rules.
source: docs/brief.md; docs/concept.md; docs/ratchet.md; ess/system.yaml
---

# The search engine

:::caution[Specification only]

Nothing on this page is built. It describes the design in `docs/concept.md` and the ESS
specification under `ess/`. The [Roadmap](../roadmap.md) says what is built first.

:::

One sentence: generate the design variants of an `ess-ui/1` document by rule, evaluate each against
every reachable UI state with no I/O, and search for the best compositions under declared rules.

## Why

The loop of the [ui-iteration-loop skill](./the-loop.md) measures one hand-built UI in a browser,
scenario by scenario.

| the current loop | the engine |
|---|---|
| one hand-built UI, a few variants behind URL flags | the design space generated from the specification; thousands of variants |
| replays of a handful of recorded sessions | every reachable state of the UI model within bounds, from ESS fixtures |
| a browser run per check, minutes each | a pure evaluation per variant, state and environment, cached by digest |
| a fixed set of metrics in one file | a declared [metric catalogue](../reference/metrics.md), each metric typed |
| one ratchet per value; a trade-off needs a decision id | a Pareto front over declared KPIs; ratchet guards the promoted choice |
| metrics that cannot rank designs | still true for aesthetics: the engine ranks what is declared and leaves taste to people |

## The pipeline

```text
base document ──▶ design space ──▶ hard rules ──▶ evaluation ──▶ metrics ──▶ search ──▶ frontier
 (ess-ui/1)       axes × operators  (static,      (layout model,  (pure       (strategy,    │
                                     prune)        fixtures,       functions)  budget)       ▼
                                                   virtual clock)                         studio ──▶ promotion ──▶ ratchet
```

1. **Design space from the specification.** Axes are read from what `ess-ui/1` already makes a
   choice: themes, page layouts, collection styles, overlay kinds, form save modes, action
   presentation. Mutation operators add structural moves. Every variant passes `ess ui check`.
2. **No-I/O evaluation.** Reads are answered from ESS fixtures, live events come from scripts on a
   virtual clock, command refusals are declared per test. A deterministic layout model turns
   (variant, UI state, environment) into geometry and an accessibility tree. No browser, no
   network, no wall clock.
3. **Metrics as pure functions.** A fixed catalogue; hard rules prune, KPIs score.
4. **Search.** Exhaustive when the space is small, otherwise beam, hill climbing with restarts or
   Pareto evolution. Every evaluation is cached by the digest of its inputs. The result is a
   **frontier**: the variants no other variant beats on every KPI.
5. **Studio.** The live view: the climb, the frontier, variants side by side in the same state and
   environment, manual axis switches. It is itself an `ess-ui/1` document in the repository and
   already runs on fixtures: [The studio](./studio.md).
6. **Promotion.** A person picks a frontier variant. The engine writes the `ess-ui/1` diff, the
   ratchet evidence and a proposed protected reference; merging the diff is a normal reviewed change.

The browser does not disappear: a browser assessor runs the promoted variant's journeys in a real
engine, and its geometry measures how faithful the layout model is.

## Targets

A campaign has one target, the part of the document it varies and evaluates.

| scope | in `ess-ui/1` | rendered in |
|---|---|---|
| component | one built-in composite (`collection`, `form`, `choice`, …) | a harness container at declared widths |
| widget | an app-defined composite with typed params | a harness container |
| section | one section of a page, with its own read and lifecycle | a harness page with only that section |
| page | one route: layout, header, sections, overlays | the shell at declared viewports |
| composition | several targets optimised together along one journey | the shell |
| product | the whole document | the shell |

Search runs bottom-up. A component's space is small enough to search exhaustively. A page treats
each component either as **fixed** (pinned at its promoted variant) or as **axes** (the component's
top frontier variants become one axis). A promoted widget changes every page that uses it, and
ratchet's merge gate checks each of them.

## Ratchet holds the results

The engine keeps no baseline of its own. Metrics are ratchet measurements, hard rules and KPIs are
ratchet requirements, the promoted variant's values become protected references, and evidence
records carry the resolved target: variant digest, layout-model version, environment. An
evaluation that cannot finish (a bound hit, a node the layout model refuses, a missing fixture) is
**insufficient evidence**, never a pass.

| gate | evaluates | requires |
|---|---|---|
| merge | the states on each journey's shortest path | every hard rule in every condition, no KPI worse than its reference, no new finding, nothing insufficient |
| promote | the full state space of every journey | the same, plus a reference change that names the trade-off it accepts |

A frontier variant is better on some KPIs and worse on others. Its promotion is a reference change
that lists every requirement that got worse, with both values, and at least one that got better;
hard rules never get worse, and the decision is a person's or an authorized policy's, never the
build agent's.

## What stays human

Aesthetics, taste, comprehension and trust are not machine-checkable here. Design proxies (emphasis
per region, colour share, type scale) are computed and labelled as proxies. **Judgement records**, a
person's or a review agent's preference between variants, never gate anything; they become a design
rule, a KPI weight or a finding that asks for a new metric.

The sizes this design implies, and how search stays within them, are on
[Design and state spaces](./spaces.md). The specification, domain by domain, is on
[ESS specification](../reference/specification.md).
