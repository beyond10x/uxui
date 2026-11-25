---
format: aep.planning-md/3
id: story:ux-browser-page-ready-signal
kind: story
status: implemented
title: ux-browser waits for a ready condition the page sets
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T01:10:41Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-25T01:10:42Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-25T01:10:42Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
# Story: ux-browser waits for a ready condition the page sets

## Outcome

Layout shift after ready is the same run to run on one snapshot, because ready is the page's own signal.

## Context

Consumer: the second consumer of ux-browser, a control plane web application whose gate runs its own page checks (Playwright). It ran ux-browser 0.3.0 on 11 pages x 2 viewports on 2026-11-25: page scroll, overflow and always visible gave its own values (0/0); none could replace an own check yet. It reported 8 gaps; line numbers are `docs/browser-adapter.md` at 0.3.0 (2a921f1).

Gap 2 (line 140). Checked against the source at 2a921f1: holds: ready is fonts loaded plus two frames, a fixed definition.

Order 2 of 8: ordered by what lets a consumer drop an own check (a running page measured reliably first).

## Acceptance

A config key names a ready condition the page sets (an attribute on an element or a global the page sets true); ready is that condition, then fonts and two frames; a page that does not set it within a declared time is incomplete; 5 runs on one snapshot give equal `layout_shift.after_ready` (the consumer saw 0.1781 and 0.8819 on two pages).
