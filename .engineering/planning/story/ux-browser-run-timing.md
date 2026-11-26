---
format: aep.planning-md/3
id: story:ux-browser-run-timing
kind: story
status: implemented
title: Time per check in the run record, and the page loaded once where the checks allow it
relations:
- decomposes: epic:engine-slices
scope:
- confidence: cited
  path: docs/browser-adapter.md
- confidence: cited
  path: engine/browser/src/evidence.rs
- confidence: cited
  path: engine/browser/src/main.rs
- confidence: cited
  path: engine/browser/src/measure.rs
- confidence: cited
  path: engine/browser/tests/timing.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T12:58:46Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-25T12:58:46Z", actor: "human:operator", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-26T18:50:22Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
# Story: Time per check in the run record, and the page loaded once where the checks allow it

## Outcome

A consumer reads from the result document where `ux-browser checks` spends its time per page, viewport and mode (waits, loads, probes, the keyboard walk, its Tab count), and a run with the keyboard walk loads each page-viewport fewer times, so llm's run takes about half its time with every check kept.

## Context

Consumer: llm (2026-11-25, relayed by the operator). Its docs UX step runs 116 s or more against the 120 s test-step budget; `ux-browser` 0.8.1 alone takes 37 to 48 s on a 4-CPU node and 1.4 times that on the forge: 54 page-viewports x 2 modes, 2.7 to 3.6 job-seconds per page-viewport and mode; `--jobs 4/6/8` gave 39-60 / 32-62 / 31-43 s. Its waits: settle 250 ms, scroll settle 200 ms per mode, mode settle 100 ms, focus settle 100 ms. Target from the consumer: 21 to 28 s on the forge.

Read from `engine/browser/src/measure.rs` at main 7969288: `measure_open` loads the page once per viewport for the checks of every mode; `keyboard_walk` then navigates again in each keyboard mode for the skip-link check (`fresh_load`) and again for the Tab walk to the targets, and once more per `activates` target. llm's configuration (keyboard modes `[dark]`, a skip link) loads each page-viewport 3 times. The run record holds no time per phase.

## Acceptance

The run section of the result document (outside the evidence, so the evidence stays the same for the same page) records per page and viewport the milliseconds of each phase (load and ready, settle, per mode: mode switch, overflow, always visible top, scroll settle, bottom, keyboard walk), the number of page loads and the Tab presses of each walk; a test reads those fields from a document. On a fixture with a skip link and a keyboard mode, the run loads each page-viewport fewer times than 0.10.0 (counted in a test), and the checks' results equal 0.10.0's on the fixture sites and on this repository's Docusaurus build; the time before and after is measured and written in `docs/browser-adapter.md`.
