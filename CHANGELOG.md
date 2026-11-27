# Changelog

## Unreleased

- The documentation site passes its own page checks at phone width and at the bottom of a page. The
  theme toggle stays in the navbar at 390 px, in view and reached by the keyboard, where the theme
  kept it only in the collapsed menu; the docs sidebar is fixed to the viewport's height, so the
  footer no longer pushes its top out of view (222 px on the metrics page at 1440x900), and a docs
  page's footer starts beside it. The rules are `website/src/css/site.css`; on a local build
  `ux-browser checks` with `uxui-docs.checks.yaml` went from 103 failed of 566 checks to 0 (always
  visible and keyboard, both sizes, both modes, every page), with no overflow or page scroll. No
  gate or CI step runs these checks yet (`website/README.md`, Layout). Closes
  `story:docs-site-phone-and-sidebar`.

- ux-search campaign run has a live view: with `--events <dir>` it appends, while it measures, one
  JSON line per finished variant to the candidates file `<dir>/hill-climb.jsonl` (`iteration`, `at`,
  `kind` `baseline` for the page as shipped, `change` the variant and its axis values, `accepted` on the
  frontier so far, `score` its place in the ranking so far with 1 the best, `score_keys`, `metrics`,
  `deltas` and `worse` against the page as shipped, and with `--shots` a screenshot per environment
  taken as it is measured) and one per phase to the phase file `<dir>/hill-climb-events.jsonl`
  (`start`, `measuring`, `ranking` with the top three so far, `end` or `failed`), for a live page that
  polls two JSON-lines files. The ranking so far is the report's `rank` code, so the last ranking line
  names the report's top three. The measurements file is the same bytes with and without it; a run
  restricted to one variant or one environment refuses it. `docs/targets.md`, The live view. Closes
  `story:campaign-live-view`.

- `tools/release-ready.sh` counts a required workflow as green when one of its runs on the commit
  ended `success` and the others ended `success` or `skipped`; a skipped run beside a success is
  printed as not counted, and a workflow whose only runs are skipped is still red.

## 0.12.0 — 2026-11-27

- Releases carry `ux-search` for x86_64-unknown-linux-gnu, packaged with its declaration
  `ux-search.assessor.yaml` like `ux-browser` and `ux-static`; a consumer runs `ux-search campaign`
  without building it. `tools/release-versions.sh` holds its `--version` answer to the release.
- ux-browser checks holds a page to its design tokens: an optional `tokens` section of
  `ux-browser-checks/1` declares the font-size scale (a list of sizes, or a base and ratio), the
  colours (custom properties read from the page's root in each mode, `--prefix*` for a family, and
  values), the spacing unit and the allowed radii (`full` for a pill). Per page, viewport and mode
  five new numbers: `proxy.type_scale@1`, `proxy.colours@1`, `proxy.spacing_units@1` and
  `proxy.radii@1` count the distinct font sizes, colours of text, backgrounds and borders, gaps
  between sibling boxes and paddings, and radii off the tokens, each within a stated tolerance;
  `a11y.contrast_min@1` is the lowest WCAG contrast of text on its known background. Each names its
  offenders (node and value, at most 20) in `details.offenders`. A kind not declared, or a property
  the page does not define in a mode, is not run there (`not_run`, the number incomplete with
  `coverage_below_plan`), never a pass. The values file takes the five numbers; a number named
  `*_min` folds to the smallest over the gating viewports. Without the section the captures,
  documents and values files are the bytes they were. `docs/browser-adapter.md`, Design tokens;
  `docs/metrics.md` § 11.1. Closes `story:ux-browser-design-tokens`.
- Campaign terms for a list page, in the declaration format `ux-search-campaign/2` (`/1` is still read,
  and a `/1` file that uses a key or a term of `/2` is refused with the name it needs):
  `rows_in_view` counts the rows of the selector `rows` fully in view at the top of the page, the one
  term where more is better; `primary_bottom_px` is the bottom edge of the primary action, so an action
  centred in a header that only gets smaller does not rank worse, as it does on `primary_px`; an
  `always_visible` item names the scroll `container` that clips it on purpose and is then no miss. The
  measurements gain the two columns as `ux-search-campaign-measurements/3`; a `/2` file is read and
  written again in its own format, so the recorded studio campaign stays as it is. The browser lane
  holds the three on a list page of its own (`site/rows.html`). Closes
  `story:campaign-terms-for-list-pages`.
