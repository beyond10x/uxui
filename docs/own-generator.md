# The smallest own generator

Story 4 of the round in [generator-round.md](generator-round.md). `generators/own/` is a Rust crate,
binary `uxui-gen`, that reads the studio document (`ess/ui/studio.ui.yaml`), its fixtures and the
ESS model it names, and writes one page, the live climb (`pages/campaigns.climb`), as static HTML
and plain JS: no React, no build step. The page plays the fixture live script.

```console
cargo run -p uxui-gen -- --path ess/ui/studio.ui.yaml --model ess --page campaigns.climb --out target/climb \
  --theme themes/studio.css --theme themes/theme-toggle.js
```

It writes four files: `index.html` (the page skeleton: header, layout columns and section frames,
drawn before any read), `page.js` (the resolved page, the channels and the fixtures as data),
`runtime.js` (reads, live effects, drawing) and `style.css` (the document's tokens, then the
stylesheet), and copies each `--theme` file to `theme/`: a stylesheet loads after `style.css`, a
script in the head. Its custom properties and classes carry the stock app's names (`--ui-color-*`,
`ui-section`, `ui-series-N` …), so one uxui theme styles both ways, light and dark. Any static server serves the directory, and so does `file://`; the page reads its
params from the query: `index.html?campaign_id=cp-01`. Every drawn node carries `data-ui-path`
with its canonical node path, rows and cells included (`…/rows/<key>/columns/<name>`).

