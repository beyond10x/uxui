---
format: aep.planning-md/3
id: story:workspace-lint-clean
kind: story
status: implemented
title: Workspace clippy and rustfmt clean, and held by the gate
relations:
- decomposes: epic:engine-slices
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T10:46:13Z", actor: "human:operator", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-25T10:46:13Z", actor: "human:operator", revision: 5}
- {from: "active", to: "implemented", at: "2026-11-25T10:46:53Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
# Story: Workspace clippy and rustfmt clean, and held by the gate

## Outcome

`cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check` exit 0 on `main`, and the gate fails when either does not.

## Context

Found 2026-11-25 by the implementor of story:ux-browser-version-flag on main 57c001c:

- `cargo clippy --workspace --all-targets -- -D warnings` exits 101: `generators/own/src/binding.rs:38` (single_element_loop) and `:107` (collapsible_if), crate `uxui-gen`.
- `cargo fmt --all --check` exits 1: diffs in `generators/own/src/*.rs`, `generators/own/tests/main.rs` and `engine/static/tests/main.rs:604`.
- The per-crate lint of ux-browser, ux-static and uxui-doc exits 0. The stories of this session ran clippy and fmt per crate (`-p ux-browser`), so the workspace never was checked; neither command runs in `tools/gate.sh` or the release Gate.

## Acceptance

Both commands exit 0 on the workspace; `tools/gate.sh` (story:forge-gate-parity, PR #37) runs both (so the forge `Gate`, the release Gate and `task check:pool` hold them); a planted lint or format diff in a test copy makes the gate fail.