- ux-browser checks has a visual ratchet: an optional `visual` section of `ux-browser-checks/1`
  names a directory (`baseline/`, `shots/`, `diff/`), whether the full page is shot besides the
  viewport, mask selectors whose boxes are blanked in every shot, and a tolerance (a share of
  differing pixels and a per-channel threshold) with rules per page and viewport. Per page,
  viewport and mode the run stills the page (caret hidden, animations finished or paused, scroll 0,
  fonts settled), takes its shots and compares each with the baseline PNG of the same name; two
  new numbers per page, `visual.diff_share@1` (the largest share over its units) and
  `visual.diff_px@1` (the differing pixels, summed), each unit over its tolerance a failed check
  and a finding, and a diff image of the changed pixels in `diff/`. A unit without a baseline is
  `not_run` (the numbers incomplete), never a pass; a shot of another size fails naming both sizes.
  `--visual update` writes the run's shots as the new baseline; nothing else does. The values file
  takes both numbers; a share folds to the largest over the gating viewports. PNG files are read
  and written by the crate itself (`src/png.rs`), no dependency. Without the section the captures,
  documents and values files are the bytes they were. `docs/browser-adapter.md`, Visual ratchet;
  `docs/metrics.md` § 6.1. Closes `story:ux-browser-visual-ratchet`.
- `ux-search campaign run` reads each mode with an attribute or an expectation back on one page load
  before the first variant is rendered, so a mode the page does not take is refused at once and not
  after the renders of the modes before it. It names a mode or a viewport whose renders measure the
  same numbers as another one's for every variant. `--variant <id>` and `--environment "<W>x<H>
  <mode>"` restrict a run to one variant or one environment: it prints the numbers of each render and
  writes them as `ux-search-campaign-renders/1`, which `campaign report` refuses to rank. A shot is
  taken with the caret hidden: a focused field's blinking caret made two runs write different bytes.
  The refusal of a mode's attribute says that the page changed the attribute after it was ready.
  `campaign report --notes` prints an element once where it is the same: once for every variant when
  each names it in every environment, else once per variant with its environments. Decided:
  `text_under_14` stays a share, and a declaration that gains a key gets a new format name
  (`docs/targets.md`). Closes `story:campaign-run-follow-ups`.
- ux-browser checks writes where it spent its time: `run.timing`, outside the evidence, holds per
  page and viewport the milliseconds of the browser context, the measured load with its ready
  condition, the settle wait, and per mode the switch, the overflow probe, the always-visible items at
  the top, the scroll settle, the items at the bottom and the keyboard walk (its reset, each of its
  loads, its Tab presses), with the number of page loads. The evidence of the same measurements is
  the same bytes. `docs/browser-adapter.md`, The run's times.
- The first keyboard walk of a page and viewport starts on the page the checks just measured, which
  they only scrolled and read: with no element focused and nothing selected it is scrolled to the top
  and taken as fresh, else loaded. The Tab walk after the skip link's Enter, each activation and a
  second walk still load the page. llm's configuration (a skip link, the walk in dark) loads each page
  and viewport 2 times where 0.11.0 loaded it 3 times. `docs/browser-adapter.md`, The keyboard walk.
  Measured on llm's documentation build (54 pairs) and this repository's (30): loads 162 to 110 and
  90 to 60, the evidence of every run the same bytes as 0.11.0's, and on each fixture site; the
  walk's saved loads are about 7 % of a run's pair time, within the wall time's noise on a loaded
  host. Per pair the declared waits are close to half the time (`docs/browser-adapter.md`, The walk
  on the measured page). Closes `story:ux-browser-run-timing`.
- The documentation landing page ships the owner's pick from the frontier of the landing-page
  campaign: `docs-system.left.1+1.features-first.d85.h85`, applied by `ux-search landing promote`
  (`docs/promotions/2026-11-26-docs-landing.evidence.json`, with the copy of the measurements the
  pick was made on). One change against the page before it: `--uxui-density` 1 to 0.85 in
  `website/src/css/landing.css`, the primary action 24 px higher (472 to 448 px); the page ranked
  35th of 144, off the frontier, and ranks 11th, on the frontier of 16. The campaign is recorded
  again on the new sources (`docs/campaign-2026-11-26-docs-landing-engine.md`, The promotion of
  2026-11-26); the site the campaign builds a second time is of
  `studio.centred.1+3.status-first.d115.h100`, since the old one shared the density 0.85 with the
  page as shipped. Closes `story:docs-site-first-climb`.
- The specification's five not-found outcomes are reachable (`ess/GAPS.md`, G20, closed in the
  model): an unknown state-space key or promotion id now answers `StateSpaceNotFound` or
  `PromotionNotFound` where it answered a state conflict. `ess verify conform synthesize --path ess
  --scenarios ess --suite-format 5` exits 0: 286 refusals, all 286 covered by the 50 authored
  scenarios.
