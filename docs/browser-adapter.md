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
| Chromium | `src/chrome.rs` | `UXUI_CHROME`, else the newest `chromium-<n>/chrome-linux*/chrome` under the Playwright cache (`PLAYWRIGHT_BROWSERS_PATH`, else `~/.cache/ms-playwright`), or the build a checks file pins (The Chrome build); started headless with its own profile, `--remote-debugging-port=0`, the switches of Playwright's headless Chromium that change what a page measures (`--hide-scrollbars`, the hover and pointer media of a headless device, sRGB), no GPU process (`--disable-gpu`, `--in-process-gpu`), then a consumer's own switches (Chromium switches); ended with `Browser.close`, then by its PID |
| pages | `src/session.rs` | one browser context per page and viewport (as a fresh Playwright context): viewport, `prefers-color-scheme`, `prefers-reduced-motion`, focus emulation, scripts before the page's own, navigation to the `load` event, keys, the accessibility tree, screenshots |
| workers | `src/pool.rs`, `src/measure.rs` | a checks run starts one Chromium and measures up to `--jobs` page and viewport pairs at a time in it, each worker on a DevTools connection of its own; pairs are taken in order, results are put back in order, and the first browser failure stops the run |
| server | `src/serve.rs`, `src/target.rs` | a static server for a built site under its base path on a free loopback port (file, file plus `.html`, `index.html`; else `404.html` with status 404); or, with `--url`, a running server: its origin and the URL of each page and of the sitemap on it |
| probes | `src/probes/*.js` | the JavaScript the page runs: it returns geometry, and decides nothing |
| checks | `src/checks.rs`, `src/evidence.rs` | every rule and every count, in Rust, from the recorded geometry |

The test step (`task test:ux-browser`, `cargo test -p ux-browser`) starts no browser and opens no
socket: framing runs over an in-memory stream, the protocol client over a scripted transport, each
check over geometry recorded from real runs (`tests/fixtures/`, written with `--capture`), the worker
pool over plain functions, the start of Chromium over a shell script that stands in for it (it records
its arguments), and the binary only with `--help`, `--version`, `values` (which starts no browser) or an
invocation it refuses before it starts a browser. Real browser runs are measurements, run by hand on a host with Chromium.

## `ux-browser --version`

`ux-browser --version` (or `-V`) prints `ux-browser <version>` and a newline on stdout, the crate
version, and exits 0; it starts no browser and reads no file. It is the version a run records as
`evidence.conditions.tools.ux-browser`, so a consumer that pins ux-browser starts a stored or `PATH`
binary only when this answer equals the pin. `ux-browser --help` (or `-h`) prints the usage of every
command. The release checks the answer of the binaries it packages against the tag
(`sh tools/release-versions.sh <version> <directory of the built binaries>`).

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
  [--values page-checks.json] [--chrome-arg <switch>]... [--visual compare|update]
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
wrong invocation; 3 infrastructure (the pinned Chrome build is not on the host, Chromium did not start or answer, the document was not written). A
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

Viewports, modes, settle times, the declared items and the layout rule are inputs, never constants of the tool. llm's,
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
| `always_visible` | `{name, role, name_is \| name_contains, css, scope, current, min_width, max_width, path_prefix}` each; located by at least one of `role`, `css` and `current: true` (below); checked at the viewports with `min_width <= width <= max_width` (each bound optional) and on the pages under `path_prefix` |
| `keyboard` | `{skip_link_is \| skip_link_contains, must_reach, max_tabs, focus_mark, activates, modes, viewports, focus_settle_ms}`; the skip link is optional (without it the walk has no skip-link checks); `focus_mark` is `true` or `false` (default); `activates` lists targets of `must_reach`, each once (default none); `modes` defaults to the last mode, `viewports` (names) to every viewport; each target is an always-visible item (The keyboard walk) |
| `opt_in` | checks that run only when named here: `layout_shift.load` (The checks); default none |
| `request_methods` | the HTTP methods a page may send to the measured server, in capitals, each once, `GET` among them; default `[GET, HEAD]`. A request with another method fails as one to another origin does (A running server) |
| `layout` | the page-scroll slack and the overflow rule (below); optional, each key left out keeps its default |
| `values` | how `ux-browser values` and `checks --values` turn the document into the consumer's values file (`ux-browser values`); optional |
| `visual` | the visual ratchet: where the baseline, the shots and the diff images lie, whether the full page is shot, the mask selectors and the tolerances; per page, viewport and mode a screenshot compared with its baseline (Visual ratchet). Default: none, no screenshot |
| `clock` | the page's clock: `{at: <RFC 3339 instant>, advance: true \| false}`; every load starts its `Date` at `at`, running on from there (`advance`, default true) or standing still. Default: none, the host's clock (A fixed page clock) |
| `chrome_args` | switches Chromium starts with after the defaults, a list: `[--enable-gpu]`; a switch the defaults set with another value is refused. Default: none (Chromium switches) |
| `chrome` | the Chrome build the run uses: `{build: <n>}`, the Playwright build `chromium-<n>`, or `{version: <major>}`, the newest build whose Chrome has that major version; exactly one key, a whole number above 0. Default: none, the newest build (The Chrome build) |
| `journeys` | steps named by CSS selector that bring a page into a state, and the states at which page scroll, overflow and the always-visible items are checked: `{name, page, steps, timeout_ms}` each. Default: none (Journeys) |
| `tokens` | the design tokens a page is held to: font sizes, colours, the spacing unit and radii; per page, viewport and mode the values it uses off them and the lowest text contrast. Default: none, no token check runs (Design tokens) |

The `layout` section; the defaults are llm's rule, the one of 0.4.0 and before (The checks):

| key | value |
|---|---|
| `page_slack_px` | how far, in CSS pixels, the page may scroll sideways before a unit counts for `layout.page_scroll`; a number, 0 or more; default 0. A rule "scroll width above client width + 1" is `1` |
| `overflow.tolerance_px` | how far, in CSS pixels, a right edge may pass the viewport or its block before it overflows; a number, 0 or more; default 1 |
| `overflow.elements` | the CSS selectors of the elements measured besides the text runs; a list, none empty, each once; default `[img, svg, video, canvas, iframe, input, select, textarea, table, pre]` |
| `overflow.exclude` | CSS selectors: a text run or element inside a match of one (or the match itself) is not overflow; a list, none empty, each once; default none |

The elements and the exclusions are inputs of the in-page probe (`src/probes/overflow.js`): it
measures the elements they name and reports, per item, the first exclusion it lies inside; it returns
them with the geometry, and `checks::page_result` refuses a recording made with other ones (a capture
of 0.4.0 or before reads as the defaults). The slack, the tolerance, the exclusion and the count are applied in
Rust (`checks.rs`). A selector the browser does not parse makes the probe throw, and the run exits 3.
With a slack above 0 a page-scroll finding names it: `the page scrolls 3 px sideways, past the slack
of 1 px`. The document's conditions name every value used (Evidence).

Measured 2026-11-25, Chrome/158.0.8127.18:

| site | layout keys | units | page scroll / overflow | against |
|---|---|---|---|---|
| llm's documentation build (`examples/llm.checks.yaml`) | none | 27 pages x 2 viewports x 2 modes | 0 / 0 | the run of the probe before this change on the same build: its 135 values and its results, findings and details byte for byte; its values file byte for byte |
| the second consumer's snapshot (a control plane web application, its configuration) | none | 11 pages x 5 viewports | 0 / 0 | its own gate: 0 / 0 on its 22 gating units |
| the same | `page_slack_px: 1` (its rule: scroll width above client width + 1) | 11 x 5 | 0 / 0 | every unit's raw page scroll is 0 px, so the slack changes nothing there |
| the same | `page_slack_px: 1`, `overflow.elements: ['body *']` (every element, as its rule takes) | 11 x 5 | 0 / 0 | 16,890 items measured where the default elements give 7,070 |

The consumer's own overflow rule is not the same rule with other values: it takes any element
whose content is wider than its box (`overflow-x: visible`) or that ends past the page edge, the
innermost only; this rule takes text runs and elements past their block or the viewport. Both give
0 on its pages.

```yaml
layout:
  page_slack_px: 1
  overflow:
    tolerance_px: 1
    elements: [img, svg, video, canvas, iframe, input, select, textarea, table, pre, span]
    exclude: ['.code-sample']
```

The `values` section (every path is under `--root`):

| key | value |
|---|---|
| `scenarios.files` | the page scenario files, a path pattern (below): one JSON object per file |
| `scenarios.path_key` | the key of that object that holds the page path; default `path`. The scenario's name is the name of the directory that holds the file |
| `waivers.files` | the finding records, a path pattern: each a file whose front matter (from line 2 to the next line `---`) has the lines `id: <id>`, `status: <status>` and, to waive, `waives: <pattern>, <pattern>, ...` |
| `waivers.closed` | the statuses of a record that waives nothing; default `[closed]` |
| `gates` | per output that has findings (`layout.page_scroll@1`, `layout.overflow@1`, `always_visible.misses@1`, `keyboard.misses@1`) the number of the hard gate its failures count against; a finding of an output not named here is no gate failure. Per visual output (`visual.diff_share@1`, `visual.diff_px@1`, Visual ratchet) `{gate: <n>, max: <bound>}`: the gate, and the bound each unit's number is held to, a share from 0 to 1 or a whole number of pixels of at least 0 |
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

`max_width` is the widest viewport, in CSS pixels, an item is checked at: an item a page shows only
on narrow viewports (a phone menu button under a media query) is declared with it, and at a wider
viewport it is not measured and counts nothing (`n/a`), as below its `min_width`. Both bounds hold
their own width: `{min_width: 1024}` and `{max_width: 1023}` split the viewports between a desktop and a
phone control. A `max_width` that is not a whole number above 0, or that is below the item's
`min_width`, is refused with the item's name. Without it an item is checked as in 0.6.0. The bounds
and `path_prefix` choose where the always-visible check runs; the keyboard walk reaches for its
targets at every viewport of `keyboard.viewports`, whatever their bounds.

