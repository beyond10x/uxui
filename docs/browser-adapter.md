# The browser adapter `ux-browser@1`

Status: version 1, 2026-11-24. Roadmap wave 2 (`docs/roadmap.md`, Trajectory), story
`story:browser-adapter`, designed in `docs/targets.md` (the `browser` adapter). Crate `engine/browser`,
binary `ux-browser`. Rust with no third-party crate: it reads and writes with `engine/doc` (`uxui-doc`)
and drives Chromium over a DevTools-protocol client of its own.

It does three things for any page with a URL:

1. **render**: one render record (`ux-browser-render/1`) for one URL at one viewport and mode, with
   each declared node's box, computed styles and accessibility-tree entry, and a screenshot.
2. **checks**: the five page checks that the UI-loop copies share
   (`research/2026-11-24-loop-copies-page-checks.md`), with the first consumer's (`llm`) definitions,
   over every page of a built site or of a running server, as one `ratchet-evidence/1` document per run.
3. **values**: the consumer's values file (`ux-browser-values/1`) from such a document: per page
   scenario the numbers its ratchet compares, and per page, viewport and mode the hard-gate failures
   with the open finding that waives each (`ux-browser values`, or `ux-browser checks --values`).

## How it drives the browser

| part | file | what |
|---|---|---|
| WebSocket | `src/ws.rs`, `src/base64.rs`, `src/sha1.rs` | RFC 6455 client: the opening handshake with its accept check, masked client frames, server frames of any length, fragments, ping and close |
| JSON | `src/json.rs` | a reader into `uxui_doc::yaml::Value`; writing is `uxui_doc::json` |
| protocol | `src/cdp.rs` | commands with ids, responses matched by id, events queued until asked for, flat sessions; a guard that answers every paused request: requests to the measured server with a declared method (`GET` and `HEAD` unless `request_methods` names more) go on, every other one fails (`BlockedByClient`) |
| Chromium | `src/chrome.rs` | `UXUI_CHROME`, else the newest `chromium-<n>/chrome-linux*/chrome` under the Playwright cache (`PLAYWRIGHT_BROWSERS_PATH`, else `~/.cache/ms-playwright`); started headless with its own profile, `--remote-debugging-port=0`, and the switches of Playwright's headless Chromium that change what a page measures (`--hide-scrollbars`, the hover and pointer media of a headless device, sRGB); ended with `Browser.close`, then by its PID |
| pages | `src/session.rs` | one browser context per page and viewport (as a fresh Playwright context): viewport, `prefers-color-scheme`, `prefers-reduced-motion`, focus emulation, scripts before the page's own, navigation to the `load` event, keys, the accessibility tree, screenshots |
| workers | `src/pool.rs`, `src/measure.rs` | a checks run starts one Chromium and measures up to `--jobs` page and viewport pairs at a time in it, each worker on a DevTools connection of its own; pairs are taken in order, results are put back in order, and the first browser failure stops the run |
| server | `src/serve.rs`, `src/target.rs` | a static server for a built site under its base path on a free loopback port (file, file plus `.html`, `index.html`; else `404.html` with status 404); or, with `--url`, a running server: its origin and the URL of each page and of the sitemap on it |
| probes | `src/probes/*.js` | the JavaScript the page runs: it returns geometry, and decides nothing |
| checks | `src/checks.rs`, `src/evidence.rs` | every rule and every count, in Rust, from the recorded geometry |

The test step (`task test:ux-browser`, `cargo test -p ux-browser`) starts no browser and opens no
socket: framing runs over an in-memory stream, the protocol client over a scripted transport, each
check over geometry recorded from real runs (`tests/fixtures/`, written with `--capture`), the worker
pool over plain functions, and the binary only with `--help`, `values` (which starts no browser) or an
invocation it refuses before it starts a browser. Real browser runs are measurements, run by hand on a host with Chromium.

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
  [--root .] [--class local] [--pages /llm/,/llm/search] [--capture <dir>] [--jobs 4] \
  [--values page-checks.json]
ux-browser checks --config ux/ux-browser.checks.yaml --url http://127.0.0.1:4173 \
  --project my-app --assessment page-checks --out page-checks.evidence.json
