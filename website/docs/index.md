---
slug: /
title: What is uxui?
sidebar_position: 1
description: uxui is an agent plugin that improves a web UI in measured rounds, and the specification of a UX search engine.
---

# Measure the UI, never merge it worse

uxui has two parts in two different states, and the engine's studio already runs on sample data.

| part | what it does | state |
|---|---|---|
| The `ui-iteration-loop` plugin for Claude Code and Codex | Drives a measured improvement loop on a web UI: rounds of trial, measure, triage, design decision, build and gate, with hard gates on every merge and lower-is-better ratchets against a committed `baseline.json` | Shipped, plugin version `0.12.0` |
| The UX search engine | Would generate the design variants of an `ess-ui/1` document, evaluate each against every reachable UI state with no I/O, search the design space, show the climb in a studio and gate the promoted variant through ratchet | A first search: `ux-search climb` ranks theme x section order of one ESS example page on layout model v0 and prints the frontier as text. No UI state exploration, no browser stage, no climb in the studio; the rest is design documents and an ESS specification |
| The studio | The engine's live UI, itself an `ess-ui/1` document, generated into an application | Runs today on fixtures and scripted live events, with no engine behind it |

## Choose your path

| You want to… | Start here |
|---|---|
| Install the plugin and start a round | [Getting started](./getting-started.md) |
| Understand rounds, gates and ratchets | [The loop](./concepts/the-loop.md) |
| Run one round on your own UI | [Run one round](./guides/run-a-round.md) |
| Read the skill as the agent reads it | [The ui-iteration-loop skill](./reference/skill.md) |
| Understand the engine that is being specified | [The search engine](./concepts/search-engine.md) · [Design and state spaces](./concepts/spaces.md) |
| See where a person watches the climb and promotes a variant | [The studio](./concepts/studio.md) |
| Look up a metric or a word | [Metric catalogue](./reference/metrics.md) · [Glossary](./reference/glossary.md) |
| Know what exists and what comes next | [Status](./status.md) · [Roadmap](./roadmap.md) · [Limitations](./limitations.md) |

## How the two parts relate

The plugin measures one hand-built UI in a browser, scenario by scenario. That is slow, bound to the
scenarios someone recorded, and it compares a few hand-built variants. The engine is the answer
being specified: variants generated from the specification by rule, every reachable state evaluated
without a browser, and a search over thousands of variants. When the engine exists, the skill
becomes a thin round on top of it; until then, the plugin is the part you can use.

## Where the content comes from

The design documents live in the repository under `docs/`, the specification under `ess/`, the
skill under `plugin/skills/ui-iteration-loop/`. Pages generated from those files say so at the top
and are rebuilt by `sh tools/docs.sh generate`; the other pages name their sources in their front
matter.