- The second round of the `ui-iteration-loop` skill on the studio is on record
  (`ess/ui/rounds/R-02/`). It builds three changes of R-01's change set, each with a check that
  failed before it: the five list filters of `ess/ui/studio.ui.yaml` match their rows (UXF-003), all
  small text of `themes/studio.css` is 14 px and a toned badge has a text colour per mode
  (UXF-017), and the gate decision of a promotion is a list row with the Owner's actions beside the
  verdict (UXF-001, the yes side of `R-02/decision.md`, which has no answer yet). Against R-01:
  scenarios not completed 6 to 5 of 11, missed expectations 22 to 16, looks 16 to 12,
  always-visible misses 24 to 20, text under 14 px 0.564 to 0, low-contrast runs on the worst page
  12 to 0. One value is above R-01's: the largest layout shift, 0.0151 to 0.0176, under the skill's
  tolerance of 0.01 and unstable from run to run on unchanged input (new finding UXF-023). Gates 1,
  2 and 3 still fail; 20 findings are open, 2 of them S1. New: `ess/ui/rounds/studio.audit.json`
  (bars for the page audit, counted into `audit.cells_over_a_bar`), and `round.sh` gives only its
  two compiling steps to a build pool and measures where it was started.
- `ux-search campaign` runs a browser campaign whose target is a declaration file, so a page of
  another repository is climbed without a change to the engine. The declaration
  (`ux-search-campaign/1`) names the page and its ready selector, the environments (viewports;
  modes by media, attribute or query), the axes (each value a list of custom-property values per
  CSS selector, the shipped value named), the primary action, and the hard rules and ordered terms
  by name from the engine's catalogue. `ux-search campaign run --declaration <file> --url <origin>
  --out <measurements.json> [--shots <dir>]` renders every variant in every environment as an
  injected style sheet and writes `ux-search-campaign-measurements/1`; `ux-search campaign report
  --measurements <file>` ranks them with the landing campaign's ranking code and prints the
  frontier, each frontier variant's patch and the shipped variant's rank. A run fails when the page
  without the sheet differs from the shipped variant, when a set property is not read back, when a
  selector matches nothing, and on any request that is not `GET` or `HEAD` on the page's origin.
  An unknown key, rule or term, an empty axis and a property set twice are refused with the file
  and the key path. Proven on the frontier page of the generated studio: 48 variants, 192 renders,
  two runs with the same bytes; `tools/studio-campaign.sh` records it. On the theme of round R-02
  the page as shipped has no low-contrast text and no text under 14 px, so only the spacing axis
  separates the space: a frontier of 12, the page as shipped at rank 36 of 48
  (`docs/campaign-2026-11-26-studio-engine.md`, with the recording on the theme before the round:
  a frontier of 3, rank 42). There is no promotion step for a declared campaign yet. A change of
  that document runs the gate (`tools/prose-only.sh`).
- `website/data/status.json` and the limitations page no longer say the specification has no
  scenarios.
- The third round of the `ui-iteration-loop` skill on the studio is on record
  (`ess/ui/rounds/R-03/`). Hard gate 2 passes for the first time: the header of a page stays at the
  top of the viewport and the narrow column under it while the main column scrolls
  (`themes/studio.css`; UXF-004, the yes side of `R-03/decision.md`, which has no answer yet), so
  Stop and the campaign state stay in view on the live climb, always-visible misses 20 to 0. The
  switch action of a variant reads "Switch" and can be clicked at 1280x720 (UXF-002); four pages
  get a title (UXF-008, held by the new floor `h1` of `studio.audit.json`); the gate decision of a
  promotion and the dialog that accepts its trade-off read as words, one field per member of each
  record (UXF-005, for that page only). Against R-02: scenarios not completed 5 to 4 of 11, runs
  failed 2 to 0, looks 12 to 8, pages without a title 4 to 0. Not built: a notice still names its
  command (UXF-010); the generated runtime writes those words and `ess-ui/1` has no key for them
  (`docs/ess-gaps.md`, "A notice's words"); the round counts it, 28 of 44 runs. One value is above
  R-02's: the largest layout shift, 0.0176 to 0.0284, 0.0008 past the skill's tolerance and
  unstable on unchanged input (UXF-023). Gates 1 and 3 still fail; 20 findings are open, none S1.
- `ux-search campaign run` parks the pointer before it measures and shoots (`page.pointer`, else
  the bottom-right corner of the viewport): a headless pointer rests at 0, 0, and a page with
  something there that opens on hover was measured open. A mode can say what the page shows in it
  (`expect`: an attribute of the root element, a selector), read back in every render; a mode set by
  a query without it is named as not read back. The run says per axis how many values change a box
  or a computed style of the page, and names an axis that changes nothing. Each frontier variant is
  shot in every mode. `ux-search campaign report --notes` names the first elements behind the
  counts; text of alpha 0 is not judged for contrast. Every command answers `--help`. Without
  `--profile` the browser profile is a directory under the system's temp directory, removed when
  the run ends. The measurements format is `ux-search-campaign-measurements/2`; a `/1` file is
  refused. The tests that run a campaign in Chromium (`campaign_live`, 11) are not in the gate,
  which has no browser; `tools/studio-campaign.sh` runs them before it records. The studio campaign
  is recorded on the theme of round R-03: the same ranking, a frontier of 12, the page as shipped
  at rank 36 of 48.
- The first campaign on a page outside this repository ran: one list page of an admin console in a
  private repository, on its fixtures (`story:trial-target`, roadmap step 6). 32 variants, 192
  renders, two runs with the same bytes, a frontier of 6, the page as shipped at rank 28 of 32. The
  first run measured the page with a navigation open under the resting pointer; the fixes above
  came from it, and the second run measured the page at rest.
  `docs/trial-2026-11-26-outside-target.md` holds the generic findings; nothing of the target is in
  this repository.
- A release is one command and changes versions only (`story:release-changes-versions-only`).
  `sh tools/release-set-version.sh <version>` (`task release:set-version -- <version>`) writes the
  version into every file that carries it: the plugin manifests and the Claude marketplace, the
  crates and this repository's packages in both lock files, the plugin version of
  `website/docs/index.md` and `website/data/status.json`, and the current release in
  `docs/roadmap.md`. It moves the entries of `## Unreleased` under the dated heading, regenerates
  the pages and ends with `tools/release-versions.sh`, which now checks each of those files and
  names every one that disagrees. `sh tools/gate.sh` passes on what the command wrote, with no
  browser run and no golden file blessed. Before it writes a file the command refuses a release
  without an entry, a version that is not greater than every dated section and a date that is no
  day of the calendar. Run again for the version it just cut, it moves later entries into that
  section; run for the version before, it takes a cut that is not released back, byte for byte.
  A recording keeps the tool version it was recorded with: no release rewrites a recorded campaign
  or the two example documents of `ux-browser`, and the tests hold a recording's adapter line to
  the form `ux-browser <version>`, not to this build. For that, the landing page's eyebrow no
  longer names the plugin version; the documentation landing campaign was recorded once more for
  it (`words` 2 lower in all 864 renders, `primary_px` 14 or 15 px lower at 390x844, the same
  frontier of 16 variants, the page as shipped still rank 35). The golden evidence of `ux-static`
  keeps `{version}` in place of the tool version. `tools/docs-landing-campaign.sh` records in
  place (measurements, report, golden evidence, screenshots), builds the variant's site in a copy
  of `website/`, and runs as one job of a build pool with `--pool`.

