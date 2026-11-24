# AGENTS.md — uxui

The `uxui` agent plugin for Claude Code and Codex, and its marketplace. It holds one skill,
`ui-iteration-loop`: a measured improvement loop on a web UI with hard gates on every merge and
lower-is-better ratchets against a baseline.

Organization-wide rules live in the workspace `AGENTS.md` (the `AGENTS.md` of the directory that holds the beyond10x repositories); this file adds
what is specific to uxui.

## Ownership

| path | what |
|---|---|
| `plugin/skills/<name>/` | the skills: `SKILL.md` and its `references/`; the one source both hosts load |
| `plugin/.claude-plugin/plugin.json` | Claude Code manifest |
| `plugin/.codex-plugin/plugin.json` | Codex manifest; `skills` points at `./skills/` |
| `.claude-plugin/marketplace.json` | Claude Code marketplace `uxui` |
| `.agents/plugins/marketplace.json` | Codex marketplace `uxui` |
| `tools/` | the plugin check (`check-plugin.sh`) and its test, POSIX sh |
| `research/` | research notes on making UI/UX quality machine-checkable from a specification |
| `reviews/` | dated reviews of the skill and its approach (`<YYYY-MM-DD>-<topic>.md`) |
| `ess/` | ESS specification `uxsearch` of the UX search engine: domains, the studio `ess-ui/1` document and its tests, `GAPS.md`; gate `ess specify validate --path ess`, `ess compile --path ess`, `ess ui check --path ess/ui/studio.ui.yaml --model ess`, `ess ui test --path ess/ui/studio.ui.yaml --model ess ess/ui/tests/studio.yaml` |
| `docs/` | design documents of the engine: `brief.md` (glossary), `concept.md`, `state-space.md`, `metrics.md` (the metric and KPI catalogue), `ratchet.md`, `roadmap.md` (state and next steps; read it first) |
| `presets/` | `web-ux.ratchet.yaml`, a proposed ratchet preset family for the metric catalogue |
| `generators/own/` | `uxui-gen`, the smallest own generator: one page of the studio document as static HTML and plain JS (`docs/own-generator.md`); Rust, no third-party crates; workspace `Cargo.toml` and `rust-toolchain.toml` at the root |
| `engine/` | the engine, Rust, no third-party crates: `doc` (`uxui-doc`: the YAML subset reader, the JSON writer, SHA-256 and the node tree of an `ess-ui/1` document, shared with `uxui-gen`) and `static` (`ux-static@1`: static metrics as ratchet evidence records, `docs/ux-static.md`; its declarations file `docs/uxui-declarations.md`, the studio's `ess/ui/studio.uxui.yaml`) |

## Direction

The goal is to implement all of this repository's tooling on its own, moving towards zero third-party
dependencies. Not required at first; every new tool is chosen with that direction in mind. The owner,
2026-11-23: "we really would like to implement all the tooling entirely on our own. So basically
zero dependencies in some sense [...] Not a big deal at first".

## Rules

- The plugin version stands in three places and they agree: `plugin/.claude-plugin/plugin.json`,
  `plugin/.codex-plugin/plugin.json` and the plugin entry of `.claude-plugin/marketplace.json`.
  `CHANGELOG.md` has a `## <version> — <YYYY-MM-DD>` heading for it.
- Name, description, author, license and repository are the same in both manifests.
- A skill's behaviour lives in its `SKILL.md` and `references/`, never only in a Claude Code
  `commands/` or `agents/` file: Codex loads skills only.
- Tools are POSIX sh or Rust; no Python.
- Remote: `github.com/beyond10x/uxui`, public. Every commit is authored by the organization bot and
  reaches GitHub through `b10x-gates` (`bot -- commit`, `check`, `publish`), never plain `git push`
  or `gh`. No company, customer or person names, internal hosts or home paths in any file.

## Gate

`task check` must pass before every commit: `claude plugin validate --strict` on `plugin` and on
the marketplace, `tools/check-plugin.sh` (manifests and marketplaces agree; every skill directory
has a `SKILL.md` with `name` equal to the directory and a `description`; every `references/` path
a skill names exists), and `tools/check-plugin.test.sh`, which proves the check fails on broken
copies of the tree (written under `target/`), `cargo test -p uxui-gen` and `cargo test -p ux-static`. The whole gate takes a few
seconds once the crate is compiled.
