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
| `tools/` | the plugin check (`check-plugin.sh`), the docs generator and check (`docs.sh`) and their tests, POSIX sh |
| `research/` | research notes on making UI/UX quality machine-checkable from a specification |
| `reviews/` | dated reviews of the skill and its approach (`<YYYY-MM-DD>-<topic>.md`) |
| `ess/` | ESS specification `uxsearch` of the UX search engine: domains, the studio `ess-ui/1` document and its tests, `GAPS.md`; gate `ess specify validate --path ess`, `ess compile --path ess`, `ess ui check --path ess/ui/studio.ui.yaml --model ess`, `ess ui test --path ess/ui/studio.ui.yaml --model ess ess/ui/tests/studio.yaml` |
| `docs/` | design documents of the engine: `brief.md` (glossary), `concept.md`, `state-space.md`, `metrics.md` (the metric and KPI catalogue), `ratchet.md`, `roadmap.md` (state and next steps; read it first) |
| `presets/` | `web-ux.ratchet.yaml`, a proposed ratchet preset family for the metric catalogue |
| `website/` | the public documentation site, Docusaurus on the shared Docs System, served at `https://beyond10x.github.io/uxui/`; `website/README.md` lists what is generated |
| `.github/workflows/` | `pages.yml` (builds and checks the site, uploads it on `main`; no credential, no deploy step) and `b10x-docs-site.yml` (hands that artifact to the shared publisher) |

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

## Documentation site

`website/` is the Docusaurus site served at `https://beyond10x.github.io/uxui/`, on the shared
Docs System pinned by commit in `website/package.json`. The site build is the only Node here.
`tools/docs.sh` generates `website/docs/reference/skill.md`, `metrics.md`, `glossary.md`,
`specification.md`, `website/docs/roadmap.md` and `website/docs/status.md` from the skill,
`docs/`, `ess/` and `website/data/status.json`; never edit those by hand. A change to one of their
sources is followed by `task docs-generate` in the same commit. `task docs-check` (a gate step)
fails on a stale or edited generated page and on a page that breaks the page rules in
`website/README.md`. A release or a roadmap change updates `website/data/status.json` and the
eyebrow of `website/product.json`, which names the plugin version.

`.github/workflows/pages.yml` builds the site with `contents: read` and no credential, runs the
docs check, binds the build to its commit with `sh tools/docs.sh provenance`, and on a push to
`main` uploads `b10x-project-site`. `.github/workflows/b10x-docs-site.yml` hands that exact
artifact to the shared `project-site.yml` publisher, pinned by commit, which deploys it to Pages.
Do not add a deploy step to `pages.yml` or a credential to either workflow. Do not claim the site
is deployed until the live site serves its commit's provenance.

The site does not mention, link to or depend on uilab (the owner, 2026-11-24); the docs check
fails on a page that names it.

## Gate

`task check` must pass before every commit: `claude plugin validate --strict` on `plugin` and on
the marketplace, `tools/check-plugin.sh` (manifests and marketplaces agree; every skill directory
has a `SKILL.md` with `name` equal to the directory and a `description`; every `references/` path
a skill names exists), `tools/docs.sh check`, and `tools/check-plugin.test.sh` and
`tools/docs.test.sh`, which prove the checks fail on broken copies of the tree (written under
`target/`). The whole gate takes a few seconds. Building the site (`task docs-site`) is not part
of the gate.
