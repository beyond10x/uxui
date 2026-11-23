# Changelog

## Unreleased

## 0.1.0 — 2026-11-23

- The `uxui` plugin for Claude Code and Codex with one skill, `ui-iteration-loop`: a measured
  improvement loop on a web UI in rounds of trial, measure, triage, design decision, build and
  gate, with hard gates on every merge and lower-is-better ratchets against a `baseline.json`.
  The skill is moved here unchanged from an earlier private plugin.
- Marketplace `uxui` for both hosts: `.claude-plugin/marketplace.json` and
  `.agents/plugins/marketplace.json`.
- `task check`: `claude plugin validate --strict` on the plugin and the marketplace,
  `tools/check-plugin.sh` (manifests and marketplaces agree on name and version, every skill has
  `name` and `description`, every `references/` path a skill names exists) and its test.
