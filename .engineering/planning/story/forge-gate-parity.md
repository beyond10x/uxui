---
format: aep.planning-md/3
id: story:forge-gate-parity
kind: story
status: implemented
title: The forge runs the release gate on every pull request, and a tag waits for it
relations:
- decomposes: epic:engine-slices
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T05:03:50Z", actor: "human:operator", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-25T05:03:50Z", actor: "human:operator", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-25T05:43:34Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
# Story: The forge runs the release gate on every pull request, and a tag waits for it

## Outcome

A change whose Rust tests fail on the forge's runner is red on its pull request, before it merges; and a release tag is pushed only on a commit whose required forge runs ended green.

## Context

2026-11-25: 0.8.0 was tagged on 5595daf and its release gate (run 45545202319) failed on two `values::` tests (exit 3 instead of 1: #32 looked for Chrome before refusing the inputs; the runner has no browser). #32's pull request had been green: its checks are `Build Docusaurus` (the documentation site) and `b10x-gates / local evidence` (a receipt of the local run). No pull-request check runs `cargo test` on the forge; the only forge test run is `release.yml`'s Gate, after the tag. The release script waited for the runs on main, which are the two documentation workflows only.

Host operator, 2026-11-25 05:2x CET: "The PR check and the release gate must run the same script on the same runner image; name the difference and remove it" and "Add the check llm built in beyond10x/llm#103 (tag only on a commit whose runs ended green) or its equivalent to uxui's release task."

## Acceptance

- One script, `tools/gate.sh` (POSIX sh), holds the checks that need no local login (`tools/check-plugin.sh`, `tools/docs.sh check`, `cargo test --workspace --locked`); `release.yml`'s Gate job and a new workflow on `pull_request` and on `push` to `main` both run it, on the same runner image and the toolchain of `rust-toolchain.toml`.
- On a pull request whose changed files are all planning or prose files (`.engineering/`, `*.md` outside the built docs site), the job runs, finds only such paths and exits 0 without building; one path list in the workflow, with a test case for each side.
- `task release:ready -- <sha>` (and the release procedure before `git tag`) refuses, naming the run, when a required run on the commit (the new workflow, `Documentation validation`, `Documentation site`) is absent, pending or not successful; a test with a stubbed `gh` covers absent, pending, red and green.
- Making the new job a required check is a ruleset change and stays the owner's; the pull request says so.
