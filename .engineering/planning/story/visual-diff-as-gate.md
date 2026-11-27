---
format: aep.planning-md/3
id: story:visual-diff-as-gate
kind: story
status: active
title: A screenshot difference can be a hard gate
relations:
- decomposes: epic:engine-slices
scope:
- confidence: inferred
  path: engine/browser/src/values.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T09:57:01Z", actor: "human:operator", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-27T09:57:01Z", actor: "human:operator", revision: 5}
---
## Outcome

A round can make a screenshot difference block a merge: the values gates of `ux-browser` accept the
visual outputs.

## Context

The visual ratchet (`story:ux-browser-visual-ratchet`) writes `visual.diff_share@1` and
`visual.diff_px@1` into the evidence and the values file, but `values.gates` cannot name a visual
output, so a visual failure is a ratchet value only (`docs/browser-adapter.md`).

## Acceptance

- `values.gates` accepts `visual.diff_share@1` with a bound; a run over the bound exits as a failed
  gate, naming the page, viewport and mode, and a run within it passes.
- A unit without a baseline is not run and is no pass of the gate.
- Each line is held by a test that failed before the change.