```

It serves `--build` itself, or measures a server it did not start (`--url`, below), measures every
page (the sitemap's, or a list), and writes one
`ratchet-evidence/1` document. With `--values` it then writes the values file from that document, as
`ux-browser values` does (below), in the same run. `--jobs` is the number of page and viewport pairs measured at a time
in the one Chromium of the run: by default the CPUs the process may use, at most 4; at least 1. The
values do not depend on it (Timings). `--capture` writes each page's recorded geometry as
`<page>.capture.json`, the input `checks::page_result` reads. Exit statuses as `ux-static`
(`docs/ux-static.md`): 0 written; 1 refused inputs (a configuration or build that does not read, a sitemap the server does not answer); 2
wrong invocation; 3 infrastructure (Chromium did not start or answer, the document was not written). A
page the server does not answer with 200 is no failure of the run: its results are incomplete.

With `--values`: the configuration must declare `values` (Configuration), and the scenario files and
finding records are read before the browser starts; a missing section or a file that does not read
exits 1 and starts no browser. An existing values file is removed first. After the document is
written, the values file is written; the exit status is then that of `ux-browser values`: 1 when the
values report a problem (an incomplete result, an unmapped page, a failure no open finding waives),
with both files written; 3 when the values file could not be written.

### A running server: `--url`

`--url` names a server the run did not start (a dev server, a preview, a deployed origin) in place of
`--build`. Exactly one of the two is given: both, or neither, is a wrong invocation (exit 2).

| part | rule |
|---|---|
| form | `http://` or `https://`, a host (a name, an IPv4 address, or an IPv6 address in `[]`), an optional port, an optional path; no user, query or fragment. Anything else is a wrong invocation (exit 2) |
| origin | scheme, host and port, in lower case, without the default port (80, 443): `HTTP://Example.ORG:80/` is `http://example.org`. The guard compares request URLs with it |
| path | where the configuration's `base_path` lies on that server. No path, or `/`: the server serves the site where the configuration says, and a page is its configured path on the origin (`/llm/search` at `http://127.0.0.1:4173/llm/search`). Any other path (`/preview` reads as `/preview/`): a page's part after `base_path` goes under it (`/llm/search` at `http://127.0.0.1:4173/preview/search`), and a configured page outside `base_path` is refused (exit 1). Page paths in the document stay the configured ones, so a document of a server and one of the build have the same items |
| pages | `pages.paths` as with `--build`; `pages.sitemap` is the file under the base path on that server (`sitemap.xml` with `base_path: /llm/` is `<origin>/llm/sitemap.xml`, moved by a path as a page is), read in the run's Chromium behind the same guard; its `<loc>` paths are measured as above. A sitemap the server does not answer with 200 is a refused input (exit 1, no document) |
| guard | requests to the origin go on when their method is in `request_methods` (default `GET` and `HEAD`; Configuration); requests to any other origin, and other methods to the origin, fail (`BlockedByClient`), as with `--build`. A measurement does not change the state of a server it did not start; a page that must send a `POST` to show itself names it in `request_methods`, and the configuration's digest in the document records that |
| status | a page the server does not answer with 200 is incomplete, as with `--build` |
| evidence | `target: {kind: deployment, class, identifiers: {environment_id: <the URL, normalized: origin and path>}}`, no `selection` and no `candidate` (Evidence) |

The values do not depend on where the same bytes are served from. Measured 2026-11-25, Chrome/158.0.8127.18,
the same directory served once by `--build` and once by a separate process on a loopback port (the same
static server code) for `--url`:

| site | pages x viewports | values | equal | wall time `--url` / `--build` |
|---|---|---|---|---|
| llm's documentation build (`examples/llm.checks.yaml`, sitemap read from the server) | 27 x 2 | 135 | 135, every result byte for byte | 40.9 s / 45.1 s |
| the second consumer's snapshot (a control plane web application, `pages.paths`) | 11 x 5 | 55 | the 44 exact values, byte for byte with their details and findings | 32.5 s / 22.1 s; again 49.3 s / 50.9 s |

