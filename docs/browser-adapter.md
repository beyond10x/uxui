# The browser adapter `ux-browser@1`

Status: version 1, 2026-11-24. Roadmap wave 2 (`docs/roadmap.md`, Trajectory), story
`story:browser-adapter`, designed in `docs/targets.md` (the `browser` adapter). Crate `engine/browser`,
binary `ux-browser`. Rust with no third-party crate: it reads and writes with `engine/doc` (`uxui-doc`)
and drives Chromium over a DevTools-protocol client of its own.

It does two things for any page with a URL:

1. **render**: one render record (`ux-browser-render/1`) for one URL at one viewport and mode, with
   each declared node's box, computed styles and accessibility-tree entry, and a screenshot.
2. **checks**: the five page checks that the UI-loop copies share
   (`research/2026-11-24-loop-copies-page-checks.md`), with the first consumer's (`llm`) definitions,
   over every page of a built site, as one `ratchet-evidence/1` document per run.

## How it drives the browser

| part | file | what |
|---|---|---|
| WebSocket | `src/ws.rs`, `src/base64.rs`, `src/sha1.rs` | RFC 6455 client: the opening handshake with its accept check, masked client frames, server frames of any length, fragments, ping and close |
| JSON | `src/json.rs` | a reader into `uxui_doc::yaml::Value`; writing is `uxui_doc::json` |
| protocol | `src/cdp.rs` | commands with ids, responses matched by id, events queued until asked for, flat sessions; a guard that answers every paused request: requests to the served build go on, every other one fails (`BlockedByClient`) |
| Chromium | `src/chrome.rs` | `UXUI_CHROME`, else the newest `chromium-<n>/chrome-linux*/chrome` under the Playwright cache (`PLAYWRIGHT_BROWSERS_PATH`, else `~/.cache/ms-playwright`); started headless with its own profile, `--remote-debugging-port=0`, and the switches of Playwright's headless Chromium that change what a page measures (`--hide-scrollbars`, the hover and pointer media of a headless device, sRGB); ended with `Browser.close`, then by its PID |
| pages | `src/session.rs` | one browser context per page and viewport (as a fresh Playwright context): viewport, `prefers-color-scheme`, `prefers-reduced-motion`, focus emulation, scripts before the page's own, navigation to the `load` event, keys, the accessibility tree, screenshots |
| server | `src/serve.rs` | a static server for a built site under its base path on a free loopback port (file, file plus `.html`, `index.html`; else `404.html` with status 404) |
| probes | `src/probes/*.js` | the JavaScript the page runs: it returns geometry, and decides nothing |
| checks | `src/checks.rs`, `src/evidence.rs` | every rule and every count, in Rust, from the recorded geometry |

The test step (`task test:ux-browser`, `cargo test -p ux-browser`) starts no browser and opens no
socket: framing runs over an in-memory stream, the protocol client over a scripted transport, and each
check over geometry recorded from real runs (`tests/fixtures/`, written with `--capture`). Real browser
runs are measurements, run by hand on a host with Chromium.

## `ux-browser render`

```sh
ux-browser render --declarations engine/browser/examples/uxui-landing.render.yaml \
  --serve website/build --base-path /uxui/ --url /uxui/ \
  --out engine/browser/examples/uxui-landing.render.json
```

Without `--serve`, `--url` is a full URL. The screenshot goes beside the record (`--screenshot` names
another file). The declarations, `ux-browser-render/1`:

| key | value |
|---|---|
| `viewport` | `{width, height}` in CSS pixels |
| `mode` | `light` or `dark`: the emulated `prefers-color-scheme` |
| `settle_ms` | the wait after fonts are loaded and two frames are drawn; default 250 |
| `styles` | the computed properties to record for each node |
| `nodes` | `{name, selector}`: a node is named by a CSS selector; the first match is recorded |

The record: `format`, `adapter` (`{name: browser, tool: ux-browser, version}`), `browser`
(`Chrome/<version>`), `url`, `viewport`, `mode`, `declarations` (`{path, digest, digest_form: bytes}`),
`nodes[]` (`{name, selector, matches, box: {x, y, width, height}, styles, accessibility: {role, name,
ignored}}`; a selector that matches nothing has `matches: 0` and nothing else), and `screenshot`
(`{file, digest, width, height}`).

