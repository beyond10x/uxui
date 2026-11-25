---
format: aep.planning-md/3
id: story:ux-browser-cover-points
kind: story
status: draft
title: Cover test over more points than the centre
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
revision: 2
---
# Story: Cover test over more points than the centre

## Outcome

An always-visible item half under a sticky bar or a dialog is reported as covered.

## Context

Consumer: the second consumer of ux-browser, a control plane web application whose gate runs its own page checks (Playwright). It ran ux-browser 0.3.0 on 11 pages x 2 viewports on 2026-11-25: page scroll, overflow and always visible gave its own values (0/0); none could replace an own check yet. It reported 8 gaps; line numbers are `docs/browser-adapter.md` at 0.3.0 (2a921f1).

Gap 5 (line 152). Checked against the source at 2a921f1: holds: `elementFromPoint` of the centre only.

Order 7 of 8: ordered by what lets a consumer drop an own check (a running page measured reliably first).

## Acceptance

The cover test samples the corners (inset 1 px) and the centre; the item is covered when any sample hits another element; the point set is documented; a fixture with an item covered at one corner counts 1 miss, 0.3.0 counted 0.
