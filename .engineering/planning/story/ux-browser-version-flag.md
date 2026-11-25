---
format: aep.planning-md/3
id: story:ux-browser-version-flag
kind: story
status: draft
title: ux-browser --version
relations:
- decomposes: epic:engine-slices
revision: 2
---
# Story: ux-browser --version

## Outcome

A consumer proves which ux-browser a binary is before it runs it: `ux-browser --version` prints `ux-browser <version>` and exits 0.

## Context

Consumer: llm, 2026-11-25. Its next wave (story docs-ux-layout-checks-from-ux-browser) pins ux-browser 0.8.1 by tag and SHA-256 and resolves it as it resolves `ess`: a stored or `PATH` binary is started only when its `--version` answer equals the pin. ux-browser 0.8.1 has `render`, `checks`, `values` and `--help`, and no `--version` (`engine/browser/src/main.rs` at tag 0.8.1). Today llm proves a fetched asset by its digest and compares `evidence.conditions.tools.ux-browser` with the pin only after the run; a binary on `PATH` has no proof before it runs.

## Acceptance

`ux-browser --version` (and `-V`) prints exactly `ux-browser <crate version>` and a newline on stdout and exits 0, starts no browser and reads no file; the version equals `CARGO_PKG_VERSION` and the `ux-browser` value in `evidence.conditions.tools` of a run; `--help` lists it; `tools/release-versions.sh` checks that the built binary's answer equals the tag; `ux-static --version` answers the same way for `ux-static`.