**Acceptance run** (the story's acceptance): the documentation landing page at 1440 x 900, light,
seven declared nodes (navbar, hero, hero title, primary action, status link, key figures, footer).
Record `engine/browser/examples/uxui-landing.render.json`, screenshot
`engine/browser/examples/uxui-landing.render.png`; every node matched once and has a box, eight computed
styles and its accessibility entry (`navigation "Main"`, `region`, `heading`, two `link`s,
`DescriptionList "Key figures"`, `contentinfo`). The record's probes and accessibility entries are the
fixture `tests/fixtures/render/landing.capture.json`.

## `ux-browser checks`

```sh
ux-browser checks --config engine/browser/examples/llm.checks.yaml --build <llm>/website/build \
  --project llm-docs --assessment page-checks --out llm-docs.evidence.json \
  [--root .] [--class local] [--pages /llm/,/llm/search] [--capture <dir>]
```

It serves `--build` itself, measures every page (the sitemap's, or a list), and writes one
`ratchet-evidence/1` document. `--capture` writes each page's recorded geometry as
`<page>.capture.json`, the input `checks::page_result` reads. Exit statuses as `ux-static`
(`docs/ux-static.md`): 0 written; 1 refused inputs (a configuration or build that does not read); 2
wrong invocation; 3 infrastructure (Chromium did not start or answer, the document was not written). A
page the server does not answer with 200 is no failure of the run: its results are incomplete.

### Configuration `ux-browser-checks/1`

Viewports, modes, settle times and the declared items are inputs, never constants of the tool. llm's,
from `llm/ux/adapter.ts`: `engine/browser/examples/llm.checks.yaml`.

| key | value |
|---|---|
| `base_path` | where the build is served, `/llm/` |
| `pages` | `{sitemap: sitemap.xml}` (a file of the build) or `{paths: [...]}` |
| `viewports` | `{name, width, height}` each |
| `modes` | `light`, `dark`, in order; the page loads in the first, the others are switched in place |
| `mode_attribute` | the attribute of `<html>` that shows the applied mode (`data-theme`); a page that does not apply a mode within 5 s is incomplete |
| `reduced_motion` | emulate `prefers-reduced-motion: reduce`; default true |
| `settle_ms`, `scroll_settle_ms`, `mode_settle_ms` | after ready (250), after the scroll to the bottom (200), after a mode switch (100) |
| `always_visible` | `{name, role, name_is \| name_contains, css, scope, min_width, path_prefix}` each |
| `keyboard` | `{skip_link_is \| skip_link_contains, must_reach, max_tabs, modes, focus_settle_ms}`; `modes` defaults to the last mode; each target is an always-visible item |

Names: `name_is` is the whole accessible name (white space folded, case kept), as Playwright's
`getByRole` with a string name; `name_contains` is a list of parts matched in any case. llm's regular
expressions translate to it: `/search/i` is `[search]`, `/dark and light mode/i` is `[dark and light
mode]`, `/skip to (main )?content/i` is `[skip to content, skip to main content]`.

### The checks

Per page and viewport, as llm's L2 spec does it: a fresh context in the first mode, the page loaded to
its `load` event, ready (fonts loaded, two frames), the settle time, the layout shifts read; then per
mode: scroll to the top, overflow and page scroll, the always-visible items, scroll to the bottom and
the scroll settle time, the items again; the keyboard walk in the keyboard modes.

| result (`scope: page`) | per page | definition (llm's) |
|---|---|---|
| `layout.page_scroll@1` (units) | sum over viewport x mode | 1 when `scrollingElement.scrollWidth - documentElement.clientWidth > 0` |
| `layout.overflow@1` (blocks) | sum | every non-blank text run (Range box) and every `img, svg, video, canvas, iframe, input, select, textarea, table, pre`: overflow when its right edge passes the viewport + 1 px or its nearest non-inline block + 1 px; none under `position: fixed`, `overflow-x: auto \| scroll` or a clipping box; at most one per block |
| `always_visible.misses@1` (items) | sum over viewport x mode x top and bottom | the first visible candidate in scope (role and name from the accessibility tree first, then CSS): missing, clipped (not wholly inside the viewport +-0.5 px) or covered (`elementFromPoint` of its centre is not it or inside it); an item outside its `min_width` or `path_prefix` is not checked |
| `keyboard.misses@1` (checks) | sum over the walks | on a fresh load the first Tab is on the skip link; Enter (or one more Tab) puts focus into its target; on another fresh load up to `max_tabs` Tabs reach each target. A skip link that is not first misses both of its checks |
| `layout_shift.after_ready@1` (score) | largest over the viewports | `layout-shift` entries without recent input after the ready time, summed, rounded to 4 places; read once per viewport, before the first mode switch |

Not covered: axe-core's rules (a third-party library; llm runs `@axe-core/playwright` 4.14.0), reading
load and the screen-only user (llm's other checks), and llm's `measured` viewports of a round. Playwright
blocks service workers; this run does not (a Docusaurus build registers none).

### Evidence

As `ux-static@1` writes it (`docs/ux-static.md`), with these differences:

| field | value |
|---|---|
| `evidence.assessor` | `ux-browser@1` |
| `evidence.target` | ratchet's format has no target kind for a URL. The target is `kind: document`: the served build, identified by its tree digest, `digest_form: tree` (SHA-256 over one line `<sha256 hex>  <path>` per file, sorted by path, as `sha256sum` prints them); `path` only for a build under `--root`. `evidence.candidate` is that digest. |
| `evidence.inputs.config` | the configuration file, `{value: <path>, digest, digest_form: bytes}` |
| `evidence.conditions` | `tools: {ux-browser, chromium}`, `other: {viewports, modes, settle_ms, reduced_motion}` |
| `evidence.coverage` | unit `check`: one check on one viewport and mode (page scroll, overflow, each applicable item at the top and at the bottom), one keyboard check per walk (skip link first, its target, each target), and one counted layout-shift reading per viewport. A page that did not load is one `not_run` entry, id the page path |
| `evidence.results[]` | per page the five numbers, then per page a `findings` identity set (`[rule, node_path]`, node path `<viewport>/<mode>/<what>`, reason in `fields`). The four counts are `exact`; layout shift is `sampled` with `samples` (the viewports read) and the per-viewport values in `details`, so the document has `run.started_at` and `run.ended_at` |

A page that is not served or does not apply a mode has every result incomplete with cause
`incompatible_target` and no value.

## Comparison with llm, 2026-11-24

llm's documentation site built in a worktree of `llm` at `58e27a4bb` (`npm --prefix website ci`,
`npm --prefix website run build`, as llm's `pages.yml`), llm's declarations (1440 x 900 and 390 x 844,
light and dark, navbar, search, theme toggle and sidebar, keyboard targets search and navbar, 40 Tabs, 250
ms). Chromium 158.0.8127.18 (Playwright revision 1254) for both: ux-browser starts `chrome
--headless=new`, llm's Playwright 1.65.0 its headless shell of the same version.

ux-browser: 27 pages, 1,234 checks, 107 s. Document: `engine/browser/examples/llm-docs.evidence.json`.

Each cell: ux-browser / llm's `ux/baseline.json` / llm's own harness (`ux/tests/l2-layout.spec.ts`)
run on the same build the same day.

| page (llm scenario) | page scroll | overflow | always visible | keyboard | layout shift after ready |
|---|---|---|---|---|---|
| `page-docs` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-concepts-accounting` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-concepts-credentials` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-concepts-neutral-boundary` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-concepts-overview` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-concepts-protocols` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0.0004 / 0.0004 / 0.0004 |
| `page-docs-concepts-routing` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-getting-started` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-guides-call-a-local-endpoint` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-guides-call-a-model-on-bedrock` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-guides-call-a-model-with-one-forced-tool` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-guides-configure-model-routes` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-guides-explain-a-route` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0.0006 / 0 |
| `page-docs-guides-interrupt-a-turn-with-new-input` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-guides-price-recorded-usage` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0.0006 / 0 |
| `page-docs-guides-resolve-a-local-secret` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-guides-run-a-local-turn` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0.0013 / 0 |
| `page-docs-guides-run-the-checks` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-guides-use-llm-from-a-synchronous-loop` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-guides-use-the-llm-command` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-reference-crates` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-reference-formats` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-status` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-status-limitations` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-docs-status-roadmap` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-home` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| `page-search` | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 | 4 / 4 / 4 | 0 / 0 / 0 |

- Against llm's baseline: 132 of 135 equal. The 3 differences are layout shift after ready on three
  guides (baseline 0.0006, 0.0006, 0.0013; ux-browser 0, no shift recorded at all). Cause: the build.
  The baseline was taken on the build of 2026-11-23 (llm `3b7046fd8`, R-01); since then the shared
  docs-system the site builds on moved (`fb8ba0d00`, docs-system 0.15.1), and the three pages' own
  sources did not change. llm's own harness on today's build reads 0 on the same three pages, so the
  values moved with the build, not with the measurement. Whether the docs-system change or run-to-run
  timing removed the shifts is not isolated; that needs the old build. Within llm's tolerance of 0.01
  all three are equal.
- Against llm's harness on the same build: 135 of 135 equal, the 0.0004 of the protocols page (at 1440 x
  900, none at 390 x 844) and the 4 keyboard misses of the search page included (llm's finding UXF-005:
  the query box takes focus on load, so the skip link is not the first Tab stop, at both viewports; both
  report skip link not first and its target not reached by it).
- Planted defects: most values are 0, so one page of the same build (`/llm/docs/status/roadmap`) got
  `.markdown p { white-space: nowrap }` and `.navbar { position: static }`, and both harnesses ran on it:
  page scroll 4 / 4, overflow 14 / 14 (the same blocks), always-visible misses 12 / 12 (navbar, search
  and theme toggle clipped at the bottom, in all four units), keyboard 0 / 0, layout shift 0 / 0. The page
  was restored afterwards.
