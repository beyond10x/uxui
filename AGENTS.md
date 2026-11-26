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
| `tools/` | the gate (`gate.sh`), the plugin check (`check-plugin.sh`), the workflow check (`check-workflows.sh`), the theme check (`check-themes.sh`: every theme has a light and a dark mode that set the same properties, no `var()` of an unset property, `themes/README.md` names every file, each fixture matches its sha256, and `dense.css` and `slot-jobs.css` declare what they promise, against slot's document in `themes/fixtures/`), the docs generator and check (`docs.sh`), the path list of planning and prose files (`prose-only.sh`), the release command (`release-set-version.sh`, § Release) and the release checks (`release-versions.sh`, which lists every file that carries the release's version, and `release-ready.sh`), the repeat of the test binaries (`repeat-tests.sh`) and their tests, and the two campaign recorders, which the gate does not run because each needs a browser (`docs-landing-campaign.sh` builds `ux-search` and the documentation site, builds the site of one variant in a copy of `website/`, runs `ux-search landing run` and writes the measurements, the report, the golden evidence of the tests and the screenshots: the one step after a change of `website/product.json` or `website/src/css/landing.css`, `sh tools/docs-landing-campaign.sh`, and on a host with a build pool `sh tools/docs-landing-campaign.sh --pool`; no release needs it; `studio-campaign.sh` generates and serves the studio app, runs `ux-search campaign run` twice, compares the bytes and writes the measurements, the report, the screenshots and the tables of `docs/campaign-2026-11-26-studio-engine.md`: the one step after a change of `ess/ui/studio.ui.yaml` or `themes/studio.css`, `sh tools/studio-campaign.sh`, and on a host with a build pool `sh tools/studio-campaign.sh --pool`, which runs it as one job of the pool), POSIX sh |
| `research/` | research notes on making UI/UX quality machine-checkable from a specification |
| `reviews/` | dated reviews of the skill and its approach (`<YYYY-MM-DD>-<topic>.md`) |
| `ess/` | ESS specification `uxsearch` of the UX search engine: domains, the studio `ess-ui/1` document and its tests, `GAPS.md`; gate `ess specify validate --path ess`, `ess compile --path ess`, `ess ui check --path ess/ui/studio.ui.yaml --model ess`, `ess ui test --path ess/ui/studio.ui.yaml --model ess ess/ui/tests/studio.yaml`; `ess/ui/rounds/` holds the rounds of the `ui-iteration-loop` skill on the studio (the React app generated from the document on its fixtures, measured with `ux-browser`): the round command `sh ess/ui/rounds/round.sh <R-nn>`, its scenario cards and declarations, one directory per round (`report.md`, `round.json`, `triage.md`, shots) and `trend.md`; `ess/ui/findings/` holds one record per finding (`UXF-nnn.md`); a round is a measurement, not a gate step |
| `docs/` | design documents of the engine: `brief.md` (glossary), `concept.md`, `state-space.md`, `metrics.md` (the metric and KPI catalogue), `ratchet.md`, `roadmap.md` (state and next steps; read it first) |
| `presets/` | `web-ux.ratchet.yaml`, a proposed ratchet preset family for the metric catalogue |
| `.engineering/` | the AEP planning store and the backlog's one home (`aep plan artifact list`); it carries the work items of `docs/roadmap.md`, `docs/generator-round.md`, `docs/ess-gaps.md`, `docs/targets.md` and `docs/comparison.md`, which stay as design records |
| `themes/` | themes and templates: CSS over the variables the stock `ess-ui/1` generator writes, plus what they do not cover, and `dense.css` over the uxui class hooks (`themes/README.md`); every theme has a light and a dark mode, chosen by `theme-toggle.js` (system setting until a person picks one); `studio.css` is the studio's, `slot-jobs.css` slot's job pages (`themes/slot-jobs.md`); `fixtures/` holds the consumer documents the theme check reads |
| `website/` | the public documentation site, Docusaurus on the shared Docs System, served at `https://beyond10x.github.io/uxui/`; `website/README.md` lists what is generated |
| `.github/workflows/` | `ci.yml` (`Gate`: `tools/gate.sh` on every pull request and push to `main`), `release.yml` (the same gate, then the release assets, on a tag), `pages.yml` (builds and checks the site, uploads it on `main`; no credential, no deploy step) and `b10x-docs-site.yml` (hands that artifact to the shared publisher) |
| `generators/own/` | `uxui-gen`, the smallest own generator: one page of the studio document as static HTML and plain JS (`docs/own-generator.md`); Rust, no third-party crates; workspace `Cargo.toml` and `rust-toolchain.toml` at the root |
| `engine/` | the engine, Rust, no third-party crates (`engine/search` excepted, its row below): `doc` (`uxui-doc`: the YAML subset reader, the JSON writer, SHA-256 and the node tree of an `ess-ui/1` document, shared with `uxui-gen`) and `static` (`ux-static@1`: static metrics as one `ratchet-evidence/1` document per run, `docs/ux-static.md`; its declarations file `docs/uxui-declarations.md`, the studio's `ess/ui/studio.uxui.yaml`) |
| `engine/browser/` | `ux-browser@1`, the browser adapter (`docs/browser-adapter.md`): Chromium driven over its own DevTools-protocol client (WebSocket, JSON, no crate), a static server for a built site or a running server it did not start (`checks --url`), the render record `ux-browser-render/1` and the five shared page checks (and the opt-in `layout_shift.load`) as one `ratchet-evidence/1` document per run, and from it a consumer's values file `ux-browser-values/1` (`ux-browser values`, `checks --values`); `examples/` holds the declarations of the documentation landing page and of the first consumer's site; its tests start no browser and read geometry recorded from real runs (`tests/fixtures/`) |
| `engine/search/` | `ux-search`, the engine's first search on layout model v0, ESS's terminal renderer driven in process: `ux-search cost` times one evaluation per UI state (`docs/cost-spike.md`), `ux-search climb` searches theme x section order (per layout column on a page with `columns`) of one page exhaustively and prints the ranked frontier; its tests climb ESS's partner-portal example (`tests/fixtures/`) and the studio's live-climb page (`ess/ui/studio.ui.yaml`, `docs/studio-climb.md`) against a golden frontier each; a change to the studio document or its fixtures regenerates `tests/fixtures/studio.frontier.txt` with the command in `docs/studio-climb.md`. Its second backend is the browser adapter (`engine/browser` and `engine/doc` by path): `ux-search landing run` renders the 144 variants of the documentation landing page in 6 environments in Chromium and `ux-search landing report` ranks the recorded measurements (`docs/campaign-2026-11-26-docs-landing-engine.md`, run with `tools/docs-landing-campaign.sh`), and `ux-search landing promote` applies one frontier variant to the two sources, names it in `engine/search/src/landing/shipped.txt` (the one place the shipped variant stands) and writes a `ratchet-evidence/1` document with a copy of the measurements beside it, both kept in `docs/promotions/` (the declaration of `ux-search@1` is `engine/search/ux-search.assessor.yaml`); its tests start no browser and hold the report over the committed measurements (`tests/fixtures/docs-landing/`) to a golden file. A campaign on any other page is declared, not compiled in: `ux-search campaign run` reads a declaration (`ux-search-campaign/1`, in the YAML subset of `engine/doc`: the page, the environments, the axes as custom-property values per selector, the hard rules and the ordered terms by name from the catalogue in `src/campaign/declaration.rs`), renders every variant in every environment as an injected style sheet and writes a measurements file (`ux-search-campaign-measurements/1`) with the declaration and a digest inside, and `ux-search campaign report` ranks it with the same `rank` code (`docs/targets.md`); the proof is the frontier page of the generated studio app, 48 variants in 4 environments (`docs/campaign-2026-11-26-studio-engine.md`, recorded by `tools/studio-campaign.sh` into `tests/fixtures/campaign/studio/`), and the test module `campaign` holds the declaration, the measurements, the report, the screenshots and the document to each other without a browser. A workspace of its own with its own `Cargo.lock`, outside the root workspace; `tools/gate.sh` runs its format, lint and test steps with `--manifest-path engine/search/Cargo.toml`. The one exception to "no third-party crates": its third-party crates come only through ESS (`ess-ui`, `ess-ui-check`, `ess-ui-tui` by git, `github.com/beyond10x/ess` tag `0.99.5`, named once in the source as `ux_search::ESS_TAG` and held to the manifest, the lock file and this row by the test module `ess_tag`; and what they depend on: ratatui, crossterm, clap, serde, serde_json, serde_norway, sha2 and others, all listed in its `Cargo.lock`); its `Cargo.toml` names no other third-party crate, and no path into an ESS checkout |

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
- No version is edited by hand. Every file that carries the release's version is checked by
  `tools/release-versions.sh` and written by `tools/release-set-version.sh`; a new place that names
  the version goes into both and into the carrier list of `tools/release.test.sh`, which fails on
  a place only one of them knows and on a file the command writes that is not on the list.
- A recording keeps the tool version it was recorded with; no release rewrites it and no check
  holds it to this build. That is one rule for a recorded campaign (its adapter line; the tests
  hold the form `ux-browser <version>`) and for the two example documents of `ux-browser` in
  `engine/browser/examples/`, each of a browser run. A new version there comes from a new run.
- A file a test holds byte for byte carries no version of this build: a golden output has
  `{version}` where the tool's version stands
  (`engine/static/tests/fixtures/ratchet-example/golden.evidence.json`,
  `engine/search/tests/fixtures/docs-landing/promote.evidence.json`; `task bless:static` and
  `task bless:search` write them again through the tests' own switches); a source of a campaign
  (`website/product.json`, `website/src/css/landing.css`) names no version, because its digest
  is in the measurements.
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
`website/README.md`. A roadmap change updates `website/data/status.json` by hand. The plugin
version in it and in `website/docs/index.md` is written by the release command (§ Release), never
by hand. `website/product.json` names no version: the status page carries it.

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
`tools/check-workflows.sh` (the pull-request gate and the release gate are the same),
`tools/check-themes.sh` (the themes under `themes/`), `tools/docs.sh check`, the test scripts
`tools/check-plugin.test.sh`, `tools/check-themes.test.sh`, `tools/release.test.sh`, `tools/docs.test.sh`,
`tools/gate.test.sh` and `tools/release-ready.test.sh`, which prove the checks fail on broken copies
of the tree (written under `target/`), `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
--locked -- -D warnings` (the forge installs both components in each gate job), and `cargo test
--workspace --locked` (no browser starts in
it; the real runs are measurements, `docs/browser-adapter.md`), then the same three for
`engine/search` with `--manifest-path engine/search/Cargo.toml` (its tests drive ESS's terminal
renderer in process and read a recorded browser campaign; no terminal, browser or network). The whole gate takes a few seconds
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
- A pool whose build node builds offline does not have a new ESS tag: after a change of
  `ux_search::ESS_TAG` and the manifest, `cargo update --manifest-path engine/search/Cargo.toml ess-ui
  ess-ui-check ess-ui-tui` moves the lock file, and `slot cargo fetch --manifest-path
  engine/search/Cargo.toml --locked` fetches the tag on the node once, before the first build.

`task check` runs both parts in one process, for a checkout without a pool.

## Release

Bare SemVer tags (`0.2.0`). On a green `main` with entries under `## Unreleased`, the session cuts the
release without asking (the operator's order of 2026-11-24) and reports the facts.

1. One pull request, made by one command: `sh tools/release-set-version.sh <version>`
   (`task release:set-version -- <version>`; a date as second argument, else today). It writes the
   version into every file that carries it: the two plugin manifests and the Claude marketplace,
   every crate under `engine/` and `generators/` and this repository's packages in the two lock
   files, the plugin version in `website/docs/index.md` and `website/data/status.json`, and the
   current release in `docs/roadmap.md`. It moves the entries of `CHANGELOG.md` from
   `## Unreleased` under a new `## <version> — <YYYY-MM-DD>` and keeps an empty `## Unreleased`
   above it, regenerates the pages (`tools/docs.sh generate`) and ends with
   `sh tools/release-versions.sh <version>`, which exits 0 on that commit. Nothing else is part of
   a release: no browser run, no recording rewritten, no golden file blessed, no page edited by
   hand; `sh tools/gate.sh` passes on what the command wrote.

   `CHANGELOG.md` decides, before a file is written. The command refuses, naming `CHANGELOG.md`
   and writing nothing: a release with no entry under `## Unreleased`; a version that is not
   greater than every dated section, the three numbers compared as numbers; a date that is no
   day of the calendar (exit 2). Two repeats are allowed. The version of the section directly
   below `## Unreleased` again: entries written under `## Unreleased` since the cut move into
   that section, in front of its own. The version of the second dated section, while the first is
   greater and nothing stands under `## Unreleased`: the cut above is taken back, byte for byte,
   its entries under `## Unreleased` again. The command does not know what is released: take a
   cut back only before its tag.
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