The second consumer's 11 `layout_shift.after_ready@1` values (sampled) vary from run to run on its page
under either flag: two `--build` runs differed in 9 of them, a `--url` and a `--build` run in 9. The
host's load moved the wall times more than the flag did.

### Configuration `ux-browser-checks/1`

Viewports, modes, settle times and the declared items are inputs, never constants of the tool. llm's,
from `llm/ux/adapter.ts`: `engine/browser/examples/llm.checks.yaml`.

| key | value |
|---|---|
| `base_path` | where the build is served, `/llm/`; with `--url`, where the pages are on the server unless its path says otherwise (A running server) |
| `pages` | `{sitemap: sitemap.xml}` (a file of the build; with `--url`, the file under the base path on the server) or `{paths: [...]}` |
| `viewports` | `{name, width, height}` each |
| `modes` | `light`, `dark`, in order; the page loads in the first, the others are switched in place |
| `mode_attribute` | the attribute of `<html>` that shows the applied mode (`data-theme`); a page that does not apply a mode within 5 s is incomplete |
| `reduced_motion` | emulate `prefers-reduced-motion: reduce`; default true |
| `ready` | the condition the page sets once its content is drawn, for a page that draws after its `load` event (from fetched data, say): `{selector: <css>}`, ready when an element matches it (`{selector: 'html[data-ready]'}` for an attribute the page sets, `{selector: 'main:has(.panel):not(:has([aria-busy=true]))'}` for every panel loaded), or `{global: <name>}`, ready when the page sets `window.<name>` to `true` (`window.name = true` or a top-level `var`; a `let` or `const` is no property of `window`). Default: none, ready at `load` as before (The checks) |
| `ready_timeout_ms` | how long after `load` the page may take to set `ready`; default 10000; only with `ready`. A page that does not set it in time is incomplete |
| `settle_ms`, `scroll_settle_ms`, `mode_settle_ms` | after ready (250), after the scroll to the bottom (200), after a mode switch (100) |
| `always_visible` | `{name, role, name_is \| name_contains, css, scope, current, min_width, path_prefix}` each; located by at least one of `role`, `css` and `current: true` (below) |
| `keyboard` | `{skip_link_is \| skip_link_contains, must_reach, max_tabs, modes, viewports, focus_settle_ms}`; `modes` defaults to the last mode, `viewports` (names) to every viewport; each target is an always-visible item |
| `opt_in` | checks that run only when named here: `layout_shift.load` (The checks); default none |
| `request_methods` | the HTTP methods a page may send to the measured server, in capitals, each once, `GET` among them; default `[GET, HEAD]`. A request with another method fails as one to another origin does (A running server) |
| `values` | how `ux-browser values` and `checks --values` turn the document into the consumer's values file (`ux-browser values`); optional |

The `values` section (every path is under `--root`):

| key | value |
|---|---|
| `scenarios.files` | the page scenario files, a path pattern (below): one JSON object per file |
| `scenarios.path_key` | the key of that object that holds the page path; default `path`. The scenario's name is the name of the directory that holds the file |
| `waivers.files` | the finding records, a path pattern: each a file whose front matter (from line 2 to the next line `---`) has the lines `id: <id>`, `status: <status>` and, to waive, `waives: <pattern>, <pattern>, ...` |
| `waivers.closed` | the statuses of a record that waives nothing; default `[closed]` |
| `gates` | per output that has findings (`layout.page_scroll@1`, `layout.overflow@1`, `always_visible.misses@1`, `keyboard.misses@1`) the number of the hard gate its failures count against; a finding of an output not named here is no gate failure |
| `gating` | the viewports (names) whose results are values and gate failures; default every viewport. When it is declared, the values file also has a per-viewport breakdown (`viewports`), and the document must carry per-viewport details (written since 0.3.0) |

A path pattern is a path relative to `--root` in which `*` stands for any run of characters inside one
path segment, never a leading `.`; the files it matches are read in the byte order of their paths.
A waiver pattern is matched against a whole failure key; there `*` stands for any run of characters,
`|` included, and every other character is itself.

