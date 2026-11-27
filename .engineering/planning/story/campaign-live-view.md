---
format: aep.planning-md/3
id: story:campaign-live-view
kind: story
status: active
title: Watch a declared campaign climb while it runs
relations:
- decomposes: epic:engine-slices
scope:
- confidence: inferred
  path: engine/search/src/campaign
- confidence: inferred
  path: engine/search/src/main.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T09:57:01Z", actor: "human:operator", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-27T09:57:01Z", actor: "human:operator", revision: 6}
---
## Outcome

A person watches a declared campaign climb while it runs: each variant's render appears as it is
measured, and the ranking and the frontier change on screen until the run ends.

## Context

`ux-search campaign run` renders every variant in every environment and writes one measurements file
at the end; `ux-search campaign report` ranks it afterwards. Nothing shows the climb in progress. A
consumer running a UI loop on a simulation page was asked to show the hill climbing while it runs
(2026-11-27). The studio's live-climb page (`ess/ui/studio.ui.yaml`) draws a climb from a stub, not
from a run.

## Acceptance

On the fixture page of the browser lane (`engine/search/tests/fixtures/campaign/live/`):

- `campaign run --events <file>` appends one JSON line per finished render (variant, environment,
  terms) and one per ranking update, flushed as written; the final measurements file is unchanged
  byte for byte against a run without `--events`.
- `campaign watch --events <file> --port <n>` serves one page that shows the renders so far, the
  current ranking and frontier, and the share of the space done, and updates it without a reload
  while lines arrive; it reads only the events file.
- After the run ends, the page shows the same ranking as `campaign report` on the run's measurements.
- Each line is held by a test that failed before the change; tests start no browser except in the
  browser lane.
