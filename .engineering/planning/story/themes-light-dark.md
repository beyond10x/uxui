---
format: aep.planning-md/3
id: story:themes-light-dark
kind: story
status: implemented
title: Themes with a light and a dark mode
tags:
- in-flight
relations:
- decomposes: epic:generators-and-themes
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: themes
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T10:50:07Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-24T10:50:07Z", actor: "human:operator", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-11-24T10:50:07Z", actor: "human:operator", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
# Story: Themes with a light and a dark mode

## Outcome

Every uxui theme over generated output has a light and a dark mode that a person can pick and that
the generated app does not overwrite.

## Context

Under way on 2026-11-24 in `themes/` (untracked in this worktree: `studio.css`,
`theme-toggle.js`), with the `themes/` row of `AGENTS.md` (an uncommitted edit here). uxui owns
themes and templates (`AGENTS.md:35-38`). Each theme has a light and a dark mode chosen by
`theme-toggle.js`, the system setting until a person picks one (`AGENTS.md:26`). The generated app
owns `data-theme` (F5, `docs/generator-round.md:64-67`), so the uxui theme keys its modes on
`data-uxui-mode`. F4 (series colours) is solved in `themes/studio.css`
(`docs/generator-round.md:59-62`).

Shared surface: `AGENTS.md`, which `story:docs-site` and `story:own-generator-smallest` also edit in
their worktrees; `themes/`, which `story:slot-styling-in-themes` also changes.

## Acceptance

For every theme under `themes/`, loaded over the generated studio app whose runtime writes
`data-theme` on load, the mode a person picked in `theme-toggle.js`, light or dark, is the mode
shown.

## Provenance

Migrated from the `themes/` row of `AGENTS.md` (line 26, an uncommitted edit of 2026-11-24) and
`docs/generator-round.md` (F4-F5, lines 59-67; untracked on 2026-11-24, `Status: plan, 2026-11-24`).
