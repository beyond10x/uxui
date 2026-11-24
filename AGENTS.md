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
| `.engineering/` | the AEP planning store and the backlog's one home (`aep plan artifact list`); it carries the work items of `docs/roadmap.md`, `docs/generator-round.md`, `docs/ess-gaps.md`, `docs/targets.md` and `docs/comparison.md`, which stay as design records |
| `themes/` | themes: CSS over the variables the stock `ess-ui/1` generator writes, plus what they do not cover; every theme has a light and a dark mode, chosen by `theme-toggle.js` (system setting until a person picks one); `studio.css` is the studio's |

## Direction

The goal is to implement all of this repository's tooling on its own, moving towards zero third-party
dependencies. Not required at first; every new tool is chosen with that direction in mind. The owner,
2026-11-23: "we really would like to implement all the tooling entirely on our own. So basically
zero dependencies in some sense [...] Not a big deal at first".

uxui owns themes and templates, and is where `ess-ui/1` itself may move. Styling is solved here, never by
asking ESS to extend `ess-ui/1`. The owner, 2026-11-24: "you're also owning the domain of themes and
templates. That means you can just locate this in your own repo. [...] we're not trying to extend ESS UI
for styling and stuff."

For every UI, reassess at any time whether the stock generator (`ess generate ui`) or a generator of our
own serves it better, and record the reason. The owner, 2026-11-24: "anytime reassess whether you want to
use a pre-built generator or if you want to own the generator for this specific case, which I observed to
be the better path usually, but maybe things have changed".

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
- Every `aep` write runs with `AEP_ACTOR=human:operator`, so no login name enters `.engineering/`. The owner,
  2026-11-24: "replace human Timo with human operator".
- uxui does not use uilab: no dependency on it, no integration with it, no format taken from it. The
  owner, 2026-11-24: "Don't use UILab, please."

## Gate

`task check` must pass before every commit: `claude plugin validate --strict` on `plugin` and on
the marketplace, `tools/check-plugin.sh` (manifests and marketplaces agree; every skill directory
has a `SKILL.md` with `name` equal to the directory and a `description`; every `references/` path
a skill names exists), and `tools/check-plugin.test.sh`, which proves the check fails on broken
copies of the tree (written under `target/`). The whole gate takes a few seconds.
