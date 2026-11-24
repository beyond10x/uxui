---
title: Getting started
sidebar_position: 2
description: Install the uxui plugin in Claude Code or Codex and ask for a first round.
---

# Getting started

The plugin installs from a clone of the repository. It holds one skill, `ui-iteration-loop`, and no
scripts: the skill names the tools a project builds its loop on (Playwright, axe-core, rrweb,
ffmpeg).

## Install in Claude Code

```console
git clone https://github.com/beyond10x/uxui
claude plugin marketplace add ./uxui
claude plugin install uxui@uxui
```

## Install in Codex

```console
git clone https://github.com/beyond10x/uxui
codex plugin marketplace add ./uxui
codex plugin add uxui@uxui
```

## Start

Start a new session after installing. Then either ask in your own words, for example for a UI
improvement round, a UX round or a visual regression ratchet, or call the skill by name:

| host | call |
|---|---|
| Claude Code | `/uxui:ui-iteration-loop` |
| Codex | `$ui-iteration-loop` |

The skill first asks your project for its adapter: scenarios, gating viewports, themes, the
always-visible elements, the completion states and where the baseline lives. The
[guide](./guides/run-a-round.md) walks through one round; the
[skill reference](./reference/skill.md) is the full text.

## Check a clone

`task check` is the repository's offline gate. It validates both plugin manifests and both
marketplaces, checks that they agree, and proves the check fails on broken copies of the tree. It
takes a few seconds.

```console
cd uxui
task check
```