## 0.11.0 — 2026-11-26

- `ux-search landing promote` makes one variant of the landing-page campaign the page as shipped:
  it applies the variant's patch to `website/product.json` and `website/src/css/landing.css`, names
  the variant in `engine/search/src/landing/shipped.txt` (the one place the shipped variant stands
  now, read at build time; the `shipped` field of the axes is gone) and writes one
  `ratchet-evidence/1` document: the SHA-256 of the measurements and of each source before and
  after, the adapter and browser, the chosen and the shipped variant with rank and terms, the size
  of the frontier, the patch, and who chose. It copies the measurements it read beside the evidence
  (`<name>.measurements.json` for `<name>.evidence.json`), and the evidence names that copy and the
  two sources by their paths from the working directory, however they were written; a file outside
  it has no path, so the document holds no absolute one. The same input gives the same bytes. It
  refuses, and writes nothing, a variant outside the space or the measurements, one that breaks a
  hard rule or is off the frontier, the shipped variant itself, sources other than those the
  measurements were recorded on, a place that is a directory or has no file name, and one file
  named for two of its files by any spelling. The five files are written all or none: when a move
  fails, the files already in place get their bytes back, and the staged files and the directories
  made are removed. Without `--shipped-file` it says in one line that the shipped file was not
  changed, and names it. `engine/search/ux-search.assessor.yaml` declares `ux-search@1`
  (`ratchet-assessor/1`: inputs, outputs with scopes and units, coverage unit, conditions), and a
  test holds the command line and the evidence documents to it. Promoted evidence files live in
  `docs/promotions/`. Operations for actions, section order and palette stay written against
  `1+1`, `features-first` and `docs-system`; a promotion off those values prints a note. The tests
  of the promotion compute the variant they promote, so a promotion leaves them standing.
  `docs/campaign-2026-11-26-docs-landing-engine.md`, Promotion.