```yaml
always_visible:
  - {name: open menu, role: button, name_is: Open menu, max_width: 1023}
  - {name: navigation toggle, role: button, name_is: Toggle navigation, min_width: 1024}
```

### A fixed page clock: `clock`

A page that shows times (a timestamp, "3 min ago", the seconds of a clock) draws other text at every
run, and other text has other geometry. `clock` starts the clock of every load at one declared
instant, so such a page measures the same on every run:

```yaml
clock: {at: 2026-01-01T12:00:20Z}
```

| key | value |
|---|---|
| `at` | an RFC 3339 instant (`date-time`): `2026-01-01T12:00:20Z`, `2026-01-01T13:00:20.25+01:00`; `T` and `Z` in either case. Refused: no zone, a leap second (`:60`, which `Date` does not have), more than 3 digits of fraction (finer than `Date` holds), a date that is not in the calendar, a number |
| `advance` | `true` (default): the clock runs on from `at` in real time; `false`: it stands at `at` |

How: a script that runs before the page's own (`src/probes/clock.js`, set as the layout-shift
observer is) replaces `Date`. `Date()`, `new Date()` and `Date.now()` read `at` plus the time since the
document's time origin (`performance.now()`, whole milliseconds), or `at` alone with `advance: false`.
`new Date(...)` with arguments, `Date.parse`, `Date.UTC`, `instanceof Date`, a subclass of `Date` and
`new Date().constructor === Date` behave as before. `performance.timeOrigin` is `at`, so `Date.now()`
and `performance.timeOrigin + performance.now()` agree. `Intl.DateTimeFormat`'s `format()` and
`formatToParts()` without a date, and `Temporal.Now` where the browser has it, read the same clock.
Every load starts at `at`: the measured load and each fresh load of the keyboard walk.

Not changed: timers (`setTimeout`, `setInterval`), frames (`requestAnimationFrame`), `performance.now()`
and the time of every performance entry. The ready wait (a `MutationObserver` or a property trap,
bounded by a timer), fonts, the two frames, the settle times (waited by the run, not by the page),
the mode wait and the layout-shift times are what they are without a clock.

Why a script and not DevTools' virtual time: `Emulation.setVirtualTimePolicy` with
`initialVirtualTime` sets `Date` too, but it moves timers, frames and the page's network waits onto a
clock that advances only by a budget or while the page is idle. The ready wait, fonts and the settle
times the run waits in real time would then depend on how that budget is spent. A script on `Date`
changes what the page reads and nothing about when it runs.