Model binding runs before anything is written: a view, event or command the page names that the
model lacks, a read param the view does not take, a column the view does not have, an input the
command does not take, is refused with its node path. It also warns where ESS does not (see
[gaps](#ess-gaps-found)).

## Dependencies

None. The crate has no third-party dependency; the YAML subset reader (`engine/doc/src/yaml.rs`) and the JSON
writer (`engine/doc/src/json.rs`, crate `uxui-doc`, shared with `ux-static`) are this repository's own. Considered and not taken:

- `serde` + `serde_yaml`: `serde_yaml` is archived and unmaintained, and depends on
  `unsafe-libyaml` (RUSTSEC-2024-0449); ESS itself moved to a fork (`serde_norway`) behind its own
  `ess-yaml` crate for that reason.
- The ESS crates (`ess-ui` loads a document into typed Rust, with shorthands, page kinds and
  widgets expanded and every node addressed): the workspace sets `publish = false`, so they are
  reachable only as a Git dependency on a tag, and their `Document` derives `Deserialize` but not
  `Serialize`, so even linked it hands no form over that a generator in another language, or a
  file on disk, could read.

The repository's direction is zero third-party dependencies; the reader covers what the studio and
the model use (block and flow collections, the three scalar styles, `|` and `>` block scalars,
comments) and refuses duplicate keys and open flow collections.

## What it had to re-implement

ESS resolves all of the parts below in `ess-ui` (`expand.rs` alone is 1,868 lines) and in each
renderer, and hands none of it over: `ess ui load` prints a summary line. Each part is fenced by
`PART:` markers in the source; the counts are non-blank lines.

| Part | Where | Lines | What it covers on this one page |
|---|---|---|---|
| YAML subset reader | `engine/doc/src/yaml.rs` | 493 | the documents themselves; with a JSON form a JSON reader replaces it (about 150 lines) |
| Defaults and short forms | `src/resolve.rs` | 144 | built-in page kinds and their merge (named lists by name, maps deep, `null` and `remove: true`), field short form, `name` from `field`, label from the name, action name derivation, inline `confirm:` expanded to its overlay, durations |
| Page resolution and node paths | `src/resolve.rs` | 117 | header (title `from_page`, `switch_to` labels), layout `columns` or the `stack` default, sections, reads (`key` defaulting to `live.match`, else `id`), empty-state text; the path of every page, header, action, confirm overlay, layout column, section, field and column |
| Tone maps | `src/resolve.rs` | 8 | `tone_by: <name>` to the document's map |
| Style tokens and themes | `src/emit.rs` | 91 | the built-in token table, merged under `tokens:` and the default theme, as CSS custom properties; type styles falling back to `body` |
| Model binding | `src/binding.rs`, `src/model.rs`, `typed` in `src/lib.rs` | 105 + 74 + 16 | the model reader (views, events, commands from `ess-inputs.yaml`), the checks above, each field's model type |
| Live and channel semantics | `src/resolve.rs`, `assets/runtime.js` | 40 + 94 | `live.match` default, `coalesce` in ms, `on`, `only_if: matches(params)`, the six effects, `max_rows`, the lifecycle with `stale_after` and `resume: refetch`, the script loop |
| Fixture semantics | `src/fixtures.rs`, `assets/runtime.js` | 58 + 21 | index, one-view and many-view files, `by_params`, `derived: by_id_from` and `same_as`, scripts with durations, params matching rows |
| Expressions | `assets/runtime.js` | 35 | paths, literals, `==`, `!=`, `not`, `and`, `or`, `matches(params)` |
| **Total** |  | **1,296** | of 1,807 non-blank lines (Rust, JS and CSS, tests excluded) |

The rest is the generator's own work and would be written on any way: the HTML skeleton (69 lines
of `src/emit.rs`), drawing records, tables, the chart, badges, the confirm dialog and the command log
(about 170 lines of `runtime.js`), the stylesheet (56 lines) and the command line (46 lines).

`ess ui load --format json` would remove the YAML reader, the defaults and short forms, page
resolution and node paths, tone maps and, if it carried the bindings, the model reader and binding:
about 960 of the 1,296 lines. Live, fixture and expression semantics would remain: they are
behaviour, not form, and only a conformance suite every generator can run (story 7) holds them.

### Decisions ESS left open

Each of these is specified nowhere, or the renderers disagree; the own generator had to pick. The
ESS column is 0.98.0 unless a row names a later release; ESS 0.98.1 settled the first and the third
row, and the own generator follows it.

| Question | ESS | Own generator |
|---|---|---|
| Period of a looping fixture script | 0.98.0: terminal and test clock: the last entry's `at` (`ess-ui-test/src/clock.rs`); React: the last `at` plus 1 s, at least 1 s (generated `runtime/live.ts`, `fixturePlayer`). From 0.98.1 both: the last entry's `at`, and a script whose last entry is at `0s` plays once | the terminal's, so `ess-ui-test` timings hold; unchanged by 0.98.1 |
| An event the script plays at `0s`, as the page opens | terminal: the step table loses it; React: the coalesced chart draws it, the step table loses it | every section receives it (reads from fixtures are synchronous) |
| `insert_top` of a key already held | 0.98.0: both renderers insert it again: two rows share one row path. From 0.98.1 both show one row: the held row takes the event's fields and moves to the top, and `max_rows` applies after the move | the rule of 0.98.1 (`apply` in `assets/runtime.js`); it inserted again while ESS did |
| `resume: refetch` in fixture mode | the fixture holds no live rows; React keeps live rows only because the re-read rows compare equal | the section keeps the events it applied and re-applies them over the re-read rows |
| Where `insert_or_patch` inserts | React: at the top | at the top |
| A confirm's button text when the short form names none | schema: `confirm_label` default `Delete`; React: `Confirm` | `Confirm` |
| A field's label | schema: `model.field_label`, which the model does not declare; renderers: the name with `_` as a space | the renderers' rule |
| A `by_id_from` view with no matching row | React: the source's first row | no row |

## Three changes on both ways

Items C1, C2 and C3 of [change-list.md](change-list.md). Stock: `ess generate ui --target react`
0.98.0 into a scratch directory, with the spec edits in a copy of `ess/`. Own: edits of
`generators/own/`, generating from the same edited spec. Minutes are the wall time of an agent
session from the first edit to the last, after the change was planned; finding where to change
(reading the stock runtime's `chart.tsx`, `styles.css`, `hosts.tsx` and the reference, about 10
minutes for the three) is not in them. Lines are lines added or changed in files a person keeps.

| Item | Way | Layer | Minutes | Files touched | Lines | `ess ui test` |
|---|---|---|---|---|---|---|
| C1 sort by seq | stock | spec + fork | 0.2 | `studio.ui.yaml`, `runtime/composites/chart.tsx` (fork) | +1; +8 −1 | 3 of 3 pass, and C1's step passes; the chart's order is not testable |
| C1 sort by seq | own | spec + own | 0.1 | `studio.ui.yaml`, `assets/runtime.js` | +1; +13 −3 | not run against the own page (see below) |
| C2 brand theme | stock | fork | 0.1 | `www/index.html` (fork) | +1 | 3 of 3 pass; a style is not testable |
| C2 brand theme | own | own | 0.7 | `src/emit.rs`, `src/lib.rs`, `src/main.rs`, `assets/style.css`, `assets/runtime.js`, `tests/main.rs` | +107 −80, of them 71 renamed names and 17 test lines | not run |
| C2 light and dark modes | stock | fork | 0.1 | `www/index.html` (fork) | +1 | 3 of 3 pass |
| C2 light and dark modes | own | own | 0.6 | `src/emit.rs`, `src/lib.rs`, `src/main.rs`, `tests/main.rs` | +32 −17; tests +13 −11 | not run |
| C3 KPI cell | stock | spec + slot | 0.3 | `studio.ui.yaml`, `src/host-components.tsx` (new) | 1 changed; +22 | 3 of 3 pass; C3's step cannot pass: the terminal draws the slot's reason in the cell |
| C3 KPI cell | own | own | 0.1 | `src/lib.rs`, `assets/runtime.js`, `assets/style.css`, `tests/main.rs` | +20 −1; +4; +1; +1 | not run |

What the rows do not show:

- **Forks are re-applied after every generate.** `ess generate ui` overwrites `chart.tsx`,
  `styles.css` and `www/index.html`. In this session the stock app was generated four times; each
  time the forks were re-applied from saved patches. A first version of C2 (spec `tokens:` for the
  colours and the font, plus a fork of `styles.css` for series 2 and 3, +4 and +4 lines) was lost
  on the next generate exactly so. On the own way nothing generated is ever edited.
- **C2 on the own way is a one-time cost.** The own generator now names its custom properties and
  classes as the stock app does (`--ui-color-*`, `ui-section`, `ui-table`, `ui-series-N` …) and
  takes `--theme <file>` once per file; every later uxui theme applies to both ways without a
  generator edit. The theme `studio` targets regions the own page does not have (navigation,
  shell); the rest applies.
- **Light and dark modes cost the generators almost nothing.** The theme carries both modes and the
  toggle script sets `data-uxui-mode`; a generator only has to draw every colour from a custom
  property (both do: neither stylesheet holds a colour literal) and load the script in the head.
  Stock: one more forked line. Own: `--theme` took a list and learned scripts. The one trap is on
  the stock way: its runtime writes `data-theme` on `<html>` (`runtime/theme.ts`), so a theme keyed
  on `data-theme` is overwritten.
- **C3 on the stock way leaves the change untested.** A host slot's content is the host's; the
  terminal runner draws the slot's `reason`, so the acceptance step fails there, and the two
  renderers can no longer give one verdict on that cell.
- **C3 on the own way keys on the model.** The own generator gives each field its model type and
  draws any `List<…KpiReading>` as chips, on every page, without a spec change. The stock way needs
  one host slot per place (`host` names are unique in a document), so the frontier page's identical
  column needs a second slot and a second registration.
- **C1 on the own way included building `sort`.** The stock collection already reads `sort`; the
  own generator had to implement it (8 of the 13 lines). Every construct the own generator draws is
  its own code.
- **Verification.** Own: `cargo test -p uxui-gen` (7 s compile, tests 0.4 s), generate (under
  0.1 s), then the page in a headless browser after 7 s of virtual time: the step table
  `[pt-05, pt-04, pt-03, pt-02, pt-01]`, the chart's x labels `1 2 3 4 5`, KPI chips, the channel
  `Live`; at 9 s `Reconnecting`; with the theme, both modes (system dark and light). Stock: generate
  (0.7 s), re-apply forks, `tsc --noEmit` (exit 0, 16 s) and an esbuild bundle (exit 0, 1 s),
  `ess ui test` (0.7 s), then the app served on a local port in the same headless browser: the step
  table `[pt-05, pt-03, pt-02, pt-01]`, the chart's x labels `1 2 3 4 5`, KPI chips from the host
  component, the theme in both modes. Each build waited for the shared build pool (6 to 16 minutes
  per job here); the waits are not in the minutes. Seen on the way: the stock chart's lines do not
  span the width its x labels take (its SVG keeps its aspect ratio).

`ess ui test` runs the terminal renderer built from the document, never the generated code: on the
stock way it does not see the forks or the host component either. Nothing runs the test files
against the own page yet; its `data-ui-path` attributes follow the paths the Playwright spec of
`ess ui test --playwright` selects, which is the next step to hold both ways to one test file.

## ESS gaps found

Each reproduces on `ess` 0.98.0 in this repository. ESS 0.98.1 fixed 1 and 3; 2, 4 and 5 are open
in ESS 0.99.5 (`docs/ess-gaps.md`, Needs per way).

1. **`insert_top` inserts a key the section already holds, so two rows share one row path.** Fixed
   in ESS 0.98.1: the held row takes the event's fields and moves to the top, in the terminal, in
   `ess ui test` and in the generated React app; the own generator does the same. On 0.98.0: with a
   looping script, every cycle inserts `pt-04`, `pt-05` and `pt-06` again; row paths
   (`…/rows/<key>`) stop naming one row, and `max_rows` fills with repeats. Reproduction:

   ```yaml
   format: ess-ui-test/1
   document: ../studio.ui.yaml
   tests:
     - name: a looping script repeats rows
       steps:
         - open: {page: campaigns.climb, params: {campaign_id: cp-01}}
         - advance: 30s
         - expect: {at: pages/campaigns.climb/sections/points, rows: 6}
   ```

   `ess ui test --path ess/ui/studio.ui.yaml --model ess <file>` fails on 0.98.0: `it shows 9
   [pt-05, pt-04, pt-06, pt-05, pt-04, pt-06, pt-01, pt-02, pt-03]`; the generated React app did the
   same (`apply` in `runtime/live.ts`). On `ess` 0.99.2 the file passes. Needed: `insert_top` patches
   a held key in place (or the format says what a repeated key means), and a check that a looping
   script with `insert_top` replays keys; ESS took the first.
2. **No check that a live event can fill the rows it inserts.** `CampaignProgressed` carries no
   `insufficient`, `best` or `at`, yet the chart and the step table insert its payload as a row of
   `uxsearch.search.Climb`: the `insufficient` line drops to zero at each live point and live steps
   show empty cells. `ess ui check --model` reports nothing; the own generator warns:
   `pages/campaigns.climb/sections/points/live: uxsearch.search.CampaignProgressed carries no
   insufficient, best, at of uxsearch.search.Climb`. (The model is this repository's to fix; the
   check is ESS's.)
3. **The loop period of a fixture script differs between renderers.** Fixed in ESS 0.98.1: the
   generated React app repeats a looping script at its last entry's `at`, as the terminal does. On
   0.98.0: terminal and test clock: the last entry's `at` (`ess-ui-test/src/clock.rs`); generated
   React: the last `at` plus 1 s (`fixturePlayer` in `runtime/live.ts`). The format says only "in a
   loop". A test that advances past one cycle sees different rows in the terminal and in the browser.
4. **An event at `0s` lands in one section and not in its neighbour.** The climb script plays
   `pt-04` at `0s`. In the generated React app the chart (`coalesce: 500ms`) draws it and the step
   table beside it (no `coalesce`) does not; in the terminal the step table does not either (`play`
   pt-05, then the table is `[pt-05, pt-01, pt-02, pt-03]`). Same cause class as 3: when a script
   starts relative to a section's first read is specified nowhere.
5. **A host slot's content cannot be tested in the terminal.** See C3: the terminal draws the
   slot's reason, so any `expect text` on a host-drawn cell fails there and the one-verdict rule
   cannot hold. Needs: a host slot declares a terminal rendering (or a text the runner reads), or
   the test format marks steps on host nodes as browser-only.

Gap F3 (no resolved form) is the list in [What it had to re-implement](#what-it-had-to-re-implement).

## Recommendation for the studio

**Own generator**, on these numbers:

- **Durability.** Of the ten change requests, the stock way needs generated code edited for 4 (C1
  chart, C2 theme link, C5 cell marks, C10 time format) and host slots for 4 (C3, C4, C8, C10); every
  fork is lost on the next `ess generate ui`. The own way changes the generator or the spec for all
  ten and never edits output.
- **Cost per change is the same order.** C1, C2 (theme, then light and dark) and C3: stock 1 to 23
  lines per item, own 17 to 49 lines, plus one 107-line renaming that makes every later uxui theme
  free. Agent minutes were under one per item on both ways; the wall time went to the shared build
  queue on both.
- **Reach.** The own generator reads the model (C3 by type, the thin-event warning) and draws what
  the product needs (a scatter plot is a renderer, not a slot the test runner cannot see).
- **Price.** One page cost 1,807 lines, 1,296 of them re-implementing what ESS resolves and does
  not hand over, and it drew only 3 of 12 composites. The stock runtime for all nine pages is
  7,263 generated lines, maintained by ESS. The own way carries the semantics risk listed in
  [Decisions ESS left open](#decisions-ess-left-open) and is not yet held by `ess ui test`.

Conditions, in order: hold the own page to `ess/ui/tests/studio.yaml` through its `data-ui-path`
attributes (a browser run of the Playwright spec) before it draws a second page; ask ESS for
`ess ui load --format json` with model bindings (removes about 960 lines here) and for a
conformance suite every generator runs (gaps 1, 3, 4). If either is refused, re-assess: without
them the own generator re-implements ESS's resolution for every construct it adds.