- `ux-search` builds on ESS 0.99.5, the newest release (was 0.99.4): the three ESS crates by that git
  tag, and `engine/search/Cargo.lock` moved with them. 0.99.5 changes no source of the crates it links,
  so both golden frontiers change in their first line only, which names the tag. `ux_search::ESS_TAG`
  is the one name of the tag in the source, and a test holds the manifest, the lock file, the reports,
  the fixture's provenance (every tag it names, and its commit against the lock file's) and
  `AGENTS.md` to it. `docs/ess-gaps.md` is read against ESS's releases up
  to 0.99.5 and marks what a release closed: item 9 of 18 (for all over related rows), by a construct
  ESS has had since 0.63.0 and the specification already uses; the other 17 items are open in 0.99.5.
  Two needs of the generator round are closed by ESS 0.98.1, and `uxui-gen` follows the first: a live
  `insert_top` of a key the section already holds shows one row (the held row takes the event's fields
  and moves to the top, `max_rows` after the move), where it inserted the key again; the loop period
  of a fixture script needed no change.
- The first round of the `ui-iteration-loop` skill on the studio is on record (`ess/ui/rounds/R-01/`):
  the React app generated from `ess/ui/studio.ui.yaml` on its fixtures, 10 pages at 5 viewports in
  light and dark, 11 scenario cards. Gates 1, 2 and 3 fail, 5 of 11 scenarios complete, and 22
  findings (4 S1, 12 S2, 6 S3) have a record each in `ess/ui/findings/`. Nothing is fixed in this
  round; `sh ess/ui/rounds/round.sh <R-nn>` runs the next one.
- The specification has 50 authored scenarios (13 before): 286 of the 291 refusals that synthesis
  cannot reach are covered by one (`ess verify conform synthesize --path ess --scenarios ess
  --suite-format 5`), and all 50 pass on the interpreted target. The 5 left are not-found outcomes
  no command sequence reaches (`ess/GAPS.md`, G20). `SourcePatch` is now the engine's patch (RFC
  6902 operations and custom-property values per selector) and `ScriptedStep` the browser adapter's
  journey step; sixteen refusing branches guard the entity invariants that had none. 19 UNMAPPED
  markers are left, each listed in `ess/GAPS.md`.
- `ux-search` gets a second backend, the browser adapter (`ux_browser`, by path), and runs its first
  campaign on it: the documentation landing page `/uxui/`, 144 variants (palette x hero alignment x
  hero actions x section order x density x heading scale) in 6 environments, 864 renders in 6 page
  loads. A variant is a patch to the page's sources: RFC 6902 operations on `website/product.json`
  and custom-property values of `website/src/css/landing.css`. `ux-search landing run` measures and
  writes `measurements.json` (`ux-search-landing/1`) and two screenshots per frontier variant;
  `ux-search landing report` ranks a measurements file: lexicographic terms, ties by id, the Pareto
  frontier first with each variant's patch. Measured in `HeadlessChrome/158.0.8127.18`: no variant
  breaks a hard rule, the frontier is 16 variants with 2 sets of terms, and the page as shipped ranks
  35th of 144. The run checks the probe's render against the built page and against a site built
  from one variant's patched sources. A golden file holds the report over the committed measurements;
  no test starts a browser. `landing.css` gains `--uxui-hero-align` and `--uxui-hero-lede-inline`,
  which change nothing at their shipped values. `ux_browser::VERSION` names the adapter's version.
  `tools/docs-landing-campaign.sh`, `docs/campaign-2026-11-26-docs-landing-engine.md`.
- `ux-search climb` climbs the studio: on `campaigns.climb` of `ess/ui/studio.ui.yaml` it permutes the
  sections inside each layout column (the terminal renderer draws the columns as one stack), 120 variants;
  the frontier of 42 trades a chart last in the main column (1 region below the fold, 2 cut) against the
  step table last (20 below, 0 cut); the page as written ranks 72nd. A golden file holds the output.
  Compact `columns:` lists and comments in `layout:` are read as YAML writes them. `docs/studio-climb.md`.
- The engine's first search: `ux-search` (`engine/search`, a crate with its own workspace, the ESS
  crates by git tag 0.99.4) climbs exhaustively over theme x section order of one page on layout
  model v0, ESS's terminal renderer in process. Each variant is checked, opened fresh and drawn at
  120x40 and 48x40; its terms (check errors, regions below the fold, regions cut, rows before the
  primary action) rank it lexicographically, ties by id, and the Pareto frontier prints first. On
  the ticket page of ESS's partner portal: 18 variants, a frontier of 9 with the send button at row
  30, against 46 when the conversation comes first; theme changes no v0 term (the renderer reads no
  style tokens). Two climbs print the same bytes, and a test holds them to a committed golden file.
  `ux-search cost` gives the time per UI state (open, steps replayed from open, geometry read) and
  the peak memory: `docs/cost-spike.md`. The gate runs its fmt, clippy and tests.
