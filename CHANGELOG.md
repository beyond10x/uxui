# Changelog

## Unreleased

## 0.3.0 — 2026-11-24

- `ux-browser values --config <checks> --evidence <document> --out <file>` writes a consumer's values
  file `ux-browser-values/1` from a checks document: per page scenario the numbers its ratchet
  compares, per page, viewport and mode the hard-gate failures in the key form
  `gate<N>|<page>|<viewport>|<mode>|<detail>`, each with the open finding record that waives it, the
  incomplete results and the pages without a scenario. It exits 1 on an incomplete result, a page
  without a scenario or a failure no open finding waives, after writing. `ux-browser checks --values
  <file>` does the same in one run. The scenario files, the finding records, the gate of each output
  and the gating viewports are declared in the configuration's new `values` section; on llm's
  fixture document it writes the bytes llm's `ux/page-checks.sh` wrote, which it replaces.
- The opt-in check `layout_shift.load` (`opt_in: [layout_shift.load]`): every layout shift since the
  navigation started, as llm's round mode read it. `keyboard.viewports` limits the keyboard walk to
  some viewports. Every number of a checks document has its per-viewport values in
  `details.viewports`. `engine/browser/examples/llm-round.checks.yaml` is llm's round mode: five
  viewports, two of them gating, settle 750 ms, the load-time layout shift.

## 0.2.0 — 2026-11-24

- Specification round 3: browser targets (base URL, build command), source patches as axis values,
  scripted journeys, source diffs on promotion, and the funnel's lifted component with its binding
  table; a lift that does not reproduce the component is refused (#10). A browser journey names its
  nodes by CSS selector; scripted steps visit, click, press, scroll and wait.
- `ux-browser` (`engine/browser`, no third-party crates): drives Chromium over the DevTools protocol
  with its own WebSocket client. `ux-browser render` writes a render record (box, computed styles and
  accessibility entry per declared node, plus a screenshot). `ux-browser checks` writes one
  `ratchet-evidence/1` document per run with five page checks: page scroll, overflow, always-visible
  items, keyboard reach and layout shift after ready, with viewports, modes and declared items as
  input. On the `llm` documentation site (27 pages) it equals that repository's own harness on 135 of
  135 values. axe is not covered.
- `ux-browser checks` measures up to `--jobs` pages at a time in one Chromium (default: the CPUs, at
  most 4): llm's 27 pages in 27 s with 4 where one at a time took 92 s, the same 1,234 values. Its
  evidence names the served build as target kind `build` (candidate and `artifact_digest`: the build
  digest), declared in `engine/browser/ux-browser.assessor.yaml`.
- `research/2026-11-24-loop-copies-page-checks.md`: the page checks of the eight UI loop copies in
  seven repositories, and how they differ.

## 0.1.0 — 2026-11-24

The first tagged release. It holds the plugin as first published and the work merged on 2026-11-24
(pull requests #1 to #8 and #5).

- The `uxui` plugin for Claude Code and Codex with one skill, `ui-iteration-loop`: a measured
  improvement loop on a web UI in rounds of trial, measure, triage, design decision, build and
  gate, with hard gates on every merge and lower-is-better ratchets against a `baseline.json`.
  The skill is moved here unchanged from an earlier private plugin.
- Marketplace `uxui` for both hosts: `.claude-plugin/marketplace.json` and
  `.agents/plugins/marketplace.json`.
- Specification `uxsearch` of the UX search engine (`ess/`), reviewed and hardened: 10 authored
  scenarios catch 24 of 24 planted defects (#1). Reuse commands replace the row-set collision
  guards; conformance synthesis 11 s, from 325 s on the same build node (#5).
- `ux-static@1` (`engine/static`, crate `ux-static`): reads an `ess-ui/1` document, its journeys and
  a `uxui-declarations/1` file, computes the static checks and writes one `ratchet-evidence/1`
  document per run, following revision 2 of the format and its assessor contract, with the
  declaration `ux-static.assessor.yaml`; exit codes 0, 1, 2, 3 (#2, #4).
- Own generator for the studio's live climb page (`generators/own`, crate `uxui-gen`, no third-party
  crates), with the measured comparison against the stock ESS generator (#1).
- Themes: `themes/studio.css` and `themes/theme-toggle.js`, light and dark mode (#1).
- Documentation site (`website/`) on the shared Docs System, published to GitHub Pages; derived
  pages generated and checked by `tools/docs.sh` (#1).
- Campaign funnel spike (`tools/scaffold/`): lift a page component to `ess-ui/1`, brute-force its
  variants on the cell model, project the best onto the real page in Chromium. First real campaign
  on the documentation landing page, with a content rule; the landing page took its result (#1, #3).
  Stage 1 reflows the cell drawing; rank agreement with the browser 0.998 on that campaign (#7).
- Research on how UI check tools work and which UI metrics have evidence (`research/`, #1).
- Planning store (`.engineering/`), synced with the merged work (#1, #6).
- Gate: `task check`; on a host with a build pool, `task validate` on the host and
  `slot task check:pool` in the pool (#8).