Names: `name_is` is the whole accessible name (white space folded, case kept), as Playwright's
`getByRole` with a string name; `name_contains` is a list of parts matched in any case. llm's regular
expressions translate to it: `/search/i` is `[search]`, `/dark and light mode/i` is `[dark and light
mode]`, `/skip to (main )?content/i` is `[skip to content, skip to main content]`.

An always-visible item is located by its candidates, in this order: the elements of `role` whose
accessible name matches (from the accessibility tree), then the elements `css` matches inside the
first match of `scope`. The item is the first candidate that is in `scope` and visible:

- `role` (and `name_is` or `name_contains`), with `css` as a fallback: as in 0.3.0.
- `css` alone: the first visible match in `scope`; no accessibility tree is read and no name is
  matched, so `name_is` and `name_contains` need a `role`.
- `current: true`: only a candidate whose `aria-current` is not `false` (an empty value counts as
  `false`; any other value, `page`, `step`, `true` and the rest, as current). With `role` or `css`
  it narrows their candidates; without both its candidates are the elements with `aria-current` in
  `scope`. One item `{name: title, current: true, scope: 'nav[aria-label=screens]'}` is the current
  entry of a navigation on every page, where one item per page with its own name and `path_prefix`
  was needed before. A page that has no such element counts the item `missing`.

`current` is `true` or `false` (default `false`). An item with neither `role` nor `css` nor `current:
true` is refused, as is a name without a `role`. A keyboard target (`must_reach`) is located the
same way.

### The checks

Per page and viewport, as llm's L2 spec does it: a fresh context in the first mode, the page loaded to
its `load` event, ready (below), the settle time, the layout shifts read; then per mode: scroll to the
top, overflow and page scroll, the always-visible items, scroll to the bottom and the scroll settle
time, the items again; the keyboard walk in the keyboard modes. A later mode is switched in place
(`Emulation.setEmulatedMedia` on the loaded page, no reload); the run waits for the page to show it in
`mode_attribute` (watched in the page, no polling), then the mode settle time. The only fixed waits
are the declared settle times.

Ready is fonts loaded and two frames drawn after `load`. With `ready` declared it is the page's
condition first, then fonts loaded and two frames drawn: a selector is watched with a
`MutationObserver` on the document, a global through a property trap set on `window` before the
page's own scripts, both for at most `ready_timeout_ms` after `load` (no polling, in the page or in
the run). The ready time is the instant after the two frames, not the instant the condition held: the
content a page draws in the task that sets its condition is laid out in the frame after, so counting
from the condition would count that content's own shift as after ready. With `ready`, each fresh load
of the keyboard walk waits for the same ready before its first Tab. A page that does not set its
condition in time, or whose selector does not parse, has every result incomplete (Evidence), as a
page that is not served.

