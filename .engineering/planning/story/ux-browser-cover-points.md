---
format: aep.planning-md/3
id: story:ux-browser-cover-points
kind: story
status: implemented
title: Cover test over more points than the centre
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T10:53:28Z", actor: "human:operator", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-25T10:53:28Z", actor: "human:operator", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-25T11:02:55Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
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

Revised at implementation, 2026-11-25: a corner point 1 px inside a rounded corner lies outside the element, as hit testing follows `border-radius` (measured: a 160 x 40 pill with `border-radius: 999px` and nothing over it reported 2 misses, all four corners). Each corner point moves in by 1 px plus r (1 - 1/sqrt(2)), r that corner's used radius, at most to the centre; without a radius that is 1 px. "1 miss" is per item and position: the fixture pages do not scroll, so one covered item counts at the top and at the bottom, 2 per page (0.9.1: 0).
