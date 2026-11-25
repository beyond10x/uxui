# uxui

An agent plugin for Claude Code and Codex that improves a web UI in measured rounds.

**Documentation: <https://beyond10x.github.io/uxui/>** (install, the loop, the search engine, the
skill and metric reference). Releases: <https://github.com/beyond10x/uxui/releases>.

The skill `ui-iteration-loop` drives rounds of trial, measure, triage, design decision, build and
gate on an app panel, a Vue app or a docs website. Every merge passes hard gates: no scroll at the
gating viewports, always-visible controls fully in view, an unmissable completion state, a
screen-only user bot that completes every scenario, and axe-core with 0 serious or critical
violations. Every finding becomes a regression test, and lower-is-better ratchets against a
committed `baseline.json` make sure no merge makes a measured value worse.

The skill names the tools to build the loop on (Playwright, axe-core, rrweb, ffmpeg); the plugin
ships no scripts.

## Install

Install from a clone of this repository.

Claude Code:

```console
git clone https://github.com/beyond10x/uxui
claude plugin marketplace add ./uxui
claude plugin install uxui@uxui
```

Codex:

```console
git clone https://github.com/beyond10x/uxui
codex plugin marketplace add ./uxui
codex plugin add uxui@uxui
```

Start a new session after installing. Ask for a UI improvement round, a UX round or a visual
regression ratchet, or call the skill by name: `/uxui:ui-iteration-loop` in Claude Code,
`$ui-iteration-loop` in Codex.

## Documents

- The documentation site, <https://beyond10x.github.io/uxui/>: install, the loop, the search
  engine, the skill and metric reference, status and limitations. Its source is `website/`
  (`website/README.md`).
- `research/`: research notes on making UI/UX quality machine-checkable from a specification.
- `reviews/`: dated reviews of the skill and its approach.
- `docs/`: design of the UX search engine (start with `docs/roadmap.md`, `docs/brief.md` and `docs/concept.md`).
- `ess/`: its ESS specification, including the studio UI.
- `presets/`: a proposed ratchet preset family for the metric catalogue.
- `engine/`: the engine's first code; `ux-static` writes static UX metrics of an `ess-ui/1` document as
  one `ratchet-evidence/1` document per run (`docs/ux-static.md`, `task ux:static`).

## Check

```console
task check
```

## License

Apache-2.0.
