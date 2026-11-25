---
title: The studio
sidebar_position: 4
description: The search engine's live UI, specified as an ess-ui/1 document and running today on fixtures with no engine behind it.
lede: The studio is where a person watches a search climb, compares variants and promotes one; it is itself an ess-ui/1 document, generated into an application that runs on fixtures.
source: ess/ui/studio.ui.yaml; ess/ui/fixtures/; ess/domains/promotion.yaml; themes/studio.css
---

# The studio

The studio is the live UI of the [search engine](./search-engine.md). It is not hand-built: it is
specified as an `ess-ui/1` document over the engine's ESS specification,
[`ess/ui/studio.ui.yaml`](https://github.com/beyond10x/uxui/blob/main/ess/ui/studio.ui.yaml),
with its fixtures and tests beside it.

:::caution[Specification only]

The studio application runs, but no engine runs behind it. Every value it shows comes from the
document's fixtures: invented sample data and scripted live events. The engine's first search
(`ux-search climb`, one toy page, a frontier printed as text) does not feed it; the engine that would
produce real campaigns is a specification; see [Status](../status.md).

:::

![The live climb of a campaign in the studio, light mode](/img/studio-climb.png)

## What a person does there

| page | what a person does |
|---|---|
| Targets | picks the part of the document to optimise (a component, a widget, a section, a page or the whole product) and starts a campaign on it |
| Campaigns | lists campaigns and opens one; a running campaign can be stopped from its climb |
| Live climb | watches the search climb live: evaluated, pruned and insufficient variants and the frontier size per step, the best variant so far and the frontier as it stands |
| Frontier | reads the non-dominated variants with their KPI values, picks one and compares it with another, or proposes it for promotion |
| Variant | reads its rule verdicts and evaluations, and switches one axis by hand to reach a neighbouring variant |
| Compare | two variants side by side, and a judgement recorded on them |
| Judgements | a designer's preferences and findings, and what each became: a design rule, a KPI weight or a finding |
| Promotions | the gate decisions ratchet returned; the **Owner**, a person, accepts, rejects, or accepts a trade-off as a new protected reference |

Accepting a trade-off is the Owner's alone. No system actor holds that command, and a hard rule can
never be traded away.

![The frontier of a campaign: variants with KPI values, compare and promote actions](/img/studio-frontier.png)

## Running it today

The application is generated from the document by ESS, with fixtures built in:

```console
ess generate ui --target react --model ess --fixtures
```

Every read is answered from the fixtures and every live channel plays its fixture script, so the
studio runs without a backend. The `climb` channel loops through campaign progress and frontier
changes on a virtual clock, including a reconnect, so the climb page moves as a real search would
make it move. The `switches` channel plays the result of an axis switch.

![The list of targets a campaign can start on](/img/studio-catalog.png)

## Theme

The studio is themed by uxui with `themes/studio.css`: a light and a dark mode. A toggle in the
sidebar follows the system's colour scheme until a person picks a mode, and keeps that pick.

![The live climb in dark mode](/img/studio-climb-dark.png)

## What it is for

The studio is the place where taste stays human. The engine ranks what is declared; a person sees
variants side by side in the same state and environment, records what no metric explains, and
decides which trade-off to accept. Generating the studio from a specification also tests the
specification: a page the document cannot express is a gap, recorded in
[`ess/GAPS.md`](https://github.com/beyond10x/uxui/blob/main/ess/GAPS.md) or as an `UNMAPPED`
marker in the document.
