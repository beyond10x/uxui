---
title: Run one round
sidebar_position: 1
description: Set up the adapter and the baseline, then run one round of the ui-iteration-loop skill on your own UI.
lede: A round needs an adapter, a baseline and scenario cards; the agent then measures, triages by rule, builds failing-first and gates the merge.
source: plugin/skills/ui-iteration-loop/SKILL.md; plugin/skills/ui-iteration-loop/references/not-available.md
---

# Run one round

This guide assumes the plugin is installed ([Getting started](../getting-started.md)). The agent
does the work; you supply the project's facts, answer one design question, and take part in the
trial if people are part of the round.

## 1. Build the tools

The plugin ships no scripts. Ask the agent to build what the round needs on the open tools, in your
project's own language: a recorder (rrweb), the round report, the layout contract (Playwright and
axe-core), the screen-only bot, the ratchet gate. The list, with what each tool does, is in the
skill's `references/not-available.md`. Keep the core free of project code: it imports nothing from
the UI.

## 2. Write the adapter

The project supplies one adapter file and these values:

| slot | for example, for a docs site |
|---|---|
| scenarios | the page list plus tasks such as "find the install command for X", under `ux/scenarios/<name>/` |
| gating viewports | the sizes every merge checks, for example 1440x900 and 1280x800 |
| measured-only viewports | sizes reported but not gated: 1024x768, 768x1024, 390x844 |
| themes | every theme the UI ships |
| always-visible elements | navbar, search, theme toggle, the sidebar at desktop sizes |
| completion states | what "done" looks like on screen |
| guidance cue | the element that says what to do next, if any; none drops level L0 and gate 6 |
| baseline file | `ux/baseline.json` |
| round and findings directories | `ux/rounds/R-nn/`, `ux/findings/UXF-nnn.md` |

## 3. Set the baseline

Run the fast loop once on `main` and write its values into `ux/baseline.json`. From now on every
merge runs levels L0 to L3 and the ratchet. Each step of the gate should stay within your test
budget; a browser run that takes minutes is the loop's known cost.

## 4. Run the round

Ask the agent for a round, `/uxui:ui-iteration-loop` in Claude Code or `$ui-iteration-loop` in
Codex. In order:

1. **Trial.** Three scripted sessions from scenario cards, two anchors and one rotating, each
   recorded, each followed by a 90-second questionnaire. A driver bot may stand in for a person.
2. **Measure.** The round command writes `report.md`, `round.json`, screenshots and a trend row
   into `ux/rounds/R-nn/`. Two fresh review agents write their reviews before anyone reads the round.
3. **Triage.** The agent ranks findings by the formula and writes `triage.md` with a change set of
   at most 5 changes; every S1 finding is in it.
4. **Design decision.** If a layout or interaction change needs one, you get one yes-or-no question
   in `decision.md`, with screenshots of both answers and a default if you do not answer before the
   next round.
5. **Build.** For each change the agent writes the finding's regression test, runs it red, then
   makes the change that turns it green.
6. **Gate.** The merge passes the hard gates and the ratchet. An improved value is written into the
   baseline in the same merge.

## 5. Read the next round

The next round reports, for each change, whether its target metric moved. A change that did not
move it after two rounds is kept with a written reason or reverted.

The rules behind each step, verbatim, are in the [skill reference](../reference/skill.md). What one
real round showed, including where the ratchet got in the way, is on
[Limitations](../limitations.md).