Why `advance` defaults to true: with a clock that stands still, code that compares two `Date.now()`
readings, to order events (Vue 3's DOM event handlers do) or to measure a time span, sees no time
pass. With a running clock a page reads `at` plus the milliseconds since its load began, so text to
the minute is the same on every run as long as the page draws it before the minute of `at` ends (an
instant at the start of a minute leaves the most time); a page that shows seconds or milliseconds
of a time it draws late is measured with `advance: false`.

Not covered: workers (dedicated, shared, service) read the host's clock, as the script runs in
documents only; the time zone is the host's, so a page that prints local times draws other text on
a host in another zone; times the server sends are as it sends them, and only what the page computes
from its clock changes; `ux-browser render` has no clock.

The document names the clock (`conditions.other.clock`): `real` without the key, `from <instant>` or
`frozen at <instant>`, the instant in UTC (`from 2026-01-01T12:00:20Z`).

Measured 2026-11-26, Chrome/149.0.7827.55, `checks --build`:

| page | runs | result |
|---|---|---|
| `tests/fixtures/clock/` (prints `new Date().toISOString()`, how long ago a fixed instant was, and one mark per second of the minute in a 240 px box), its configuration | 2, 7 s apart, `clock: {at: 2026-01-01T12:00:20Z}` | the same three text runs: 230.4, 153.6 and 192.0 px wide (20 marks); overflow 0 in both |
| the same, without `clock` | 2, 7 s apart | the marks 393.6 and 470.4 px wide (41 and 49 seconds), the age 182.4 px; overflow 1 in both: the geometry differs, the value happens not to |
| a scratch page of 24 checks of the clock in the page (the constructor's forms, `instanceof`, a subclass, `parse`, `UTC`, `timeOrigin`, `Intl`, `Temporal.Now`, a 100 ms timer against `performance.now()` and `Date.now()`), each failed check one overflowing line; `ready: {global}` set by that timer | `advance` true, then false | 0 failed in both, ready met; without `clock` 5 failed (the ones that compare a reading with the instant) |
| the second consumer's snapshot (a control plane web application: relative ages and times computed from `Date.now()` and the times in its data), its configuration without `values`, 11 pages x 5 viewports | without, with `clock: {at: 2026-11-25T09:00:00Z}`, without, with | the 44 exact values, their per-viewport details, the findings and the 451 checks equal in all four runs. Recorded geometry (overflow items and always-visible candidates): the two runs with the clock equal on 11 of 11 pages; the two without equal on 7 (on the sessions page a run of text moved 5 px between them); with against without equal on 7 (on four pages runs of text of other widths, 43.1 px against 51.7 px). `layout_shift.after_ready` varied in every pair, as without a ready condition before (A page that draws from fetched data) |

Its ages are drawn to the minute, inside boxes wide enough for them, so the clock changes its
geometry and none of its values.

### Chromium switches: `chrome_args` and `--chrome-arg`

Chromium starts with the switches of `chrome::SWITCHES`: headless (`--headless=new`), the DevTools
port (`--remote-debugging-port=0`), Playwright's switches that change what a page measures, the ones
that keep the browser from reaching out on its own, and two that start no GPU process:

- `--disable-gpu`: no GPU work, by the hardware or by SwiftShader; pages are drawn in software.
- `--in-process-gpu`: what `--disable-gpu` still leaves to a GPU process (compositing) runs on a
  thread of the browser process. With `--disable-gpu` alone Chromium 149 keeps a process of
  `--type=gpu-process`.

A consumer adds switches after these, in the configuration and on the command line:

```yaml
chrome_args: [--enable-gpu, '--use-gl=egl']
```

```sh
ux-browser checks ... --chrome-arg --enable-gpu --chrome-arg '["--use-gl=egl", "--user-agent=probe"]'
```

`chrome_args` is a list of switches. `--chrome-arg` may be given more than once; each value is one
switch, or a JSON list of them (how ratchet passes a list input). The configuration's switches come
first, then the flag's, each in its order. A switch is `--<name>` or `--<name>=<value>`; one that
`SWITCHES` sets with another value is refused and named (`--headless`, `--headless=old`,
`--remote-debugging-port=9222`, `--force-color-profile=display-p3-d65`), as is `--user-data-dir`
(the profile is `--profile`): from the configuration with exit 1, from the flag with exit 2, before
a browser starts. One that `SWITCHES` sets with the same value adds nothing.

Switches that undo the default are allowed. Chromium has no switch that cancels another, so
`--enable-gpu` is read by ux-browser: with it among the added switches, Chromium starts without
`--disable-gpu` and `--in-process-gpu`, with the GPU process and the GPU work of earlier versions, and gets
`--enable-gpu` itself (which it ignores). Other GPU switches (`--use-gl=...`, `--use-angle=...`,
`--enable-unsafe-swiftshader`) are passed as they are, after `--disable-gpu`; give `--enable-gpu` with
them for them to reach a GPU process.

The switches added appear in `run.local_inputs.chrome-arg`, a JSON list (`["--enable-gpu"]`),
outside the evidence: two runs that differ only in switches give documents with the same evidence.
Switches declared in `chrome_args` are also part of the configuration's bytes, so its digest under
`evidence.inputs.config` differs from that of a configuration without them, as for any edit of the
file. `ux-browser render` starts Chromium with `SWITCHES` alone.

**When Chromium does not start.** The run ends with exit 3 and an error that names the exit status
and Chromium's last lines on stderr, at most 20: `Chromium exited before it listened: signal: 6
(SIGABRT) (core dumped); its last 20 lines on stderr:`, one line each below it. The same when
Chromium prints no DevTools endpoint within 30 s. Up to 0.9.1 the error held the status only.

**On a hosted CI runner.** The default switches keep Chromium's sandbox: there is no `--no-sandbox`
in `SWITCHES`. The sandbox needs unprivileged user namespaces; a runner that refuses them (Ubuntu
24.04 restricts them through AppArmor, `kernel.apparmor_restrict_unprivileged_userns=1`) makes
Chromium abort before it listens. llm met a SIGABRT at the start on a GitHub `ubuntu-latest` runner
(2026-11-25, run 45597987448) and reads it as this, not yet verified; the last lines now printed
show the cause. The consumer chooses: allow
the namespaces on the runner (`sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0`), or
start Chromium without its sandbox, `chrome_args: [--no-sandbox]`, for pages it trusts. ux-browser
does not drop the sandbox on its own.

**A low-load run.** On a workstation in use, measure one page and viewport at a time:

```sh
ux-browser checks --config ux/ux-browser.checks.yaml --build website/build \
  --project my-app --assessment page-checks --out page-checks.evidence.json --jobs 1
```

`--jobs 1` keeps one renderer busy instead of four; the values do not depend on it (Timings).

Measured 2026-11-25, Chrome/149.0.7827.55, `checks --build`, on a shared 20-CPU workstation (1-minute
load 7 to 11). Each run was sampled every 0.5 s: the CPU of every Chromium process of the run summed (peak:
the highest sample; CPU: seconds over the run), and the `--type` of each Chromium process. Before:
the switches of earlier versions; after: this version's default.

| site | switches | `--jobs` | wall | peak | CPU | `--type=gpu-process` | values |
|---|---|---|---|---|---|---|---|
| llm's documentation build, `examples/llm.checks.yaml` | before | 4 | 27.4 s | 540 % | 79.9 s | in 52 samples, at most 20 % | all 162 results equal to `examples/llm-docs.evidence.json` |
| the same | after | 4 | 29.5 s | 468 % | 86.7 s | none | 161 equal; `layout_shift.after_ready` of `/llm/docs/concepts/protocols` at 1440 x 900 0 instead of 0.0004 |
| the same | after, `--chrome-arg --enable-gpu` | 4 | 35.0 s | 579 % | 125.5 s | in 66 samples, at most 96 % | all 162 equal; the evidence byte for byte; `run.local_inputs.chrome-arg` `["--enable-gpu"]` |
| the same | after | 1 | 96.2 s | 165 % | 82.6 s | none | all 162 equal |
| the second consumer's snapshot (a control plane web application), 11 pages x 5 viewports | before | 4 | 22.3 s | 566 % | 86.8 s | in 42 samples, at most 178 % | |
| the same | after | 4 | 19.6 s | 485 % | 56.3 s | none | its 55 exact results and its 451 checks equal to the run before |

The one value that moved is a sampled one: the shift of a web font's swap on one page, which a reused
renderer also read as 0 (Timings). Over all runs on llm's build that day: with the switches of before
(or `--enable-gpu`) 0 in 0 of 6 runs; with this version's default 0 in 2 of 5, and the other three
equal to the example document's evidence byte for byte, so it stands; `--disable-gpu` alone 0 in 0 of
2, `--in-process-gpu` alone 1 of 3. On llm's build the GPU process is light and the CPU goes to the
renderers; on the second consumer's snapshot it was the busiest process, and the default takes about
a third of the run's CPU off. `--jobs 1` takes the peak from about 5 CPUs to under 2, for over three
times the wall time.

### The Chrome build: `chrome`

Without a pin a run takes the newest `chromium-<n>` build under the Playwright cache, so a build
removed from the cache moves the next run to another Chrome without a word (the second consumer's
0.4.0 and 0.5.0 runs went from Chrome 158 to Chrome 149). A checks file pins one:

```yaml
chrome: {build: 1228}
chrome: {version: 149}
```

- `build: <n>` takes `chromium-<n>/chrome-linux*/chrome` under the cache.
- `version: <major>` takes the newest build whose Chrome has that major version. The cache holds no
  file that names the version (`chrome-linux64/` has an `ABOUT` without one), so each build's binary
  is asked with `--version` (`Google Chrome for Testing 149.0.7827.55`); it prints the line and
  exits, no browser starts.
- A pin no build meets, or a host with no browser, ends the run after every refusal of the inputs
  (those exit 1 first) and before a browser starts, with exit 3 and nothing written. The message names the pin, the cache and the builds found,
  newest first; for a version pin with the version of each:

  ```
  chrome: the configuration pins chromium-1254, which is not under <cache>; found: chromium-1228
  chrome: the configuration pins Chrome 158, which is not under <cache>; found: chromium-1228 (149.0.7827.55)
  ```

- `UXUI_CHROME` wins over the cache, as before, and the pin is checked against it: a `build` pin
  by the `chromium-<n>` directory the binary lies in, a `version` pin by its `--version`. A binary
  that is not the pinned build ends the run in the same way (exit 3), naming both. A binary outside
  any `chromium-<n>` directory (a wrapper script, a system Chrome) cannot be checked against a
  `build` pin and is refused; pin `version` for it.
- Without `chrome` the run takes the newest build, or `UXUI_CHROME`, as before.
- A value that is not a map of exactly one of `build` and `version` with a whole number above 0
  (`{build: '1228'}`, `{build: 1228, version: 149}`, `{revision: 1228}`, `chrome: 1228`) is refused
  with the configuration, exit 1.

The Chrome that ran is in every document, `evidence.conditions.tools.chromium`
(`Chrome/149.0.7827.55`, from `Browser.getVersion`), as before. The pin itself is not a condition
and not a local input: it is how the binary was found, not what the pages were measured under, so a
pinned run and an unpinned run (or one through `UXUI_CHROME`) on the same Chrome give the same
conditions, and a run on another Chrome differs in `tools.chromium` whether a pin chose it or not.
A pin is part of the configuration's bytes, so its digest under `evidence.inputs.config` changes
with it, as for any edit of the file. `ux-browser render` takes the newest build, or `UXUI_CHROME`.

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
| `layout.page_scroll@1` (units) | sum over viewport x mode | 1 when `scrollingElement.scrollWidth - documentElement.clientWidth > layout.page_slack_px` (default 0) |
| `layout.overflow@1` (blocks) | sum | every non-blank text run (Range box) and every element of `layout.overflow.elements` (default `img, svg, video, canvas, iframe, input, select, textarea, table, pre`) that the page draws (Content the page does not draw): overflow when its right edge passes the viewport or its nearest non-inline block by more than `layout.overflow.tolerance_px` (default 1); none under `position: fixed`, `overflow-x: auto \| scroll` or a clipping box, none inside a match of `layout.overflow.exclude` (default none); at most one per block |
| `always_visible.misses@1` (items) | sum over viewport x mode x top and bottom | the first visible candidate in scope (role and name from the accessibility tree first, then CSS; with `current`, only a current one; Configuration): missing, clipped (not wholly inside the viewport +-0.5 px) or covered (`elementFromPoint` of one of five points of its box, its centre and its four corners, is not it or inside it; The cover test); an item outside its `min_width`, `max_width` or `path_prefix` is not checked |
| `keyboard.misses@1` (checks) | sum over the walks | with a skip link declared: on a fresh load the first Tab is on the skip link; Enter (or one more Tab) puts focus into its target; a skip link that is not first misses both of its checks. On another fresh load up to `max_tabs` Tabs reach each target; with `focus_mark`, each reached target shows a focus mark; each reached target of `activates` acts on Enter or Space (The keyboard walk) |
| `layout_shift.after_ready@1` (score) | largest over the viewports | `layout-shift` entries without recent input after the ready time (above), summed, rounded to 4 places; read once per viewport, before the first mode switch |
| `layout_shift.load@1` (score), opt-in | largest over the viewports | every `layout-shift` entry without recent input since the navigation started (the observer is buffered and set before the page's own scripts), summed, rounded to 4 places; read in the same reading as after ready, after the settle time, so it holds the shifts after ready too. llm's round mode (`readShift().load` of its `ux/core/layout.ts` before 0.3.0): it varies with how fast fonts arrive, so llm reports it and does not ratchet it |

`keyboard.viewports` limits the walk to some viewports, as llm's round walks only its gating ones.

### Content the page does not draw

Chromium lays out content it does not draw when a script asks for its box, so a line inside a
closed `details` has a right edge. The overflow probe skips it:

| item | skipped when |
|---|---|
| element of `layout.overflow.elements` | `checkVisibility({visibilityProperty: true})` is false: no box (`display: none` on it or an ancestor), under `content-visibility: hidden` (the contents of a closed `details` and of `hidden="until-found"`), or its own `visibility` is not `visible` |
| text run | its `visibility` is not `visible`; or its parent element, or the nearest ancestor with a box when the parent is `display: contents`, fails `checkVisibility()`, has `content-visibility: hidden` (its contents are skipped), or is a `details` whose `::details-content` has `content-visibility: hidden` (text straight inside a closed `details`) |

`checkVisibility` is not passed `contentVisibilityAuto: true`: content under `content-visibility:
auto` that is off screen is drawn once it is scrolled to, and its overflow counts as before. Text in a
`display: contents` element is drawn and counts; `checkVisibility()` of that element itself is false,
so the probe asks its nearest ancestor with a box. An element with `visibility: visible` under a
`visibility: hidden` ancestor is drawn and counts.

Measured 2026-11-25, Chrome/149.0.7827.55, `tests/fixtures/checks/hidden-site/` with
`hidden.checks.yaml` (400 x 300; the captures in `hidden/`, 0.6.0's in `hidden-0.6.0/`):

| page | 0.6.0 | now |
|---|---|---|
| `/details/`: `p.full`, one line 263 px wide in a 200 px card, inside a closed `details.more` | 1 | 0 |
| `/details-open/`: the same with `details open` | 1 | 1 |
| `/hidden/`: a line straight inside a closed `details`, a `pre` in one, a line inside and one straight inside `content-visibility: hidden`, one inside `hidden="until-found"`, one under `display: none`, one under `visibility: hidden` | 5 | 0 |
| `/drawn/`: a closed `details`' summary, a line in a `display: contents` span, a visible `pre` under a `visibility: hidden` card, a line below the fold under `content-visibility: auto` | 4 | 4 |

The probe with `checkVisibility({contentVisibilityAuto: true, visibilityProperty: true})` on the
text run's parent and on the element, and nothing else, gave 0, 1, 2 and 2: it counted the line
straight inside the closed `details` and the one straight inside `content-visibility: hidden`, and
dropped the `display: contents` line and the one under `content-visibility: auto`.

llm's documentation build (`examples/llm.checks.yaml`, the same Chrome): all 162 results byte for
byte those of `examples/llm-docs.evidence.json`. Over its 27 page captures the probe measures 1,016
items fewer than 0.6.0's, every one of them a zero box (under `display: none`, never overflow), and
the same 46,064 items at least 1 px wide.

### The cover test

An always-visible item is covered when `document.elementFromPoint` at one of five points of its box
(`getBoundingClientRect`, viewport coordinates) is neither the item nor inside it. The probe
(`src/probes/candidate.js`) records `hit` (true when none is) and `missed`, the names of the points
that are, in this order:

| point | x | y |
|---|---|---|
| `centre` | left + width / 2 | top + height / 2 |
| `top-left` | left + dx | top + dy |
| `top-right` | right - dx | top + dy |
| `bottom-right` | right - dx | bottom - dy |
| `bottom-left` | left + dx | bottom - dy |

dx and dy are each corner's own: 1 + rx (1 - 1/sqrt(2)) and 1 + ry (1 - 1/sqrt(2)), at most width / 2
and height / 2, where rx and ry are that corner's used `border-radius` along x and y. The used radii
are the computed ones (a length, or a percentage of the box's width or height), all scaled by one
factor when two radii on one side add up to more than that side, as CSS draws them: a pill of 160 x 40
with `border-radius: 999px` has radii of 20. Without a radius a corner point lies 1 px inside the box.
Hit testing follows `border-radius`, so a point 1 px inside a rounded corner lies outside the element;
r (1 - 1/sqrt(2)) is where the corner's curve crosses its diagonal, and the 1 px puts the point inside
it. The bound keeps a corner point of a box a few pixels wide or high from passing the centre.

A capture of 0.9.1 and before has `hit` of the centre only and no `missed`; it reads as before.

Measured 2026-11-25, Chrome/149.0.7827.55, `tests/fixtures/checks/cover-site/` with
`cover.checks.yaml` (400 x 300, one mode; one item `button#item` per page, declared only on its page;
the pages do not scroll, so an item counts at the top and at the bottom: 2 per covered item; the
captures in `cover/`, 0.9.1's in `cover-0.9.1/`):

| page | 0.9.1 | now | `missed` |
|---|---|---|---|
| `/corner/`: the item (160 x 60 at 20,10) under a sticky bar from x 170, 30 px high, at its top right corner only | 0 | 2 | `top-right` |
| `/centre/`: a 40 x 20 badge over the item's centre, its corners free | 2 | 2 | `centre` |
| `/free/`: the item alone | 0 | 0 | none |
| `/pill/`: a 160 x 40 pill (`border-radius: 999px`) alone | 0 | 0 | none |
| `/dialog/`: an open `dialog` from x 110 over the item's right side, short of its centre | 0 | 2 | `top-right`, `bottom-right` |

The same probe with every corner 1 px inside the box and no radius term gave the pill 2, all four
corners missed: each lies outside the curve.

Not covered: an inline item that wraps over two lines (a link, for one) has one bounding box over
both lines, and a corner of it can lie on space the item does not draw; such an item is reported
covered. Sampling each of its line boxes (`getClientRects`) would avoid it; no consumer has asked.
`missed` stays in the capture; a finding names `covered` only.

### The keyboard walk

One walk per page, viewport and keyboard mode. Its checks, each a miss or not, in this order (the
node path of a miss is `<viewport>/<mode>/<check>`):

| check | when | a miss |
|---|---|---|
| `skip-link-first` | a skip link is declared (`skip_link_is` or `skip_link_contains`) | on a fresh page, the label of the first Tab stop (its `aria-label`, else its text) is not the skip link's name |
| `skip-link-moves-focus` | a skip link is declared | the skip link is not first, or neither Enter on it nor one more Tab puts focus on its target (the element its `href="#id"` names) or inside it |
| `tab-reaches-<target>` | per target of `must_reach` | on another fresh page, no Tab of the first `max_tabs` puts focus on the target or inside it; a target with no visible candidate on the page is not reached |
| `focus-mark-<target>` | `focus_mark: true`, per reached target | the element that has focus on the Tab that first reached the target shows no focus mark (below) |
| `activates-<target>` | per reached target of `activates` | neither Enter nor Space on it acts (below) |

**A fresh page.** The first walk of a page and viewport starts on the page the checks just measured:
they only scrolled it and read it. When no element has focus (`document.activeElement` is the body)
and nothing is selected, the walk scrolls it to the top, waits for fonts and two frames, and takes it
as fresh; else it loads the page. Every other fresh page is a load: the Tab walk after the skip
link's Enter (focus moved, and the fragment set the point the next Tab starts from), each activation,
and every check of a second walk on the same page and viewport (it starts where the first one left
focus). With a ready condition a load waits for it. So a walk with a skip link loads the page once,
a walk without one not at all, plus one load per activation (`measure::walk_plan`,
`measure::planned_loads`; up to 0.11.0 every fresh page was a load: two and one).

Without a skip link the walk has no skip-link check: a page that
has none and declares none misses nothing for it. An unreached target is one miss, whatever
`focus_mark` and `activates` say of it. A configuration of 0.5.0 (a skip link, no `focus_mark`, no
`activates`) walks and counts as 0.5.0 and before did. A target's `min_width`, `max_width` and
`path_prefix` do not limit the walk: it runs at the viewports of `keyboard.viewports`, so a target
shown only on narrow viewports is walked only where those name narrow ones.

**A focus mark.** On the Tab that first puts focus on a target or inside it, `probes/focus-mark.js`
reads the focused element (`document.activeElement`): whether it matches `:focus-visible`, its
computed `outline-style`, `outline-width` (px), `outline-color` and `box-shadow`, and a short
description of it. The decision is the tool's (`checks::focus_marked`): the element matches
`:focus-visible`, and its outline is drawn (`outline-style` not `none`, `outline-width` above 0,
`outline-color` not fully transparent) or its `box-shadow` is not `none`. That is the second
consumer's own rule (its focus-visible check) with one difference: an outline of a fully transparent
colour is no mark here. A mark drawn only by a change of colour, background or border, or by a
pseudo-element, is not seen; a page that marks focus so declares no `focus_mark`.

**Enter or Space acts.** For each target of `activates` the walk reached on its `n`-th Tab: a fresh
load, `n` Tabs (focus must be on the target or inside it again, else the check misses), a quiet watch
of `focus_settle_ms` with nothing pressed, then Enter and a wait of `focus_settle_ms`; when Enter did
not act, Space and the same wait. A key acts when, within its wait:

- the page navigated: the browser reported `Page.frameRequestedNavigation`,
  `Page.frameStartedLoading` or `Page.navigatedWithinDocument` (a link, a submitted form, a new hash,
  `history.pushState`);
- focus moved to another element;
- the focused element changed its state: `aria-expanded`, `aria-pressed`, `aria-selected`,
  `aria-checked`, or its `checked`, `value` or `open`;
- the document changed where it had not changed in the quiet watch: an element was added or removed,
  or `hidden`, `open`, `aria-hidden` or `disabled` changed on an element, on a node that did not change
  during the watch (a ticking clock does not count, a dialog that opens does).

`probes/activate.js` reports these facts per key; the decision is the tool's (`checks::acted`). Not
seen: a key whose only effect is a change of text, a new window, or a change that comes after the wait
(a slow request). A page that changes itself within the wait in one of these ways, on a node it did
not change in the quiet watch, is taken to act. A configuration without `activates` presses neither
key.

The walk's record (`--capture`) keeps `located` and `tabs` as before; `first_label`, `after_enter`
and `after_tab` only with a skip link; `marks` (per reached target the focus-mark probe's answer)
only with `focus_mark`; and `activations` (per reached target `{refocused, enter, space}`, each key's
facts or `null` for a key not pressed) only with `activates`.

Measured 2026-11-25 with `--build`, Chrome/149.0.7827.55:

| site and configuration | keyboard checks | `keyboard.misses@1` |
|---|---|---|
| the second consumer's snapshot (a control plane web application, 11 pages, walks at 2 viewports), its configuration without the skip link, with `focus_mark: true` and `activates: [screens]` | 66: reach, focus mark, Enter per walk | 0 on every page, as its own keyboard rule (Tab reaches the step's control within 250 presses, a visible focus mark, Enter or Space acts) has 0 on the same snapshot; 0.3.0 had 4 per page (2 skip-link checks per walk). Its 33 other exact values are unchanged |
| the same, one page copied with `:focus, :focus-visible { outline: none !important; box-shadow: none !important; }` | 6 | 2: `focus-mark-screens` at each viewport, ``focused a "Outbox": outline none 3px ...`` |
| the same snapshot with its 0.3.0 configuration (a skip link) | as 0.3.0 | 4 per page; all 55 exact results byte for byte those of its 0.3.0 document |
| llm's documentation build, `examples/llm.checks.yaml` (a skip link, no `focus_mark`) | 1,234 checks in all | every one of the 162 results byte for byte that of `examples/llm-docs.evidence.json` (135 values) |
| a test page of seven buttons and links (scratch, not committed), all seven in `activates` | 21 | 2, the two that do nothing on either key: a button, and a button beside a clock that redraws every 20 ms. Acted: an `aria-expanded` toggle (Enter), an element added (Enter), a modal dialog (Enter: focus moved), a `role=button` that acts on Space only, a link to `#out` (`Page.navigatedWithinDocument`) |

Not covered: axe-core's rules (a third-party library; llm runs `@axe-core/playwright` 4.14.0), reading
load and the screen-only user (llm's other checks), the scroll containers llm's round reports, and the
screenshots of a round. Playwright blocks service workers; this run does not (a Docusaurus build
registers none).

A round of llm's loop runs a second configuration of the same checks,
`engine/browser/examples/llm-round.checks.yaml`: the gating viewports 1440 x 900 and 390 x 844 and the
measured ones 1280 x 800, 1024 x 768 and 768 x 1024 (the docs-site sizes of the skill's adapter
table), settle 750 ms, `layout_shift.load` on, the keyboard walk at the gating viewports only, and
`values.gating` so that the measured viewports are reported per viewport and gate nothing.

### Journeys

The checks above run on a page as it loads. A journey brings a page into a state by steps (open a
menu, fill a form, open a dialog, open a closed `details`) and checks it there. Every node a step
names is a CSS selector, never an `ess-ui/1` node path (spec scenario
`browser-journey-names-nodes-by-selector`).

```yaml
journeys:
  - name: open details
    page: /details/
    steps:
      - {click: 'details.more > summary'}
      - {state: opened}
  - name: search
    page: /search/
    timeout_ms: 2000
    steps:
      - {type: '#q', text: a word}
      - {press: Enter}
      - {wait: '#results p'}
      - {scroll: bottom}
      - {state: results}
```

| key | value |
|---|---|
| `name` | the journey's name, unique, not empty, without `/` or a line break |
| `page` | the path it starts on, served as one of `pages.paths` is (with `--url`, moved by the URL's path in the same way); it need not be one of the measured pages |
| `steps` | a list of steps, at least one of them a `state` |
| `timeout_ms` | how long a `wait` step waits; a whole number above 0; default 5000 |

A step is a map with exactly one of these keys (`text` goes with `type`):

| step | what it does | fails when |
|---|---|---|
| `{click: <css>}` | takes the first element the selector matches that is rendered (`checkVisibility()`, a box of some size), scrolls it into view when part of it is outside the viewport, and clicks the left mouse button at the centre of its box (`Input.dispatchMouseEvent`: a move, a press, a release) | the selector does not parse, matches no element, matches no rendered element, or the element at the centre is neither it nor inside it (another element covers it) |
| `{type: <css>, text: <text>}` | the same element, focused and its content selected; the text is typed in place of it (`Input.insertText`, which sends the page's `beforeinput` and `input` events) | as `click`, or the element does not take focus |
| `{press: <key>}` | a key on the focused element, as the keyboard walk presses it: `Tab`, `Enter`, `Space` or `Escape` | never |
| `{wait: <css>}` | until the selector matches a rendered element, watched with a `MutationObserver` on the document (nodes, attributes, text), not polled | the selector does not parse, or no rendered element matched within `timeout_ms` |
| `{scroll: top \| bottom}` | the page scrolled to its top or its bottom, then `scroll_settle_ms` | never |
| `{state: <name>}` | fonts loaded, two frames drawn and `settle_ms`; then the checks of the state. A name is unique in its journey, not empty, without `/` | never |

A click or a key that starts a navigation (a link, a submitted form) is followed: when the browser
reports it before the step ends, the run waits for the new document to stop loading (at most 10 s),
as the keyboard walk does. A step after it that needs the new page's content should `wait` for it.

**What runs in a state.** Per journey, viewport and mode: a fresh browser context opened in that
mode, the page loaded and ready as for the checks above (its `load` event or `ready`, then
`settle_ms`, and the mode shown in `mode_attribute`), then the steps in order. At each `state`, on
the page as the steps left it (no reload, no scroll): page scroll, overflow (with the `layout` rule),
and the always-visible items that apply to the journey's `page` at that viewport, each checked once
where the page stands, at position `state`. The keyboard walk and the layout shifts are not run in a
state: each needs a fresh load, which would undo the steps.

**A step that fails** ends the journey at that viewport and mode; nothing crashes and the run goes
on. Each of its states after the step is one `not_run` entry in `evidence.coverage.not_executed`, id
`journey/<journey>/<state>/<viewport>/<mode>`, reason the step and why: ``step 2, click `#nothing`:
matches no element``, ``step 1, wait `#never`: no rendered element matched it within 300 ms``. A page
that does not load (step 0), is not ready in time or does not apply the mode skips every state of the
run the same way. A state with a skipped run has its results incomplete, cause `incompatible_target`,
detail ``not run at <viewport>/<mode>: <reason>``; a count above 0 from its other runs is kept as a
lower bound (`at_least: true`), a count of 0 is left out. The browser failing ends the run (exit 3), as
for a page.

**Identity.** Each state has its own results: scope `state`, item `<journey>/<state>`
(`open details/opened`), the numbers `layout.page_scroll@1`, `layout.overflow@1` and
`always_visible.misses@1` and a `findings` set, after the pages' results in the same order (the
numbers of every page, then of every state; the sets of every page, then of every state). Each has
`details.page`, `details.journey` and `details.state`; the numbers have `details.viewports` as a page's
do. A finding's node path is `journey/<journey>/<state>/<viewport>/<mode>/<what>`, `<what>` as on a
page (`brand@state` for an always-visible item). A scope of its own keeps a ratchet's measurements of
pages apart from those of states: a page's results are the same with or without journeys. A
configuration without `journeys` writes the document it wrote before, byte for byte. `--pages` limits
the journeys to those whose `page` is in its list.

**Refused** at configuration read, exit 1 before the browser is looked for, the message naming the
journey and the step (``journey `open details`: step 2: unknown key `hover`; a step is one of click,
type, press, wait, scroll, state``): an unknown key in a journey or a step; a step with none or two of
the step keys, or `text` without `type`; a `click`, `type` or `wait` without a selector; `type` without
`text`; a key that is not one of the four; a `scroll` that is not `top` or `bottom`; a state named twice
in a journey, or with `/`; a journey without steps or without a `state`; a journey without `name` or
`page`, named twice or with `/`; a `page` that does not start with `/` (or, with `--url`, has no place
on the server); a `timeout_ms` that is not a whole number above 0.

**Capture.** `--capture` writes each journey as `journey-<name>.capture.json` (the name in lower case,
every other character `-`): per viewport, per mode `{mode, mode_applied, states, failed}`, each state
`{state, overflow, visible}` as a page's unit records them, `failed` `{step, error}` or null.

Measured 2026-11-25, Chrome/149.0.7827.55, `tests/fixtures/checks/journey-site/` with
`journey.checks.yaml` (400 x 300, one mode; the captures in `journey/`):

| journey and state | steps to it | after load (the page) | in the state |
|---|---|---|---|
| `open details/opened` | click `details.more > summary` | overflow 0 (`/details/`: the line 263 px wide in a 200 px card is inside the closed `details`) | overflow 1: `p.full`, right edge 263 past its parent at 200 |
| `open menu/closed` | none | always visible 0 (`/menu/`: `brand` and `toggle`, top and bottom) | always visible 0 |
| `open menu/opened` | click `#toggle`, wait `#menu a` | always visible 0 | always visible 1: `brand@state` covered by the opened drawer; `toggle` ok |
| `search/results` | type `#q`, press `Enter`, wait `#results p`, scroll bottom, press `Escape` | overflow 0 (`/search/`) | overflow 1: the typed word, 372 px, in a 200 px box |
| `missing button/before`, `/after` | none; click `#nothing` | always visible 0 | 0; `after` not run: ``step 2, click `#nothing`: matches no element`` |
| `slow wait/never` | wait `#never`, `timeout_ms: 300` | always visible 0 | not run: ``step 1, wait `#never`: no rendered element matched it within 300 ms`` |

Coverage of that run: 29 checks executed (13 of the pages, 16 of the 5 states reached), 3 failed, 2
`not_run`.

Not covered: a step on an element inside an `iframe` or a shadow root (the selector is matched in the
document), a hover, a drag or a key other than the four, a wait for a change that touches no node
(a CSS transition, a canvas), and a state's keyboard walk and layout shift (above).

### Design tokens: `tokens`

Whether a page uses only the scale's font sizes, the theme's colours, the spacing unit and its
radii, and how well its text stands out from its background. Story `story:ux-browser-design-tokens`;
the outputs are the live-page form of the design proxies of `docs/metrics.md` § 11 and of
`a11y.contrast_min` (§ 8).

```yaml
tokens:
  font_sizes: {base: 16, ratio: 1.25}
  colours:
    properties: ['--color-*', --focus-ring]
    values: ['#000000', 'rgba(0, 0, 0, 0.5)']
  spacing_unit_px: 4
  radii: [4, 8, full]
```

| key | value |
|---|---|
| `font_sizes` | the type scale: a list of sizes in CSS pixels (`[12, 13, 17]`), or `{base, ratio}`, every size `base x ratio^k` for a whole number `k` (`{base: 16, ratio: 1.25}`: ..., 12.8, 16, 20, 25, 31.25, ...); each above 0, the ratio above 1 |
| `colours.properties` | custom properties read from the page's root (`getComputedStyle(document.documentElement)`) in each mode, so a theme's light and dark values are each in their mode; `--prefix*` takes every custom property whose name starts with `--prefix` and holds a colour (names from the root's computed style and the page's readable style sheets) |
| `colours.values` | colours declared as values: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb()`, `rgba()` |
| `spacing_unit_px` | the spacing unit in CSS pixels, above 0 |
| `radii` | the allowed corner radii in CSS pixels (0 or more), and `full`: a corner rounded to at least half its box's shorter side (a pill, a circle, `50%`, `999px`) |

Every key is optional; the section declares at least one, and `colours` at least one property or
value. A section that does not read (an unknown key, an empty list, a value that is not a number,
a size or radius below 0, a ratio of 1 or less, a property that is not `--name` or `--prefix*`, a
colour in another form) is refused with the configuration, exit 1, before a browser starts. Without
the section nothing changes: no probe runs, and the captures, documents and values files are the
bytes they were (`tests/tokens.rs` holds the documents of the test runs to the bytes of the build
before the section existed).

**What is measured.** Per page, viewport and mode, at the top of the page after the always-visible
items, the probe `src/probes/tokens.js` reads every element the page draws (`checkVisibility` with
`visibilityProperty`; not inside an `svg`) and returns each distinct value once, with how often it
is used and the first node that uses it (`tag#id.class "text"`, as the overflow check names a block):

| what | read from |
|---|---|
| font size and text colour | an element with text of its own (a non-blank text node as a child): `font-size`, `color`, `font-weight` |
| colours | `color` (of such an element), `background-color`, and `border-*-color` of each side with a width above 0 and a style other than `none` and `hidden`; a fully transparent colour is not used |
| gaps | between an element and the box before it among its siblings: both drawn boxes (not `display: inline`, `contents` or a table part; not `position: absolute` or `fixed`), with no text and no drawn inline element between them; the distance below it, or beside it (to the right, or to the left), when above 0 |
| paddings | each of `padding-top`, `-right`, `-bottom`, `-left` above 0 |
| radii | each corner's `border-*-radius` (its first value; a percentage of the box's width) above 0, and whether it is at least half the box's shorter side (`full`) |
| text on its background | per element with text: its colour, and the background colours of the element and its ancestors composited down to the first opaque one; unknown when a background image lies in between or no opaque background is reached (the canvas) |

Colours are read as sRGB: `rgb()` and `rgba()` as the computed style writes them, `color(srgb ...)`
by number, any other form drawn on a 1 x 1 canvas and read back. The declared custom properties
are read the same way.

**The rules**, in Rust (`src/tokens.rs`), each a stated tolerance:

| result (`scope: page`, `exact`) | per viewport and mode | tolerance |
|---|---|---|
| `proxy.type_scale@1` (sizes) | distinct font sizes of text off the scale | a size within 0.5 px of a size of the scale is on it |
| `proxy.colours@1` (colours) | distinct colours of text, backgrounds and borders that match no declared colour | each of red, green and blue within 2 (of 255), alpha within 0.02 |
| `proxy.spacing_units@1` (gaps) | distinct gaps and paddings that are not a multiple of the spacing unit; one value counts once, as a gap and a padding | within 0.5 px of a multiple |
| `proxy.radii@1` (radii) | distinct radii that match no declared radius; a radius below 0.5 px is none; a `full` corner matches `full` | within 0.5 px |
| `a11y.contrast_min@1` (ratio) | the lowest WCAG contrast ratio of text on its known background, the text's alpha composited on it, rounded down to 2 places; text below its limit (4.5, or 3 for large text: 24 px, or 18.66 px at weight 700 or more; WCAG 1.4.3) is an offender | none |

Per page a count is summed over viewport x mode and the contrast is the lowest; `details.viewports`
has the value per viewport (a count summed over its modes, the contrast the lowest). The
`details.offenders` list names each value off the tokens (`{viewport, mode, node, value, what}`;
`what` the properties of a colour, `gap` or `padding`, `full` for a radius, the colours and limit
of a contrast), at most 20 per result in viewport and mode order, then the first use in document
order; `details.more` counts the ones left out. `details.unjudged` counts text whose background is
unknown, which the contrast does not judge. Not judged at all: text over an image, a gradient, a
canvas or a sibling drawn behind it (the background comes from ancestors only), `opacity`, and the
colours of `svg` content.

**Undeclared or unknown is not run, never a pass.** A kind the section does not declare, a declared
custom property the root does not define in a mode or that holds no colour there (or a `--prefix*`
that matches no colour), and a page with no text on a known background for the contrast, make that
output not run at that viewport and mode: one `not_run` entry in `evidence.coverage.not_executed`,
id `<page>/<viewport>/<mode>/<output without suffix>` (`/on-scale//400x300/dark/proxy.colours`),
reason the cause (``the page's root does not define `--only-light` ``, `the tokens declare no
radii`). The page's number is then incomplete, cause `coverage_below_plan`, detail ``not run at
<viewport>/<mode>: <reason>``; a count above 0 from the other viewports and modes is kept as a lower
bound (`at_least: true`), anything else has no value. A run that judged an output counts one check
per viewport and mode, passed when it has no offender. The values file takes the five numbers as
it takes the others (`ux-browser values`): `proxy.type_scale`, ..., `a11y.contrast_min`, the
contrast folded to the smallest over the gating viewports; an incomplete one is a problem line
there, as any incomplete result. The token outputs are not gated (`values.gates` names only the
outputs with findings and the visual outputs) and run on pages, not in the states of a journey.

The declared tokens are part of the configuration's bytes (its digest under
`evidence.inputs.config`) and add no condition. **Capture.** With the section each unit of a page's
capture has `tokens`, the probe's answer (`properties`, `colours`, `nodes`, `sizes`, `usage`, `text`,
`gaps`, `radii`, `elements`); a configuration with the section refuses a capture recorded without
it, and `run.timing` has the probe's milliseconds per mode as `tokens`.

Measured 2026-11-27, Chrome/149.0.7827.55, `tests/fixtures/checks/tokens-site/` with
`tokens.checks.yaml` (400 x 300, light and dark; the captures in `tokens/`):

| page | type scale | colours | spacing | radii | contrast min |
|---|---|---|---|---|---|
| `/on-scale/`: sizes 12.8, 16, 20, 25 on `{base: 16, ratio: 1.25}`, colours from `--color-*`, gaps and paddings of 4, 8, 12, 16, radii 4, 8 and a pill | 0 | 0 | 0 | 0 | 6.66 |
| `/offenders/`: a size of 18 px, the colour `#2e7d32`, a gap of 13 px and a padding of 10 px, a radius of 6 px, faint text (`--color-muted`) | 2 (18 px in each mode) | 2 (`#2e7d32`) | 4 (13 px, 10 px) | 2 (6 px) | 1.74 (faint text in dark; 2.16 in light) |

With `tokens-unknown.checks.yaml` (sizes as a list, colours as values plus `--only-light`, which
the page defines in light only, no spacing unit and no radii; the capture in `tokens-unknown/`):
type scale 0 and contrast 6.66; colours, spacing and radii incomplete, 5 `not_run`.

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
| `evidence.conditions` | `tools: {ux-browser, chromium}`, `other: {viewports, modes, ready, settle_ms, reduced_motion, page_slack_px, overflow_tolerance_px, overflow_elements, overflow_exclude, clock}`; `ready` is `load` without a ready condition, else `selector <css>` or `global <name>`. A document of 0.3.0 has no `ready`; its ready was `load`. The layout keys are the rule values the run used, defaults included (`"0"`, `"1"`, the elements joined by `, `, the exclusions joined by `, ` or `""`); a document of 0.4.0 or before has none, its rule was the default. `clock` is `real` without a clock, else `from <instant>` or `frozen at <instant>` in UTC (A fixed page clock); a document of 0.4.0 or before has none, its clock was the host's |
| `run.local_inputs` | `jobs` (the value used), `capture`, `profile`, `values` and `visual` when given, and `chrome-arg`, the switches added by `chrome_args` and `--chrome-arg` as a JSON list, when there are any: they differ per machine and change no value, so they stay outside the evidence |
| `run.timing` | where the run spent its time (Timings, The run's times): `{unit: ms, pages: [...]}`, one entry per page and viewport measured. Outside the evidence: the evidence of the same measurements is the same bytes with and without it. Absent when no page was measured |
| `evidence.coverage` | unit `check`: one check on one viewport and mode (page scroll, overflow, each applicable item at the top and at the bottom), one keyboard check per walk check (The keyboard walk: the skip link's two when declared, each target, each focus mark, each activation), and one counted layout-shift reading per viewport; per state of a journey and its viewport and mode, page scroll, overflow and each applicable item once. A page that did not load is one `not_run` entry, id the page path; a state a run did not reach is one, id `journey/<journey>/<state>/<viewport>/<mode>` (Journeys) |
| `evidence.results[]` | per page the five numbers (six with `layout_shift.load`, after the shift after ready), then per page a `findings` identity set (`[rule, node_path]`, node path `<viewport>/<mode>/<what>`, reason in `fields`). The four counts are `exact`; layout shift is `sampled` with `samples` (the viewports read), so the document has `run.started_at` and `run.ended_at`. Every number has its per-viewport values in `details.viewports` (`{<viewport>: "<value>"}`: a count summed over the viewport's modes or walks, a shift as read); the keyboard count keeps its `details.walks`: per walk `viewport`, `mode`, `skip_link_first` and `skip_moves_focus` (only with a skip link), `reached`, `focus_marks` (when a mark was judged) and `activates` (when an activation was judged), each a map of target to true or false. A keyboard finding's reason is `missed`, except a focus mark's (the focused element and what it showed) and an activation's (what failed). With a `tokens` section, after those numbers: the five token numbers (Design tokens). After the pages' numbers and before their sets: per state of a journey three numbers and a set, scope `state`, item `<journey>/<state>` (Journeys) |

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
| `inputs` | `config` (path, required), `build` (path) and `url` (text), of which a run gets exactly one (a choice of the usage; the format has no way to say so), `pages` (text: paths separated by commas), and the local inputs `capture`, `profile`, `values` (paths), `jobs` (integer), `chrome-arg` (a list of text: the added Chromium switches) and `visual` (text: `compare` or `update`); `values` names a file the run writes and changes no value of the document, as `capture`, and `visual update` writes the baseline after the document, which is the same in both |
| `conditions` | `viewports`, `modes`, `ready`, `settle_ms`, `reduced_motion`, `page_slack_px`, `overflow_tolerance_px`, `overflow_elements`, `overflow_exclude`, `clock`: the keys of `conditions.other` |
| `coverage_unit`, `coverage` | `check`; all six counts |
| `outputs` | the six numbers (`scope: page`; page scroll, overflow and always visible also `scope: state`, as is `findings`; unit `units`, `blocks`, `items`, `checks`, `score`, `score`; the two layout shifts `sampled`, the others `exact`), the five token numbers (`scope: page`, `exact`; unit `sizes`, `colours`, `gaps`, `radii`, `ratio`; Design tokens) and `findings` (`identity_key: [rule, node_path]`, `item_fields: [reason]`) |

A test (`tests/assessor.rs`) reads it and compares its inputs with the arguments of the `checks` usage
(required where the usage neither brackets them nor puts them in a choice `( | )`), its conditions with a document's, and its outputs with
every result and the target kind of the example document and of the test documents (a build's and a running server's).
The Chrome pin (`chrome`) adds no input and no condition: it is read from the configuration, which
is an input already, and the Chrome it chose is `conditions.tools.chromium` (The Chrome build).

### Visual ratchet: `visual`

Whether a page still looks as it did: per page, viewport and mode a screenshot, compared pixel by
pixel with a baseline PNG within a declared tolerance. Story `story:ux-browser-visual-ratchet`; the
visual ratchet of the `ui-iteration-loop` skill (§ 6).

```yaml
visual:
  dir: ux/visual
  full_page: true
  masks: ['[data-live]', '.clock']
  tolerance: {share: 0.0005, threshold: 8}
  tolerances:
    - {page: /run/, share: 0.002}
    - {viewport: phone, threshold: 16}
```

| key | value |
|---|---|
| `dir` | the directory of the ratchet, under `--root`: `baseline/` (the consumer's, committed), `shots/` (the run's screenshots) and `diff/` (the run's diff images); required |
| `full_page` | `true`: the full page is shot besides the viewport; default `false` |
| `masks` | CSS selectors whose boxes are blanked in every shot (live data, clocks, carets); a list, none empty, each once; default none |
| `tolerance` | `{share, threshold}` of a unit no rule names: `share` the part of its pixels that may differ (0 to 1), `threshold` the difference of a channel (0 to 255) up to which a pixel counts as equal; each defaults to 0, so by default one differing pixel fails |
| `tolerances` | rules `{page, viewport, share, threshold}`, each naming a page path, a declared viewport or both; the first rule that matches a unit gives its tolerance, a key it leaves out is the section's `tolerance` |

A section that does not read (an unknown key, no `dir`, a share outside 0 to 1, a threshold that is
not a whole number from 0 to 255, a rule without `page` and `viewport`, a viewport not declared, a
page that does not start with `/`) is refused with the configuration, exit 1, before a browser
starts. Without the section nothing changes: no screenshot is taken, no file is written under a
`dir`, and the captures, documents and values files are the bytes they were.

**Files.** One per unit, the same name in the three directories:
`<page>.<viewport>.<mode>.<shot>.png`, `<page>` as a capture's (`home` for the base path, else the
path's words joined by `-`), `<shot>` `viewport` or `page`: `run.1366x768.dark.viewport.png`. A run
with the section first removes the PNG files in `shots/` and `diff/`, so every file there is of
that run; it never removes one from `baseline/`.

**Commands.**

```sh
ux-browser checks --config ux/ux-browser.checks.yaml --build site --project my-app \
  --assessment visual --out visual.evidence.json [--values visual.json] [--visual compare]
ux-browser checks ... --visual update
```

Every checks run with the section compares (`--visual compare`, the default). `--visual update`
also writes each shot of the run to `baseline/` after the document is written; the document is the
same as without it (the comparison with the baseline as it was), and baseline files of units the
run did not take (`--pages`) stay. Only `update` writes the baseline. Exit statuses as for every
checks run: a unit over its tolerance is a failed check and a finding, not an exit status;
`--visual` with a configuration without the section is refused (exit 1), any value but `compare` and
`update` is a wrong invocation (exit 2), both before a browser starts; a shot or baseline that is
no PNG this reader reads is refused (exit 1, no document written); a shot or diff image that cannot
be written is exit 3.

**A shot.** In each mode, first, on the page as it loaded (or as it was switched to the mode),
before the other checks of that mode: the probe `src/probes/visual.js` makes the page still (the
text caret transparent, through a style sheet the document adopts, no element added; every
animation and transition `document.getAnimations()` returns finished when it ends, or paused at its
start when it repeats forever; scrolled to 0, 0; fonts and two frames awaited) and returns the boxes
of the mask selectors at scroll 0. Then `Page.captureScreenshot` of the viewport (device scale 1),
with `full_page` the page (`captureBeyondViewport`, the viewport's width and the larger of the
document's scroll height and the viewport's height), each box filled with `#ff00ff`, out to whole
pixels; then the style sheet is dropped and the paused animations play again. A shot with no mask
box in it is Chromium's bytes; one with a box is written by `src/png.rs`. A mask that matches
nothing blanks nothing; a selector the browser does not parse makes the probe throw, and the run
exits 3. With the section, the other checks of the mode run on the page after its shots (finished
animations, and after a full-page shot a page that was resized for the capture and back).

**Comparison** (`src/visual.rs`, no browser): a pixel differs when one of its four channels differs
from the baseline's by more than the unit's `threshold`. The unit's share is the differing pixels
over its pixels, in millionths rounded up (one differing pixel never reads 0); it passes when the
differing pixels are at most `share` times its pixels. A shot of another size than its baseline
fails, its reason `size 240x160 against the baseline's 240x320`, its share 1. A unit with no
baseline file is not run: a `not_run` entry, never a pass. For a unit with a differing pixel the
diff image is written to `diff/`, the shot's size: each differing pixel `#ff0000`, every other the
baseline's grey, faded three quarters of the way to white.

PNG files are read and written by the crate itself (`src/png.rs`, no crate): 8-bit, non-interlaced
grey, grey and alpha, RGB and RGBA (what Chromium writes), inflated as RFC 1951 says; written as
RGBA, rows filtered `Up`, deflated with the fixed Huffman codes. The same pixels give the same
bytes.

**Outputs**, after the numbers of the pages and journeys and before the findings sets:

| result (`scope: page`, `exact`) | value | unit | `details` |
|---|---|---|---|
| `visual.diff_share@1` | the largest share over the page's units | `ratio` | `viewports`: the largest per viewport; `units`: per unit `viewport`, `mode`, `shot`, `result` (`passed`, `failed`, `not_run`), and for a compared one `width`, `height`, `px`, `share`, `tolerance`, `threshold`, `baseline` and `shot_digest` (SHA-256 of the two files), `diff` (`diff/<file>`, when written) and for two sizes `reason`; for one not run its `reason` |
| `visual.diff_px@1` | the differing pixels, summed over the page's units | `px` | `viewports`: the sum per viewport |

A page with a unit not compared has both numbers incomplete (`coverage_below_plan`, the detail
names each such unit and why); a value above 0 is then kept as a lower bound with `at_least: true`,
any other value is left out. A page that could not be measured has both incomplete with its error
(`incompatible_target`), as its other numbers; its units count nothing more. A unit over its
tolerance is a finding in its page's set: rule `visual.diff_share@1`, node path
`<viewport>/<mode>/<shot>`, reason `3200 of 38400 pixels differ (0.083334), above the tolerance
0.001 at threshold 0` (or the two sizes). Coverage: one check per compared unit; a unit not compared
is a `not_run` entry, id `<page>/<viewport>/<mode>/visual/<shot>`. No condition is added: the
configuration's digest (an input) covers the masks and tolerances, and each unit names its
baseline's digest. The values file takes both numbers per scenario as any other; with `gating`, a
share (a metric named `*_share`) is the largest over the gating viewports and `visual.diff_px` the
sum.

**A hard gate.** Without a visual output in `values.gates` a differing screenshot is a value of the
ratchet, not a gate failure, and the values file is the bytes it was. `values.gates` takes each of the
two with a bound:

```yaml
values:
  gates:
    visual.diff_share@1: {gate: 6, max: 0.001}
    visual.diff_px@1: {gate: 7, max: 400}
```

Each unit at a gating viewport (`details.units` of `visual.diff_share@1`) is held to the bound, not to
its `tolerance`: a share over the bound (differing pixels over `max` times the unit's pixels, as the
tolerance is compared), or differing pixels over `max`, is a failure; a shot of another size than its
baseline is always over. A unit not run (no baseline, no shot) is a failure too, never a pass. Each
failure is one line of its page, viewport and mode in the values file (`units`), its detail
`visual.diff_share:<shot>`, `visual.diff_px:<shot>` or `<...>:<shot>:not-run`, its key
`gate6|/moved/|small|light|visual.diff_share:viewport`, waived by an open finding record as any other.
An unwaived one is a problem line with the number and the bound, or why the unit was not run, and
`ux-browser values` (and `checks --values`) exits 1:

```
ux-browser: hard gate failure not waived by an open finding: gate6|/moved/|small|light|visual.diff_share:viewport (share 0.083334 above the bound 0.001)
ux-browser: hard gate failure not waived by an open finding: gate6|/clock/|small|dark|visual.diff_share:page:not-run (no baseline baseline/clock.small.dark.page.png)
```

A unit not run also leaves its page's numbers incomplete (`incomplete: /clock/ visual.diff_share@1:
coverage_below_plan`), a problem line of its own, so a waiver of the gate failure does not end the run
with 0. A page that could not be measured has no units: its numbers are incomplete, a problem line.
A run whose units are all within the bound passes the gate, whatever their tolerance. The tests
(`tests/visual.rs`, the visual gate) run `ux-browser values` on the documents of the recorded runs.

**Determinism**, measured 2026-11-27, Chrome/149.0.7827.55, `tests/fixtures/visual/visual.checks.yaml`
(one viewport 240 x 160, light and dark, full page, mask `.clock`; 4 pages, 16 shots per run):

| runs | result |
|---|---|
| the base site twice, 2 s apart | 12 of 16 shots the same bytes. The 4 others are `/clock-bare/`: a clock that prints the time to the millisecond, unmasked; its text is the time, so its pixels differ by design. The same clock masked (`/clock/`) gives the same bytes |
| the changed site against the base run as baseline | `/moved/` 3200 px per unit (the block moved 40 px: 40 x 40 uncovered, 40 x 40 covered), `/colour/` 3200 per unit at threshold 6 (red 0x33 to 0x3a on an 80 x 40 block), `/clock/` 0, `/clock-bare/` 1700 px; 16 checks, 4 passed, 12 failed; `diff/` 12 images |
| the same with `--visual update`, then again | 16 shots written to `baseline/`; the next run 0 on `/moved/`, `/colour/` and `/clock/`, `/clock-bare/` differs again; 12 of 16 passed |

The same bytes need the same pixels, and those need the same Chrome build, fonts and host: drawn in
software (`--disable-gpu`), a page renders the same twice on one host. Another Chrome build, another
font set or another host may draw text with other edge pixels; pin the build (`chrome`) and keep the
baseline of one host image, or allow for it with `threshold` and `share`. Not covered: a page that
moves on its own without an animation the browser knows (a script on a timer, a canvas, a video) is
not stilled; mask it. `--jobs` changes no shot: each page and viewport is measured in a browser
context of its own.

The tests (`tests/visual.rs`) start no browser: they read the shots and captures of the recorded
runs (`tests/fixtures/visual/base/`, `changed/`, the two sites beside them) and hold the
configuration, the comparison, the diff images, the numbers, findings and coverage, the values file
and the PNG reader and writer to them.

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
| `values` | per page scenario (`scenarios`) an object: per number of the document its output without the `@<version>` suffix (`layout.page_scroll`) and its value. A number without a value (incomplete) is left out; a page without a scenario has none. Without `gating`, the document's value: a sum over viewport x mode, a layout shift the largest over the viewports, `a11y.contrast_min` (a name that ends in `_min`) the smallest. With `gating`, the same sum, largest or smallest over the gating viewports only, from `details.viewports`. A state of a journey counts under the scenario of its page (`details.page`), its numbers named `<output without suffix>#<journey>/<state>` (`layout.overflow#open details/opened`); a state some run did not reach has no value there |
| `units` | per page, viewport and mode that has a gate failure, `{path, viewport, theme, failures}`; `theme` is the mode. A state's `path` is `<page>#<journey>/<state>`. Sorted by path, viewport, mode |
| `units[].failures` | `{gate, detail, key, waivedBy}` each, one per key, sorted by key. `waivedBy` is the `id` of the first finding record (in the order of `waivers.files`) whose status is not closed and one of whose `waives` patterns matches the key; else `null` |
| `incomplete` | `<page> <output>: <cause>` per result that is incomplete, in the document's order |
| `unmapped` | the pages of the document without a page scenario, sorted, each once |
| `viewports` | only with `gating`: per viewport `{gating, pages, <output>: <value>, ...}`: `pages` the number of pages with values at it, each count summed over them, each layout shift the largest, `a11y.contrast_min` the smallest |

A failure is a finding of an output named in `gates`, at a gating viewport. Its node path
`<viewport>/<mode>/<what>` gives the unit, and `<what>` its detail:

| output | detail |
|---|---|
| `layout.page_scroll@1` | `page-scroll` |
| `layout.overflow@1` | `overflow:<block>` |
| `always_visible.misses@1` | `<item>:<reason>@<top \| bottom \| state>`; `<item>:missing` for a missing item |
| `keyboard.misses@1` | `keyboard:<check>` (The keyboard walk: `keyboard:tab-reaches-<target>`, `keyboard:focus-mark-<target>`, ...); `skip-link-moves-focus` is left out in a unit where `skip-link-first` failed, as it follows from it |
| `visual.diff_share@1`, `visual.diff_px@1` | not a finding: a unit over the gate's bound, `visual.diff_share:<shot>` (`visual.diff_px:<shot>`), or not run, `visual.diff_share:<shot>:not-run` (Visual ratchet, A hard gate) |

The key: `gate<N>|<page>|<viewport>|<mode>|<detail>`, with every `|` and `,` of the detail a space,
so that a key splits on `|` and a `waives` line on `,`. llm's `core/findings.ts` writes the same form.
A finding of a journey's state (node path `journey/<journey>/<state>/<viewport>/<mode>/<what>`) has
`<page>#<journey>/<state>` in place of `<page>`:
`gate3|/menu/#open menu/opened|400x300|light|brand:covered@state` (`tests/journey.rs`).

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

### The run's times

`run.timing` (outside the evidence) says where a checks run spent its time: `unit` (`ms`) and
`pages`, one entry per page and viewport measured, in the run's order, one per line of the document.
Whole milliseconds of the worker's clock; workers overlap, so the entries add up to more than the
run's wall time.

| key | what |
|---|---|
| `path`, `viewport` | the page and the viewport |
| `loads` | the page loads of the pair: the measured one and each walk's (`measure::planned_loads`, plus one per activation and one where a reset found focus on the page) |
| `open` | the browser context and its page, created and set up |
| `load` | the measured load: navigation to the `load` event, then the ready condition, or fonts and two frames without one |
| `settle` | the `settle_ms` wait |
| `modes` | per mode, in order: `mode`; `switch` (the media switch, the wait for `mode_attribute`, `mode_settle_ms` and the check that the mode applied; for the first mode the check only); `overflow` (the overflow probe); `top` (the always-visible items at the top: the accessibility tree and each candidate); `scroll_settle` (the scroll to the bottom and `scroll_settle_ms`); `bottom` (the items at the bottom and the scroll back); and `keyboard` in a keyboard mode |
| `modes[].keyboard` | `ms` the whole walk; `reset` the reset of the measured page when the walk started on it (The keyboard walk, A fresh page); `loads` each load of the walk; `tabs` its Tab presses (the skip link's, the walk's, each activation's) |
| `close` | the context disposed |
| `total` | from the start of `open` to the end of `close` |

A page that could not be measured has no entry; journeys are not timed. A capture
(`--capture`) holds the same times under each viewport's `timing`.

### The walk on the measured page, 2026-11-26

Before: 0.11.0 (every fresh page of the walk a load); after: this change (the first walk starts on
the measured page, The keyboard walk). The same Chromium (Chrome/149.0.7827.55), the same builds, on a
shared 20-CPU workstation with other jobs running (1-minute load 6 to 16). Two sites: this
repository's documentation build (15 pages, `tests/fixtures/checks/uxui-docs.checks.yaml`: a skip
link, the walk in dark) and llm's (27 pages, `examples/llm.checks.yaml`, the same walk).

| site | `--jobs` | page loads before / after | wall before (s) | wall after (s) |
|---|---|---|---|---|
| uxui docs, 30 pairs | 4 | 90 / 60 | 15.3, 16.6, 15.8 | 15.2, 14.9, 20.0 |
| llm docs, 54 pairs | 4 | 162 / 110 | 30.4, 30.1, 27.9 | 26.2, 34.5, 27.0 |
| llm docs, 54 pairs | 1 | 162 / 110 | 122.1, 91.6 | 104.4, 104.3 |

Loads: 2 per pair after, where 3 before, except llm's `/llm/search`, whose query box takes focus on
load: the reset finds focus and loads, so it stays at 3 (2 pairs). Results: every run's evidence,
before and after, is the same bytes (llm 164 lines, 1,234 checks; uxui 92 lines, 566 checks), and so
is each fixture site's (`cover`, `hidden`, `journey`, `layout`, `narrow-menu`; none walks).

The wall times move with the host's load more than with the change. `run.timing` of the
one-worker runs says what the change is worth: a walk's load took 166 ms on the mean, the reset that
replaces it 28 ms, so the 52 loads saved are about 7 s of the run's 104 s of pair time (7 %). Where
the rest goes, per pair (mean of 54, one worker): 1,920 ms in all; the declared waits 250 (settle),
410 (two scroll settles), about 165 (the mode switch with its 100 ms settle) and the keyboard walk's
100 ms focus settle; the walk itself 460 (its load, 9 Tabs with a probe per target after each); the
measured load 235; the context 105. The waits a configuration declares are close to half of each
pair; they, and `--jobs`, are what moves the wall time further.

## Comparison with llm, 2026-11-24

llm's documentation site built in a worktree of `llm` at `58e27a4bb` (`npm --prefix website ci`,
`npm --prefix website run build`, as llm's `pages.yml`), llm's declarations (1440 x 900 and 390 x 844,
light and dark, navbar, search, theme toggle and sidebar, keyboard targets search and navbar, 40 Tabs, 250
ms). Chromium 158.0.8127.18 (Playwright revision 1254) for both: ux-browser starts `chrome
--headless=new`, llm's Playwright 1.65.0 its headless shell of the same version.

ux-browser: 27 pages, 1,234 checks, 107 s at PR #11 (one pair at a time; now: Timings). Document:
`engine/browser/examples/llm-docs.evidence.json`, rewritten by 0.3.0's run on a rebuild of the
same commit: the same 1,234 values, the new target kind, another build digest. Rewritten again on
2026-11-25, when documents began to name their ready condition (`ready: load`), by a run on a build of
llm `6df5102cc`: the same 135 numbers and findings and the same 1,234 checks; per-viewport details
on every number, as written since 0.3.0. Rewritten again when the layout rule became configuration
(Configuration, `layout`), by that change's run on the same build: the same 135 numbers and 1,234
checks (1,176 passed, 4 failed), and the rule values in its conditions. Rewritten again when
documents began to name their clock (`clock: real`), by that change's run on the same build with
Chrome/149.0.7827.55, the newest Chromium left on the host (158's full build was no longer in its
Playwright cache): the 135 numbers with their per-viewport details, the findings, the coverage
(1,234 checks) and the build digest equal to the document before; only the conditions and the run's
times differ.

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
