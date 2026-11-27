---
format: aep.planning-md/3
id: story:gate-step-budget
kind: story
status: active
title: The Gate job keeps the test budget on the forge
relations:
- decomposes: epic:engine-slices
scope:
- confidence: inferred
  path: .github/workflows/ci.yml
- confidence: inferred
  path: engine/search/Cargo.toml
- confidence: inferred
  path: tools/gate.sh
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T18:47:51Z", actor: "human:operator", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-27T18:47:51Z", actor: "human:operator", revision: 7}
---
## Outcome

The `Gate` job of a pull request ends within the test budget of the organization's rules (the test
step within 2 minutes of wall time, no test over 60 s), on the forge and locally.

## Context

The `Gate` check of the release pull requests of 2026-11-27 ran 6m47s (pull request 72) and 7m19s
(pull request 75). Run 46039866893 (pull request 75) spends it compiling, not testing:

- `engine/search` lint build: `Finished dev profile [unoptimized + debuginfo] in 1m 55s`;
- `engine/search` test build: `Finished test profile [unoptimized + debuginfo] in 3m 58s`;
- the tests themselves: `ux-browser` 246 in 3.42 s, `engine/search` 174 in 15.67 s.

`engine/search` builds its third-party crates (through the ESS git dependency) from nothing on
every run, with `opt-level = 2` for every dependency (`[profile.dev.package."*"]` of
`engine/search/Cargo.toml`) and with debug info, which the organization's rules exclude from test
builds; the workflow keeps no compile cache between runs. The local gate does not see this: the
host's compile cache and `debug = false` hide it.

## Acceptance

- The `Gate` job of a pull request that changes a source file ends in at most 2 minutes from the
  start of `tools/gate.sh`, on a run whose dependency cache is warm; the run's log shows it.
- No test build of either workspace carries debug info, on the forge as locally (`readelf -S` on
  one test binary: no `.debug_*` section).
- The runner kills a test after 60 s and the step prints its ten slowest tests, as the rules ask.
- The local gate and the forge build with the same profile settings.
