---
format: aep.planning-md/3
id: story:ux-browser-fixed-clock
kind: story
status: implemented
title: A fixed page clock
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T02:15:49Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-25T02:15:49Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-25T02:15:49Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
# Story: A fixed page clock

## Outcome

Pages that show times or animate on timers measure the same on every run.

## Context

Consumer: the second consumer of ux-browser, a control plane web application whose gate runs its own page checks (Playwright). It ran ux-browser 0.3.0 on 11 pages x 2 viewports on 2026-11-25: page scroll, overflow and always visible gave its own values (0/0); none could replace an own check yet. It reported 8 gaps; line numbers are `docs/browser-adapter.md` at 0.3.0 (2a921f1).

Gap 3 (line 27). Checked against the source at 2a921f1: holds: no clock control; only the probes are injected before the page's scripts.

Order 6 of 8: ordered by what lets a consumer drop an own check (a running page measured reliably first).

## Acceptance

A config key sets the page clock (`Date`, `performance.now` start, timers) to a declared instant before the page's own scripts; two runs of a fixture page that prints the time give equal geometry.
