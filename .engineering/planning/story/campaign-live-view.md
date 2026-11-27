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
revision: 7
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
(2026-11-27) and already serves a live page that polls two JSON-lines files every 2 s: one line per
finished candidate (`iteration`, `at`, `kind`, `change`, `accepted`, `score`, `score_keys`,
`metrics`, `deltas`, `worse`, optional `screenshots`) and one line per phase (`at`, `iteration`,
`phase`, `text`, optional `screenshots`). The owner decided (D25) that the live view builds on that
page rather than a second one.

## Acceptance

On the fixture page of the browser lane (`engine/search/tests/fixtures/campaign/live/`):

- `campaign run --events <dir>` appends, flushed as written, one candidates line per finished
  variant (all its environments measured: `change` names the variant and its axis values, `metrics`
  its term values, `score` its rank score so far, `accepted` whether it is on the frontier so far,
  `kind` `baseline` for the page as shipped) and one phase line per ranking update (`phase`
  `ranking`, `text` the top three) and per start and end of the run.
- The final measurements file is the same bytes as a run without `--events`.
- After the run, the last ranking line names the same top three as `campaign report` on the run's
  measurements.
- The two line shapes are documented in `docs/targets.md`, and a test holds every field name the
  document lists to what the run writes.
- Each line is held by a test that failed before the change; tests start no browser except in the
  browser lane.
