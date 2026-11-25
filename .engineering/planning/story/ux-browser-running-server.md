---
format: aep.planning-md/3
id: story:ux-browser-running-server
kind: story
status: implemented
title: ux-browser checks a running server
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T01:04:52Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-25T01:04:52Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-25T01:04:52Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
# Story: ux-browser checks a running server

## Outcome

A consumer measures its page where it runs (dev server, preview, deployed origin) without building a static directory.

## Context

Consumer: the second consumer of ux-browser, a control plane web application whose gate runs its own page checks (Playwright). It ran ux-browser 0.3.0 on 11 pages x 2 viewports on 2026-11-25: page scroll, overflow and always visible gave its own values (0/0); none could replace an own check yet. It reported 8 gaps; line numbers are `docs/browser-adapter.md` at 0.3.0 (2a921f1).

Gap 1 (lines 25, 81). Checked against the source at 2a921f1: holds: `checks` serves `--build` only (`src/serve.rs`); `render` alone takes a full URL.

Order 1 of 8: ordered by what lets a consumer drop an own check (a running page measured reliably first).

## Acceptance

`ux-browser checks --url <origin>` measures the configured pages on a server it did not start; the network guard lets requests to that origin through and blocks the rest; on a recorded fixture the results equal those of the served build; `--url` and `--build` together are a wrong invocation (exit 2).
