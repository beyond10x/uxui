---
format: aep.planning-md/3
id: story:ux-browser-always-visible-by-css
kind: story
status: draft
title: Always-visible items by CSS alone or by aria-current
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
revision: 2
---
# Story: Always-visible items by CSS alone or by aria-current

## Outcome

A consumer declares a per-page item (the current page title, the current navigation entry) once instead of once per page.

## Context

Consumer: the second consumer of ux-browser, a control plane web application whose gate runs its own page checks (Playwright). It ran ux-browser 0.3.0 on 11 pages x 2 viewports on 2026-11-25: page scroll, overflow and always visible gave its own values (0/0); none could replace an own check yet. It reported 8 gaps; line numbers are `docs/browser-adapter.md` at 0.3.0 (2a921f1).

Gap 4 (lines 112, 152). Checked against the source at 2a921f1: holds: `role` is required (`src/checks.rs` `always_visible ... no role`); `aria-current` is not read.

Order 4 of 8: ordered by what lets a consumer drop an own check (a running page measured reliably first).

## Acceptance

An always-visible item may give `css` without `role`, or `current: true` to take the element with `aria-current` in its scope; one such declaration replaces the 11 per-page title items the consumer needed; the refusal of an item with neither role nor css stays.
