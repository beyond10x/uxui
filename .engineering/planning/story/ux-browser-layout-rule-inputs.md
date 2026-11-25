---
format: aep.planning-md/3
id: story:ux-browser-layout-rule-inputs
kind: story
status: draft
title: Page-scroll slack and the overflow rule as configuration
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
revision: 2
---
# Story: Page-scroll slack and the overflow rule as configuration

## Outcome

A consumer whose page-scroll and overflow rules differ from the built-in ones gets its own values from ux-browser.

## Context

Consumer: the second consumer of ux-browser, a control plane web application whose gate runs its own page checks (Playwright). It ran ux-browser 0.3.0 on 11 pages x 2 viewports on 2026-11-25: page scroll, overflow and always visible gave its own values (0/0); none could replace an own check yet. It reported 8 gaps; line numbers are `docs/browser-adapter.md` at 0.3.0 (2a921f1).

Gap 6 (lines 150, 151). Checked against the source at 2a921f1: holds: page scroll is `> 0`, overflow `+ 1 px` with a fixed exclusion list, constants in `src/checks.rs`.

Order 3 of 8: ordered by what lets a consumer drop an own check (a running page measured reliably first).

## Acceptance

`layout.page_slack_px` (default 0) and the overflow rule's tolerance and exclusions are config keys; defaults give 0.3.0's values on the existing fixtures; a fixture with a 1 px page scroll is 0 with slack 1 and 1 without.
