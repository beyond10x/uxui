---
format: aep.planning-md/3
id: story:ux-browser-journey-states
kind: story
status: draft
title: Checks at each state of a scripted journey
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
revision: 3
---
# Story: Checks at each state of a scripted journey

## Outcome

A consumer measures the five checks in a page state reached by steps (open a menu, fill a form, open a dialog), not only after load.

## Context

Consumer: the second consumer of ux-browser, a control plane web application whose gate runs its own page checks (Playwright). It ran ux-browser 0.3.0 on 11 pages x 2 viewports on 2026-11-25: page scroll, overflow and always visible gave its own values (0/0); none could replace an own check yet. It reported 8 gaps; line numbers are `docs/browser-adapter.md` at 0.3.0 (2a921f1).

Gap 8 (lines 140 to 146). Checked against the source at 2a921f1: holds: checks run after load and per mode only; journeys exist in the specification (`ess/domains/journeys.yaml`), not in ux-browser.

Order 8 of 8: ordered by what lets a consumer drop an own check (a running page measured reliably first).

## Acceptance

A config declares journeys: steps (click, type, key, wait for a selector) named by CSS selector, and the states after which the checks run; results carry the journey and state in their identity; spec scenario `browser-journey-names-nodes-by-selector` (ess) is the contract.

Added 2026-11-25 from the second consumer on 0.8.1: a journey state can open a closed `details` (a click on its `summary`), and the overflow check in that state measures the content the closed state skips; a fixture page with a long line in a closed `details` counts 0 overflow after load and 1 in the state after the click.
