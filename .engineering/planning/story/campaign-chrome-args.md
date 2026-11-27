---
format: aep.planning-md/3
id: story:campaign-chrome-args
kind: story
status: active
title: A campaign passes Chromium switches its caller names
relations:
- decomposes: epic:engine-slices
scope:
- confidence: inferred
  path: engine/search/src/campaign
- confidence: inferred
  path: engine/search/src/main.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T20:00:37Z", actor: "human:operator", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-27T20:00:37Z", actor: "human:operator", revision: 6}
---
## Outcome

A campaign runs where Chromium needs switches of its caller, such as a container that runs as root.

## Context

A consumer's CI runs `ux-search campaign run` in a container image that runs as root. Chromium
there exits with "Running as root without --no-sandbox is not supported". `ux-browser` takes
`--chrome-arg`; `ux-search campaign` takes none, so the consumer wraps the browser in a script that
adds `--no-sandbox` and hands that script to `ux-search` as its Chromium. Decided: a flag the caller
names, as in `ux-browser`; `ux-search` adds no switch on its own when it runs as uid 0, so turning
the sandbox off stays the caller's choice.

## Acceptance

- `ux-search campaign run` and `ux-search campaign check` take `--chrome-arg <arg>`, repeatable;
  each argument reaches Chromium's command line in the given order, after the engine's own
  switches.
- Without the flag the command line is the one of 0.13.1.
- `--help` of both commands names the flag; an empty value is refused naming the flag.
- The measurements of a run do not depend on the flag (a test holds the bytes equal with and
  without a harmless switch).