| result (`scope: page`) | per page | definition (llm's) |
|---|---|---|
| `layout.page_scroll@1` (units) | sum over viewport x mode | 1 when `scrollingElement.scrollWidth - documentElement.clientWidth > 0` |
| `layout.overflow@1` (blocks) | sum | every non-blank text run (Range box) and every `img, svg, video, canvas, iframe, input, select, textarea, table, pre`: overflow when its right edge passes the viewport + 1 px or its nearest non-inline block + 1 px; none under `position: fixed`, `overflow-x: auto \| scroll` or a clipping box; at most one per block |
| `always_visible.misses@1` (items) | sum over viewport x mode x top and bottom | the first visible candidate in scope (role and name from the accessibility tree first, then CSS; with `current`, only a current one; Configuration): missing, clipped (not wholly inside the viewport +-0.5 px) or covered (`elementFromPoint` of its centre is not it or inside it); an item outside its `min_width` or `path_prefix` is not checked |
| `keyboard.misses@1` (checks) | sum over the walks | on a fresh load the first Tab is on the skip link; Enter (or one more Tab) puts focus into its target; on another fresh load up to `max_tabs` Tabs reach each target. A skip link that is not first misses both of its checks |
| `layout_shift.after_ready@1` (score) | largest over the viewports | `layout-shift` entries without recent input after the ready time (above), summed, rounded to 4 places; read once per viewport, before the first mode switch |
| `layout_shift.load@1` (score), opt-in | largest over the viewports | every `layout-shift` entry without recent input since the navigation started (the observer is buffered and set before the page's own scripts), summed, rounded to 4 places; read in the same reading as after ready, after the settle time, so it holds the shifts after ready too. llm's round mode (`readShift().load` of its `ux/core/layout.ts` before 0.3.0): it varies with how fast fonts arrive, so llm reports it and does not ratchet it |

`keyboard.viewports` limits the walk to some viewports, as llm's round walks only its gating ones.

Not covered: axe-core's rules (a third-party library; llm runs `@axe-core/playwright` 4.14.0), reading
load and the screen-only user (llm's other checks), the scroll containers llm's round reports, and the
screenshots of a round. Playwright blocks service workers; this run does not (a Docusaurus build
registers none).

A round of llm's loop runs a second configuration of the same checks,
`engine/browser/examples/llm-round.checks.yaml`: the gating viewports 1440 x 900 and 390 x 844 and the
measured ones 1280 x 800, 1024 x 768 and 768 x 1024 (the docs-site sizes of the skill's adapter
table), settle 750 ms, `layout_shift.load` on, the keyboard walk at the gating viewports only, and
`values.gating` so that the measured viewports are reported per viewport and gate nothing.

### Evidence

As `ux-static@1` writes it (`docs/ux-static.md`), with these differences:

| field | value |
|---|---|
| `evidence.assessor` | `ux-browser@1` |
| `evidence.target` | with `--build`, `kind: build` (ratchet `docs/evidence-format.md`, `evidence.target.kind`): the served build, `identifiers: {artifact_digest: <build digest>}`, `selection: candidate`, no `revision`. With `--url`, `kind: deployment`: the running server, `identifiers: {environment_id: <the --url, normalized>}`, no `selection` and no `revision`; the URL is not repeated under `inputs`, as ratchet's format says of a document target |
| `evidence.candidate` | the build digest, as ratchet's format asks of a `build` ("the artifact digest"): see below. Absent with `--url`: what a server sends has no digest the run could compute before it measures, so the document is attributable to no candidate (ratchet's rule: `candidate` exactly when `selection`) |
| `evidence.inputs.config` | the configuration file, `{value: <path>, digest, digest_form: bytes}` |
| `evidence.inputs.build` | `{value: <path>}` for a build under `--root`, absent for one outside it; no digest beside it, the candidate is its digest |
| `evidence.inputs.pages` | `{value: <the --pages list>}` when the run measured a list and not the whole sitemap |
| `evidence.conditions` | `tools: {ux-browser, chromium}`, `other: {viewports, modes, ready, settle_ms, reduced_motion}`; `ready` is `load` without a ready condition, else `selector <css>` or `global <name>`. A document of 0.3.0 has no `ready`; its ready was `load` |
| `run.local_inputs` | `jobs` (the value used), and `capture` and `profile` when given: they differ per machine and change no value, so they stay outside the evidence |
| `evidence.coverage` | unit `check`: one check on one viewport and mode (page scroll, overflow, each applicable item at the top and at the bottom), one keyboard check per walk (skip link first, its target, each target), and one counted layout-shift reading per viewport. A page that did not load is one `not_run` entry, id the page path |
| `evidence.results[]` | per page the five numbers (six with `layout_shift.load`, after the shift after ready), then per page a `findings` identity set (`[rule, node_path]`, node path `<viewport>/<mode>/<what>`, reason in `fields`). The four counts are `exact`; layout shift is `sampled` with `samples` (the viewports read), so the document has `run.started_at` and `run.ended_at`. Every number has its per-viewport values in `details.viewports` (`{<viewport>: "<value>"}`: a count summed over the viewport's modes or walks, a shift as read); the keyboard count keeps its `details.walks` |

A page that is not served, does not apply a mode or does not set its ready condition in time has
every result incomplete with cause `incompatible_target` and no value; the detail says why, and names
the condition: ``ready: the page did not set `selector html[data-ready]` within 10000 ms``.

**The build digest** (`evidence::build_digest`), the candidate of every run: SHA-256 over the text
that has one line `<hex>  <path>\n` per file of the served directory, where `<hex>` is the SHA-256 of
the file's bytes in lower-case hex and `<path>` is the file's path relative to that directory with `/`;
the lines sorted by the bytes of the path; written `sha256:<hex>`. Directories add nothing, a symbolic
link counts as what it points to. The same value from a shell, in the served directory:

```sh
find . -type f | sed 's|^\./||' | LC_ALL=C sort | xargs sha256sum | sha256sum
```

The digest is of the bytes served, so two builds of one commit that differ in a byte are two
candidates: the two builds of llm `58e27a4bb` measured here, PR #11's and this change's, have two
digests and the same 1,234 values.

### Declaration `ratchet-assessor/1`

`engine/browser/ux-browser.assessor.yaml`, the declaration of ux-browser@1 (ratchet
`docs/evidence-format.md` § 10), modelled on `engine/static/ux-static.assessor.yaml`:

| field | value |
|---|---|
| `command` | `ux-browser checks` |
| `targets` | `[build, deployment]`: a served build (`--build`) or a running server (`--url`); no `target_input`, which only a `document` target has |
| `inputs` | `config` (path, required), `build` (path) and `url` (text), of which a run gets exactly one (a choice of the usage; the format has no way to say so), `pages` (text: paths separated by commas), and the local inputs `capture`, `profile`, `values` (paths) and `jobs` (integer); `values` names a file the run writes and changes no value of the document, as `capture` |
| `conditions` | `viewports`, `modes`, `ready`, `settle_ms`, `reduced_motion`: the keys of `conditions.other` |
| `coverage_unit`, `coverage` | `check`; all six counts |
| `outputs` | the six numbers (`scope: page`; unit `units`, `blocks`, `items`, `checks`, `score`, `score`; the two layout shifts `sampled`, the others `exact`) and `findings` (`identity_key: [rule, node_path]`, `item_fields: [reason]`) |

A test (`tests/assessor.rs`) reads it and compares its inputs with the arguments of the `checks` usage
(required where the usage neither brackets them nor puts them in a choice `( | )`), its conditions with a document's, and its outputs with
every result and the target kind of the example document and of the test documents (a build's and a running server's).

## `ux-browser values`

```sh
ux-browser values --config ux/ux-browser.checks.yaml --evidence page-checks.evidence.json \
  --out page-checks.json [--root .]
```

It reads a `ratchet-evidence/1` document of `ux-browser checks` and the configuration's `values`
section, and writes the values file `ux-browser-values/1`: what a consumer's ratchet compares and what
its hard gates fail on, with no conversion of the consumer's own. It replaces llm's
`ux/page-checks.sh` (POSIX sh and jq) and writes the bytes that script wrote from the same document.

Exit statuses: 0 written, nothing to report; 1 written, and the values report a problem (below) — or
the inputs are refused (a configuration without `values`, a document, scenario file or finding record
that does not read) and nothing is written; 2 wrong invocation; 3 the file could not be written. The
`--out` file is removed before anything is read, so a run that ends with 1 and leaves no file was
refused. Every waived failure and every problem is one line on standard error.

The problems, each one line: an incomplete result (`incomplete: <page> <output>: <cause>`), a page with
no scenario (`no page scenario for <page>`), and a failure no open finding waives (`hard gate failure
not waived by an open finding: <key>`).

The file: JSON with the keys of every object sorted by their bytes, two spaces of indent, a number
written as the document's decimal text, a newline at the end (as `jq -S` writes it). It has no `format`
key, so that it is the bytes the script it replaces wrote; `ux-browser-values/1` names its shape here.

| key | value |
|---|---|
| `values` | per page scenario (`scenarios`) an object: per number of the document its output without the `@<version>` suffix (`layout.page_scroll`) and its value. A number without a value (incomplete) is left out; a page without a scenario has none. Without `gating`, the document's value: a sum over viewport x mode, a layout shift the largest over the viewports. With `gating`, the same sum or largest over the gating viewports only, from `details.viewports` |
| `units` | per page, viewport and mode that has a gate failure, `{path, viewport, theme, failures}`; `theme` is the mode. Sorted by path, viewport, mode |
| `units[].failures` | `{gate, detail, key, waivedBy}` each, one per key, sorted by key. `waivedBy` is the `id` of the first finding record (in the order of `waivers.files`) whose status is not closed and one of whose `waives` patterns matches the key; else `null` |
| `incomplete` | `<page> <output>: <cause>` per result that is incomplete, in the document's order |
| `unmapped` | the pages of the document without a page scenario, sorted, each once |
| `viewports` | only with `gating`: per viewport `{gating, pages, <output>: <value>, ...}`: `pages` the number of pages with values at it, each count summed over them, each layout shift the largest |

A failure is a finding of an output named in `gates`, at a gating viewport. Its node path
`<viewport>/<mode>/<what>` gives the unit, and `<what>` its detail:

| output | detail |
|---|---|
| `layout.page_scroll@1` | `page-scroll` |
| `layout.overflow@1` | `overflow:<block>` |
| `always_visible.misses@1` | `<item>:<reason>@<top \| bottom>`; `<item>:missing` for a missing item |
| `keyboard.misses@1` | `keyboard:<check>`; `skip-link-moves-focus` is left out in a unit where `skip-link-first` failed, as it follows from it |

The key: `gate<N>|<page>|<viewport>|<mode>|<detail>`, with every `|` and `,` of the detail a space,
so that a key splits on `|` and a `waives` line on `,`. llm's `core/findings.ts` writes the same form.

**Acceptance** (`tests/values.rs`): `tests/fixtures/values/` holds llm's fixture document of 0.2.0's
patch (`page-checks.evidence.json`: the search page with the 4 keyboard misses UXF-005 waives, a planted
broken roadmap page, a page that was not served), the page scenarios and the front matter of llm's
finding records it needs, and `page-checks.json`, the file `ux/page-checks.sh --evidence` wrote from
it. `ux-browser values` writes the same bytes and exits 1, as the script did.

## Timings

`ux-browser checks` over llm's documentation site (27 pages, 2 viewports, 2 modes, 1,234 checks),
built at llm `58e27a4bb`, Chromium 158.0.8127.18, on a shared 20-CPU workstation with other jobs
running, 2026-11-24. Each run started as a transient user service so the CPU time of every Chromium
process is counted. Before: PR #11 (`4171e3a`), one page and viewport after the other; after: this
version.

Two rounds, each the four runs one after another; the host's 1-minute load at the start of each run
beside it. PR #11 has no `--jobs`: it measures one pair at a time.

| round | binary | `--jobs` | load | wall | CPU | values against PR #11's document |
|---|---|---|---|---|---|---|
| 1 | PR #11 | none | 20.3 | 91.7 s | 83.3 s | equal |
| 1 | this | 1 | 8.9 | 88.0 s | 80.3 s | equal |
| 1 | this | 2 | 5.0 | 47.4 s | 86.5 s | equal |
| 1 | this | 4 | 4.3 | 27.2 s | 92.4 s | equal |
| 2 | PR #11 | none | 4.1 | 96.4 s | 91.0 s | equal |
| 2 | this | 1 | 6.4 | 90.1 s | 85.1 s | equal |
| 2 | this | 2 | 5.6 | 64.0 s | 122.8 s | equal |
| 2 | this | 4 | 17.4 | 42.2 s | 139.8 s | equal |

The example document of that version was written by a run with the default (4 on this host) at a load
of 14.8: 31.7 s.
"Equal": all 162 results (numbers, findings, the per-viewport layout shifts) and the coverage counts of
the 1,234 checks are the same. The time moves with the host's load more than with anything else; at a
low load four workers take about 30 % of the wall time of one, for about 15 % more CPU time.

What the time goes to, per page and viewport: three loads (the measured one and the two fresh loads of
the keyboard walk), the accessibility tree five times, and the declared waits (250 + 200 + 200 + 100 +
100 ms). The declared waits alone are 46 s of a one-worker run; workers overlap them.

Tried and left out: one page reused per worker and viewport, reset to the first mode and with the
origin's storage cleared between paths (the server sends every file with `no-store`). One worker took
93.2 s, a fresh context per pair 93.3 s, and the reused page read the layout shift of
`/llm/docs/concepts/protocols` at 1440 x 900 as 0 where every fresh-context run (PR #11, the rebuild
before this change, and each run above) read 0.0004. Not isolated; the shift is likely the swap of a web
font that a reused renderer already holds. So each pair keeps a context of its own, as a fresh
Playwright context does.

## Comparison with llm, 2026-11-24

llm's documentation site built in a worktree of `llm` at `58e27a4bb` (`npm --prefix website ci`,
`npm --prefix website run build`, as llm's `pages.yml`), llm's declarations (1440 x 900 and 390 x 844,
light and dark, navbar, search, theme toggle and sidebar, keyboard targets search and navbar, 40 Tabs, 250
ms). Chromium 158.0.8127.18 (Playwright revision 1254) for both: ux-browser starts `chrome
--headless=new`, llm's Playwright 1.65.0 its headless shell of the same version.

ux-browser: 27 pages, 1,234 checks, 107 s at PR #11 (one pair at a time; now: Timings). Document:
`engine/browser/examples/llm-docs.evidence.json`, rewritten by this version's run on a rebuild of the
same commit: the same 1,234 values, the new target kind, another build digest. Rewritten again on
2026-11-25, when documents began to name their ready condition (`ready: load`), by a run on a build of
llm `6df5102cc`: the same 135 numbers and findings and the same 1,234 checks; per-viewport details
on every number, as written since 0.3.0.

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

## A page that draws from fetched data, 2026-11-25

The second consumer, a control plane web application: a static snapshot of 11 pages, each drawing its
panels and a header widget from JSON it fetches, after its `load` event; its configuration (5
viewports, dark, settle 500 ms), served by `checks --build`. Five runs one after another per row,
Chromium 158.0.8127.18.

| site | ready | pages whose `layout_shift.after_ready` differed between the 5 runs | largest value |
|---|---|---|---|
| the snapshot | `load` (none declared) | 10 of 11 | 0.8892 |
| the snapshot | a selector that matches once no panel is loading | 7 of 11 | 0.1781 |
| a copy whose header widget also marks itself loading (one line) | `load` | 11 of 11 | 0.8871 |
| the copy | the selector, covering the widget too | 0 of 11: every value 0; the 5 documents' values, findings and coverage equal | 0 |

What moved after ready, read from the shift entries' sources (further runs: two of the first row, three
of the second): without `ready`, mostly the header widget, whose text arrives late and moves the page
below it by 21 px at three viewports, and in some runs a panel as the data above it arrived; with the
panels' signal, the header widget alone. A ready condition is as good as what the page signals: a part
that draws late and is not in the condition still counts after ready. Every other value (451 checks)
was the same in all 20 runs.

## A per-page item declared once, 2026-11-25

The same snapshot and configuration. It declared the current screen's title as 11 items, one per page,
each a `link` with the page's name under `path_prefix`. They were replaced by one item, run by run:

| title item(s) | `always_visible.misses@1` per page, every viewport | its findings | checks (executed / passed / failed) | located on |
|---|---|---|---|---|
| the 11 items (as before) | 0 on all 11 pages | 0 | 451 / 352 / 44 | 11 of 11 pages, 10 of 10 viewport and position pairs each |
| `{name: title, current: true, scope: 'nav[aria-label=screens]'}` | 0 on all 11 pages | 0 | 451 / 352 / 44 | the same, each time the link with `aria-current="page"` |
| `{name: title, css: 'a[aria-current=page]', scope: 'nav[aria-label=screens]'}` | 0 on all 11 pages | 0 | 451 / 352 / 44 | the same |
| control: `{name: title, current: true, scope: main}` | 10 per page (2 per viewport) | 110, `missing` | 451 / 242 / 154 | none |

The values file's `always_visible.misses` was 0 for all 11 scenarios and every viewport in the first
three rows, as in the consumer's 0.3.0 run; the control row is 4 per scenario at the two gating
viewports and fails 22 `gate2` keys. Between the first three rows every result and finding was the same
except `layout_shift.after_ready` (no ready condition declared; above). llm's documentation build with
`engine/browser/examples/llm.checks.yaml`, in the same build of this change: the 135 numbers, their
per-viewport details, the 27 findings sets and the 1,234 checks equal to
`engine/browser/examples/llm-docs.evidence.json`.
