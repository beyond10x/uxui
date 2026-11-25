# Changelog

## Unreleased

- The page-scroll slack and the overflow rule are configuration: `layout: {page_slack_px,
  overflow: {tolerance_px, elements, exclude}}` in a `ux-browser-checks/1` file. A unit counts for
  `layout.page_scroll` when the page scrolls sideways by more than `page_slack_px` (default 0); an
  edge overflows when it passes the viewport or its block by more than `tolerance_px` (default 1);
  `elements` are the CSS selectors measured besides the text runs (default 0.3.0's ten) and nothing
  inside a match of an `exclude` selector is overflow (default none). The defaults are 0.4.0's rule:
  the recorded pages give the same values, and on llm's documentation build the 135 numbers, results
  and values file are byte for byte those of the probe before; on the second consumer's snapshot
  page scroll and overflow are 0 / 0 on 55 units with its own slack of 1 px, as its own gate has
  them. `examples/llm-docs.evidence.json` is rewritten by such a run. The overflow probe takes the
  elements and exclusions as inputs, returns them with its geometry and reports per item the
  exclusion it lies in; the rule is applied in Rust, and a capture made with other inputs does not
  read back. The document's `conditions.other` names the values used (`page_slack_px`,
  `overflow_tolerance_px`, `overflow_elements`, `overflow_exclude`), and the assessor declaration
  lists them.
- An always-visible item (and so a keyboard target) may be located without a role: by `css` alone,
  the first visible match in its `scope` with no accessible name checked, or by the new `current:
  true`, the element whose `aria-current` is not `false` in its `scope` (with `role` or `css`, only
  their current candidates). One declaration of a per-page item, the current page's entry of a
  navigation, replaces one item per page. An item with neither `role` nor `css` nor `current: true`
  is still refused, and so is `name_is` or `name_contains` without a `role`. The candidate probe
  records `aria-current` (`current` in a `--capture` file). Items with a role compute as in 0.3.0.

## 0.4.0 — 2026-11-25

- `ux-browser checks` waits for a ready condition the page sets, for a page that draws after its
  `load` event: `ready: {selector: <css>}` (an element matches it) or `ready: {global: <name>}`
  (`window.<name>` set to `true`), within `ready_timeout_ms` (default 10000). Ready is then the
  condition, fonts loaded and two frames; `layout_shift.after_ready` counts from there, and each fresh
  load of the keyboard walk waits for it too. The page is watched (a `MutationObserver`, a property
  trap), not polled. A page that does not set its condition in time has every result incomplete
  (`incompatible_target`, the detail names the condition). Without `ready` every value is 0.3.0's.
- Every checks document names its ready condition in `evidence.conditions.other.ready` (`load`
  without one) and the assessor declaration lists `ready` among its conditions; a 0.3.0 document has
  no `ready`. `engine/browser/examples/llm-docs.evidence.json` is rewritten by such a run: the same
  135 numbers, findings and 1,234 checks. On the second consumer's snapshot, 5 runs gave 5 equal
  documents with a ready condition that covers everything the page draws late, where without one
  `layout_shift.after_ready` differed between runs on 10 of 11 pages (`docs/browser-adapter.md`).
- `ux-browser checks --url <origin[/path]>` measures a server the run did not start (a dev server,
  a preview, a deployed origin) in place of `--build`; exactly one of the two is given (both or
  neither: exit 2, as is a URL that is not `http(s)://host[:port][/path]`). Without a path a page is
  its configured path on that origin; a path is where the configuration's `base_path` lies on the
  server. `pages.sitemap` is read from the server, under the base path, in the run's Chromium (not
  answered with 200: exit 1). The document's target is `kind: deployment` with
  `identifiers.environment_id` the normalized URL, and no `candidate` and no `selection`; the results
  are those of the served build: on llm's documentation build 135 of 135 values equal, on the
  second consumer's snapshot its 44 exact values equal (`docs/browser-adapter.md`, A running server).
  The assessor declaration lists `targets: [build, deployment]` and the input `url`; `build` is no
  longer required.
- The request guard lets only `GET` and `HEAD` reach the measured server, with `--url` and with
  `--build`, unless the configuration's new `request_methods` names more: a measurement does not
  change the state of the server it measures. Before, any method to the served build went on.

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
