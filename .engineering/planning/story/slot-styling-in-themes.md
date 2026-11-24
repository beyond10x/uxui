---
format: aep.planning-md/3
id: story:slot-styling-in-themes
kind: story
status: draft
title: slot's styling and layout needs as uxui themes and templates
relations:
- decomposes: epic:generators-and-themes
scope:
- confidence: cited
  path: themes
revision: 2
---
# Story: slot's styling and layout needs as uxui themes and templates

## Outcome

The styling and layout gaps slot's web UI found are solved in uxui's themes and templates, not by
extending `ess-ui/1`.

## Context

The slot case study classifies three of slot's findings as styling and layout, for uxui themes and
templates, not ESS: G9 (column widths, one-line cells, class hooks), its F9 (label beside value),
and the colour per value of G11 (`docs/generator-round.md:87-89`). uxui owns themes and templates
(`AGENTS.md:35-38`). slot has since moved to a generator of its own (`docs/generator-round.md:91-92`),
so where the theme or template plugs in is part of the work. Shared surface: `themes/`, which
`story:themes-light-dark` also changes.

## Acceptance

A uxui theme or template under `themes/` declares column widths, one-line cells, class hooks, a label
beside its value and a colour per value, and slot's job list and job detail use it with no change
to `ess-ui/1`.

## Provenance

Migrated from `docs/generator-round.md`, "slot case study", lines 87-89, added on 2026-11-24 while
this migration ran (untracked; the document dates itself `Status: plan, 2026-11-24`).
