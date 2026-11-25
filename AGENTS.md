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
| `tools/` | the gate (`gate.sh`), the plugin check (`check-plugin.sh`), the workflow check (`check-workflows.sh`), the docs generator and check (`docs.sh`), the path list of planning and prose files (`prose-only.sh`), the release checks (`release-versions.sh`, `release-ready.sh`), the repeat of the test binaries (`repeat-tests.sh`) and their tests, POSIX sh |
| `research/` | research notes on making UI/UX quality machine-checkable from a specification |
| `reviews/` | dated reviews of the skill and its approach (`<YYYY-MM-DD>-<topic>.md`) |
| `ess/` | ESS specification `uxsearch` of the UX search engine: domains, the studio `ess-ui/1` document and its tests, `GAPS.md`; gate `ess specify validate --path ess`, `ess compile --path ess`, `ess ui check --path ess/ui/studio.ui.yaml --model ess`, `ess ui test --path ess/ui/studio.ui.yaml --model ess ess/ui/tests/studio.yaml` |
| `docs/` | design documents of the engine: `brief.md` (glossary), `concept.md`, `state-space.md`, `metrics.md` (the metric and KPI catalogue), `ratchet.md`, `roadmap.md` (state and next steps; read it first) |
| `presets/` | `web-ux.ratchet.yaml`, a proposed ratchet preset family for the metric catalogue |
| `.engineering/` | the AEP planning store and the backlog's one home (`aep plan artifact list`); it carries the work items of `docs/roadmap.md`, `docs/generator-round.md`, `docs/ess-gaps.md`, `docs/targets.md` and `docs/comparison.md`, which stay as design records |
| `themes/` | themes: CSS over the variables the stock `ess-ui/1` generator writes, plus what they do not cover; every theme has a light and a dark mode, chosen by `theme-toggle.js` (system setting until a person picks one); `studio.css` is the studio's |
| `website/` | the public documentation site, Docusaurus on the shared Docs System, served at `https://beyond10x.github.io/uxui/`; `website/README.md` lists what is generated |
| `.github/workflows/` | `ci.yml` (`Gate`: `tools/gate.sh` on every pull request and push to `main`), `release.yml` (the same gate, then the release assets, on a tag), `pages.yml` (builds and checks the site, uploads it on `main`; no credential, no deploy step) and `b10x-docs-site.yml` (hands that artifact to the shared publisher) |
| `generators/own/` | `uxui-gen`, the smallest own generator: one page of the studio document as static HTML and plain JS (`docs/own-generator.md`); Rust, no third-party crates; workspace `Cargo.toml` and `rust-toolchain.toml` at the root |
| `engine/` | the engine, Rust, no third-party crates: `doc` (`uxui-doc`: the YAML subset reader, the JSON writer, SHA-256 and the node tree of an `ess-ui/1` document, shared with `uxui-gen`) and `static` (`ux-static@1`: static metrics as one `ratchet-evidence/1` document per run, `docs/ux-static.md`; its declarations file `docs/uxui-declarations.md`, the studio's `ess/ui/studio.uxui.yaml`) |
| `engine/browser/` | `ux-browser@1`, the browser adapter (`docs/browser-adapter.md`): Chromium driven over its own DevTools-protocol client (WebSocket, JSON, no crate), a static server for a built site or a running server it did not start (`checks --url`), the render record `ux-browser-render/1` and the five shared page checks (and the opt-in `layout_shift.load`) as one `ratchet-evidence/1` document per run, and from it a consumer's values file `ux-browser-values/1` (`ux-browser values`, `checks --values`); `examples/` holds the declarations of the documentation landing page and of the first consumer's site; its tests start no browser and read geometry recorded from real runs (`tests/fixtures/`) |

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
the marketplace, then `tools/gate.sh`, the checks that need no local login: `tools/check-plugin.sh`
(manifests and marketplaces agree; every skill directory has a `SKILL.md` with `name` equal to the
directory and a `description`; every `references/` path a skill names exists),
`tools/check-workflows.sh` (the pull-request gate and the release gate are the same), `tools/docs.sh
check`, the test scripts `tools/check-plugin.test.sh`, `tools/release.test.sh`, `tools/docs.test.sh`,
`tools/gate.test.sh` and `tools/release-ready.test.sh`, which prove the checks fail on broken copies
of the tree (written under `target/`), `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
--locked -- -D warnings` (the forge installs both components in each gate job), and `cargo test
--workspace --locked` (no browser starts in
it; the real runs are measurements, `docs/browser-adapter.md`). The whole gate takes a few seconds
once the crates are compiled. Building the site (`task docs-site`) is not part of the gate.

The forge runs the same `tools/gate.sh` on `ubuntu-latest` with the toolchain of
`rust-toolchain.toml`: the workflow `Gate` (`.github/workflows/ci.yml`) on every pull request and
every push to `main`, and the release workflow's Gate job on a tag. What the runner does
differently is made the same in `tools/gate.sh`: it has no browser (the script hides the Playwright
cache and `UXUI_CHROME`), and it builds with `--locked`. The gate runs the test binaries 4 more times (`tools/repeat-tests.sh`): two tests that race on a scratch directory failed on the forge only (#38, the 0.9.0 release gate). A pull request whose changed files are all
planning or prose files ends `Gate` green without building; the one list of those paths is
`tools/prose-only.sh`. Whether `Gate` is a required check is the owner's ruleset.

On a host with a shared build pool the gate runs in two parts:

- `task validate` runs on the host, without the pool. It needs the host's `claude` executable and
  its login, which the pool's nodes do not have, and it compiles nothing and starts no test runner.
  The operator's rule of 2026-11-24.
- `slot task check:pool` runs the rest in the pool: `sh tools/gate.sh`.

`task check` runs both parts in one process, for a checkout without a pool.

## Release

Bare SemVer tags (`0.2.0`). On a green `main` with entries under `## Unreleased`, the session cuts the
release without asking (the operator's order of 2026-11-24) and reports the facts.

1. One pull request: `version` of every crate under `engine/` and `generators/`, of
   `plugin/.claude-plugin/plugin.json` and the marketplaces; `CHANGELOG.md` entries move under
   `## <version> — <YYYY-MM-DD>`. `sh tools/release-versions.sh <version>` exits 0 on that commit.
2. After the merge, wait until `task release:ready -- <sha>` (`tools/release-ready.sh`; `<sha>` the
   full id of the merge commit) exits 0: it asks the forge for the runs on that commit and refuses,
   naming each, while one of `Gate`, `Documentation validation` and `Documentation site` is absent,
   pending or did not end `success`. No tag without it. Then an annotated tag on the merge commit
   as the bot,
   `b10x-gates bot -- tag -a <version> -m "uxui <version>" <sha>`, pushed with
   `b10x-gates bot --repository beyond10x/uxui -- push origin refs/tags/<version>`
   (`b10x-gates publish` takes branch refs only).
3. The tag push runs `.github/workflows/release.yml`: the gate without `claude`, the versions check,
   `ux-browser` and `ux-static` built for x86_64-unknown-linux-gnu with their assessor declarations,
   `SHA256SUMS` and `notes.md` (the changelog section) in the artifact `uxui-<version>-release`.
4. `gh run download <run> -n uxui-<version>-release -D target/release-assets`, then
   `sha256sum -c SHA256SUMS` there.
5. `b10x-gates gh --repo . --repository beyond10x/uxui -- release create <version> -R beyond10x/uxui
   --verify-tag --title "uxui <version>" --notes-file target/release-assets/notes.md
   target/release-assets/*.tar.gz target/release-assets/SHA256SUMS` (paths relative to the checkout).
6. Released means: the release author is `b10x-bot[bot]` and the downloaded assets pass
   `sha256sum -c`.