- `ui-iteration-loop`: a decision template that compares A (now) with B (change); a decision order
  (hard gates, open S1/S2 by `found_on`, ratchet per scenario, survey medians, reviewers, the person)
  that recommends and sets a default while the person decides, and B never lands with a worse key
  without a decision id; a survey rule; the sections each review agent writes.
- Themes: `themes/dense.css` (fixed columns, one-line cells, a label beside its value, a colour per
  value through `data-uxui-value`) and `themes/slot-jobs.css` for slot's job pages, with the steps
  slot takes in `themes/slot-jobs.md`; `tools/check-themes.sh` and its 25-case test run in the gate.
- `docs/ess-gaps.md` read again against ESS 0.99.4 and its main: the stories behind each gap, and a
  section "Needs per way" whose open needs ESS filed as stories on 2026-11-25.
- The studio's terminal tests read the budget as a map, the KPI values as a list and the step table
  in its declared order (7 tests).

## 0.10.0 — 2026-11-25

- ux-browser checks pages in states reached by steps, not only after load: a configuration may
  declare `journeys`, each a `page`, `steps` named by CSS selector (`click`, `type` with `text`,
  `press` Tab, Enter, Space or Escape, `wait` for a rendered match within `timeout_ms`, `scroll` top
  or bottom) and the `state`s at which page scroll, overflow and the always-visible items run on the
  page as the steps left it, per viewport and mode. Each state has its own results, scope `state`,
  item `<journey>/<state>`, node paths `journey/<journey>/<state>/<viewport>/<mode>/<what>`; a step
  that fails ends that run and its later states are `not_run` with the step and the reason. A
  journey that does not read is refused with exit 1 before the browser, naming the journey and the
  step. The values file counts a state under its page's scenario as `<metric>#<journey>/<state>`. A
  fixture page with a long line in a closed `details` counts overflow 0 after load and 1 in the
  state after a click on its `summary`; an opened menu counts its covered item in its state only. A
  configuration without journeys writes the document it wrote before. `docs/browser-adapter.md`,
  Journeys.
- When Chromium ends before it listens, or prints no DevTools endpoint within 30 s, the error (exit 3)
  holds its last lines on stderr, at most 20, below the status; up to 0.9.1 it held the status only
  (llm, run 45597987448: `signal: 6 (SIGABRT)` and nothing else). `docs/browser-adapter.md` says what a
  consumer does on a hosted runner that refuses Chromium's sandbox: the defaults keep it,
  `chrome_args: [--no-sandbox]` or allowing unprivileged user namespaces is the consumer's choice.
- ux-browser's cover test samples five points of an always-visible item, not its centre only: the
  centre and the four corners, each 1 px inside the box and further in along the corner's diagonal
  by r (1 - 1/sqrt(2)) for its used `border-radius` r, so a point stays inside a rounded corner. The
  item is covered when `elementFromPoint` at any of them is another element. An item under a sticky
  bar at one corner, or under a dialog short of its centre, now counts as covered; 0.9.1 counted it
  visible. The probe records the points that missed (`missed`) in a capture; the evidence is
  unchanged. `docs/browser-adapter.md`, The cover test, lists the points and the fixture's counts.
- The workspace is formatted (`cargo fmt --all`) and clippy-clean with `-D warnings` on every target;
  two lints in `uxui-gen` are fixed. `tools/gate.sh` runs both checks, so `task check:pool`, `Gate`
  and the release gate fail on a format diff or a lint; both gate jobs install `clippy` and
  `rustfmt`, and `tools/check-workflows.sh` fails a gate job that does not.

## 0.9.1 — 2026-11-25

- ux-static's two `--version` tests each run in a directory of their own. Both used
  `version/version`, and one removed it while the other ran `ux-static` in it: 0.9.0's release
  gate (run 45586279102) failed on that, after the same commit's `Gate` had passed.
- `tools/gate.sh` runs the workspace's test binaries 4 more times (`tools/repeat-tests.sh`), so a
  race between two tests shows in the local gate and in `Gate` more often than once.
- 0.9.0 was tagged on 659f93d and never published. 0.9.1 holds everything listed for 0.9.0 below
  it, plus this fix.

- `ux-browser --version` (and `-V`) prints `ux-browser <version>` and a newline and exits 0; it starts
  no browser and reads no file, and the version is the crate's, the one a run records as
  `evidence.conditions.tools.ux-browser`. A consumer that pins ux-browser can now prove a stored or
  `PATH` binary before it runs it. `ux-static --version` (and `-V`) answers the same way for
  ux-static. Both usages list the flag.
