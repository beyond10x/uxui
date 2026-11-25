---
format: aep.planning-md/3
id: story:ux-browser-keyboard-declared-controls
kind: story
status: implemented
title: Keyboard walk without a skip link, to declared controls, with a focus mark
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T02:15:04Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-25T02:15:04Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-25T02:15:04Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
# Story: Keyboard walk without a skip link, to declared controls, with a focus mark

## Outcome

A consumer without a skip link gets 0 keyboard misses where its own rule has 0, and the walk checks that focus is visible.

## Context

Consumer: the second consumer of ux-browser, a control plane web application whose gate runs its own page checks (Playwright). It ran ux-browser 0.3.0 on 11 pages x 2 viewports on 2026-11-25: page scroll, overflow and always visible gave its own values (0/0); none could replace an own check yet. It reported 8 gaps; line numbers are `docs/browser-adapter.md` at 0.3.0 (2a921f1).

Gap 7 (lines 113, 153). Checked against the source at 2a921f1: holds: the skip link is required (`src/checks.rs`: the skip link is named by `skip_link_is` or `skip_link_contains`), and a page without one misses 2 checks; focus visibility is not checked. Confirmed by the consumer on 2026-11-25: no false positive; its page has no skip link (its own review found the same), so each walk misses both skip-link checks (value 4 per page over 2 walks). Its own rule: per scenario step, Tab reaches the step's control (role and name) within 250 presses, the focused control shows a visible focus mark, Enter or Space acts.

Order 5 of 8: ordered by what lets a consumer drop an own check (a running page measured reliably first).

## Acceptance

`keyboard.skip_link` is optional: without it the walk checks only that each `must_reach` control is reached within `max_tabs`; a declared focus-mark rule (outline or box-shadow present on the focused element) counts a miss per control without one; a page without a skip link and without the key gives 0 skip-link misses; Enter or Space on a reached control is optional per target; on a control plane fixture page the misses equal the consumer's rule (0).