- `tools/release-versions.sh <version> <binary directory>` also checks that the built `ux-browser`
  and `ux-static` in that directory answer `--version` with exactly the release; the release
  workflow's package job runs it on the binaries it builds. Without the directory it checks the
  manifests, crates and changelog as before.
- The forge runs the release gate on every pull request and every push to `main`: the new workflow
  `Gate` (`.github/workflows/ci.yml`) and the release workflow's Gate job both run `tools/gate.sh`
  on `ubuntu-latest` with the toolchain of `rust-toolchain.toml`, and `task check:pool` runs the
  same script. It hides the browser, as the runner has none: 0.8.0's two `values::` tests that
  failed only after the tag (run 45545202319, 183 passed, 2 failed) fail the same way locally. A
  pull request of planning and prose files only (`tools/prose-only.sh`) ends `Gate` green without
  building. `task release:ready -- <sha>` (`tools/release-ready.sh`) refuses a release tag, naming
  the run, while `Gate`, `Documentation validation` or `Documentation site` on that commit is
  absent, pending or not `success`; the release procedure runs it before the tag.

## 0.8.1 — 2026-11-25

- `ux-browser checks` refuses its inputs (exit 1) before it looks for a browser (exit 3): the
  configuration, the values section and its scenario files, the build and the pages. 0.8.0 looked
  for Chrome right after reading the configuration, so on a host without a browser a refused input
  ended with exit 3 and the browser's message. The tests of those refusals now run with no
  `UXUI_CHROME` and an empty Playwright cache, as on a CI runner, so the local gate fails the same
  way the release gate did.
- 0.8.0 was tagged on 5595daf and never published: its release gate failed on the two refusal
  tests above. 0.8.1 holds what 0.8.0 was to hold (the entries below) and this fix.
- The Chrome build of a run is pinned in its `ux-browser-checks/1` file: `chrome: {build: <n>}` takes
  the Playwright build `chromium-<n>`, `chrome: {version: <major>}` the newest build whose Chrome has
  that major version (read with the binary's `--version`, which starts no browser). A pin no build
  meets ends `ux-browser checks` with exit 3 after every refusal of the inputs and before a browser starts, naming the
  pin, the cache and the builds found (`found: chromium-1228 (149.0.7827.55)`). `UXUI_CHROME` still
  wins and is checked against the pin: a `build` pin by the binary's `chromium-<n>` directory, a
  `version` pin by its `--version`; a mismatch, or a `build` pin on a binary outside any build
  directory, is refused the same way. Without the key the newest build is taken, as before. The
  Chrome that ran stays `evidence.conditions.tools.chromium`; the pin is neither a condition nor a
  local input, so runs on the same Chrome have the same conditions however it was found. A value
  that is not exactly one of `build` and `version` with a whole number above 0 is refused with the
  configuration (exit 1).
- `layout.overflow@1` counts only content the page draws. The overflow probe skips an element for
  which `checkVisibility({visibilityProperty: true})` is false, and a text run whose `visibility` is
  not `visible` or whose nearest element with a box fails `checkVisibility()`, has
  `content-visibility: hidden` or is a closed `details` (text straight inside it): content of a
  closed `details`, under `content-visibility: hidden` or `hidden="until-found"`, or under
  `display: none`. A long line inside a closed `details` (the second consumer's `p.full` in
  `details.more`, "right edge 895 past its parent at 892") is no longer overflow; the same line with
  the `details` open still is. Text in a `display: contents` element and content under
  `content-visibility: auto` below the fold still count (`docs/browser-adapter.md`, Content the
  page does not draw).
- An always-visible item may set `max_width`: it is checked only at viewports with `min_width <=
  width <= max_width`, and at a wider one it is not measured and counts nothing, as below its
  `min_width`. A `max_width` that is not a whole number above 0, or that is below the item's
  `min_width`, is refused with the item's name. An item a page shows only on narrow viewports (a
  phone menu button) is declared with it: on the fixture page `tests/fixtures/checks/narrow-menu-site/`
  "Open menu" with `max_width: 1023` counts 0 misses at 390 and at 1440 px, where 0.6.0 counted the
  item missing at 1024 and 1440 px (2 each, top and bottom). The keyboard walk is unchanged: it
  reaches for its targets at the viewports of `keyboard.viewports`. A configuration without
  `max_width` gives 0.6.0's document: llm's documentation build, all 162 results equal to
  `engine/browser/examples/llm-docs.evidence.json` byte for byte.

## 0.7.0 — 2026-11-25

- No GPU process by default: Chromium starts with `--disable-gpu` and `--in-process-gpu`
  (`--disable-gpu` alone keeps a `--type=gpu-process` for compositing). A consumer adds its own
  switches after the defaults with `chrome_args: [...]` in a `ux-browser-checks/1` file and the
  repeatable `--chrome-arg <switch>` (or a JSON list of switches, as ratchet passes a list input). A
  switch the defaults set with another value (`--headless=old`, `--remote-debugging-port=9222`) and
  `--user-data-dir` are refused by name before a browser starts (exit 1 from the file, 2 from the
  flag); `--enable-gpu` undoes the default, starting Chromium with the earlier switches. The added
  switches are `run.local_inputs.chrome-arg`, a JSON list outside the evidence, and the assessor
  declaration lists the local input `chrome-arg`. On the second consumer's snapshot the default took
  the run from 86.8 to 56.3 CPU seconds (its GPU process had peaked at 178 %), with the same 55 exact
  results. On llm's documentation build 3 of 5 runs gave `examples/llm-docs.evidence.json`'s evidence
  byte for byte, which stands; 2 read `layout_shift.after_ready` of `/llm/docs/concepts/protocols` at
  1440 x 900 as 0 instead of 0.0004 (a web font's swap; 0 of 6 runs with the earlier switches).
- Limits: with the new default, 2 of 5 runs on llm's documentation build read one sampled value,
  `layout_shift.after_ready` of `/llm/docs/concepts/protocols` at 1440 x 900, as 0 instead of 0.0004
  (0 of 6 runs with the earlier switches). A consumer whose ratchet holds layout shift at 0 tolerance
  can add `--chrome-arg --enable-gpu` to keep the earlier switches. This entry was merged under
  0.6.0 by mistake on 2026-11-25 and is moved here; 0.6.0 does not contain it.

## 0.6.0 — 2026-11-25

- The keyboard walk of `ux-browser checks` works without a skip link: `keyboard.skip_link_is` and
  `skip_link_contains` are optional, and without them the walk has no skip-link checks (and loads the
  page once, not twice); it checks that up to `max_tabs` Tabs reach each `must_reach` target. On the
  second consumer's snapshot (a control plane web application without a skip link) that takes
  `keyboard.misses@1` from 4 to 0 on each of its 11 pages, the value its own keyboard rule has.
- `keyboard.focus_mark: true` checks, on the Tab that first reaches each target, that the focused
  element shows a focus mark: it matches `:focus-visible` and draws an outline (a style, a width
  above 0, a colour not fully transparent) or a box-shadow (`probes/focus-mark.js` reads the computed
  style, `checks::focus_marked` decides). A reached target without one is a miss,
  `focus-mark-<target>`, whose reason names the element and what it showed.
- `keyboard.activates: [<target>, ...]` (targets of `must_reach`) checks that Enter, or else Space, on
  a reached target acts within `focus_settle_ms`: the page navigated (the browser's navigation
  events, a new hash included), focus moved, the focused element's `aria-expanded`, `aria-pressed`,
  `aria-selected`, `aria-checked`, `checked`, `value` or `open` changed, or an element was added or
  removed (or `hidden`, `open`, `aria-hidden`, `disabled` changed) on a node that did not change in a
  quiet watch before the key. A miss is `activates-<target>`. A change of text only, a new window or
  a change after the wait is not seen (`docs/browser-adapter.md`, The keyboard walk).
- `keyboard.misses@1` keeps its name; its findings are `tab-reaches-<target>`,
  `focus-mark-<target>`, `activates-<target>` and, with a skip link, the two skip-link checks, and
  `details.walks` names only the checks the walk had. A configuration of 0.5.0 gives 0.5.0's
  document: llm's documentation build, all 162 results equal to
  `engine/browser/examples/llm-docs.evidence.json` byte for byte; the second consumer's 0.3.0
  configuration, its 55 exact results equal to its 0.3.0 document.
- A fixed page clock: `clock: {at: <RFC 3339 instant>, advance: true | false}` in a
  `ux-browser-checks/1` file. A script before the page's own sets `Date` (`Date()`, `new Date()`,
  `Date.now()`) to the declared instant plus the time since the document's time origin, or to the
  instant alone with `advance: false`; `performance.timeOrigin` is the instant, and
  `Intl.DateTimeFormat`'s `format()` and `formatToParts()` without a date and `Temporal.Now` read
  the same clock. Timers, frames and `performance.now()` run as before, so the ready wait, the
  settle times and the layout-shift times do not change. An instant without a zone, with a leap
  second or with more than 3 digits of fraction is refused. Two runs of a fixture page that prints
  the time (`tests/fixtures/clock/`) give equal geometry with the key and different geometry
  without it. Every document names its clock in `evidence.conditions.other.clock` (`real` without
  the key, `from <instant>` or `frozen at <instant>` in UTC) and the assessor declaration lists
  `clock`; `examples/llm-docs.evidence.json` is rewritten by such a run with the same values.
## 0.5.0 — 2026-11-25

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
