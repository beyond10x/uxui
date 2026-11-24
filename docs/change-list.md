# Change list: typical requests after a UI is generated

Story 3 of the round in [generator-round.md](generator-round.md). A fixed list of change requests a
product team asks for once the studio is generated, on the live climb (`pages/campaigns.climb`) and
the target catalog (`pages/targets.catalog`). Both ways are scored on this list: the stock generator
(`ess generate ui --target react`, `ess` 0.98.0) and the application's own generator
(`generators/own/`, [own-generator.md](own-generator.md)).

Each item has the request, its acceptance and the layer that changes on each way. Acceptance is an
`ess-ui-test/1` step list where the format can express it; where it cannot, a manual check, and the
reason. The steps on nodes that exist today were run with `ess ui test` on 0.98.0 (paths of cells
are `…/rows/<key>/columns/<name>`); steps on nodes an item adds (C7, C8) are written for after it.

Layers:

- **spec**: an edit of `ess/ui/studio.ui.yaml` (or the model under `ess/`), then generate again.
- **option**: an option of `ess generate ui`. 0.98.0 has none that changes what is drawn
  (`--target`, `--model`, `--fixtures`, `--http`, `--refines`, `--strict-requires`), so this column
  is empty for every item.
- **slot**: a host slot declared in the spec and a component the host writes
  (`src/host-components.tsx`).
- **fork**: an edit of generated code, lost on the next `ess generate ui` unless re-applied.
- **theme**: a uxui theme stylesheet (`themes/<name>.css`) loaded after the generated one.
- **own**: an edit of the own generator (`generators/own/`); the output is generated again.

## C1. Sort the climb chart and the step table by `seq`

Request: the chart's x axis reads 1, 2, 3, 4, 5 in order, also after live points arrive; the step
table shows the newest step first.

Acceptance:

```yaml
- open: {page: campaigns.climb, params: {campaign_id: cp-01}}
- play: {event: uxsearch.search.CampaignProgressed, with: {climb_point_id: pt-05}}
- advance: 1s
- expect: {at: pages/campaigns.climb/sections/points, rows: [pt-05, pt-03, pt-02, pt-01]}
```

`pt-04` is missing on purpose: the script plays it at `0s`, and in the terminal runner that event
lands before the section's read has answered and is lost (today the step shows
`[pt-05, pt-01, pt-02, pt-03]`); the generated React app loses it in this table too, and draws it in
the chart. See own-generator.md, gaps.

The chart's order is a manual check: no step reads the order of a chart's points (`expect text`
holds on containment, not order). Check: after two live points, the x labels under the chart read
`1 2 3 4 5` left to right.

| Way | Layer | What changes |
|---|---|---|
| stock | spec | `sort: {by: seq, dir: desc}` on the `points` collection |
| stock | fork | the chart: `chart` has no `sort` (finding F1); sort rows by `x` in `runtime/composites/chart.tsx` |
| stock | slot (alternative) | a section host slot standing in for `chart` |
| own | own | sort by `x` in the chart renderer; read `sort` for collections |

## C2. Brand theme

Request: the studio wears uxui's theme `studio` (`themes/studio.css`): its colours and fonts, four
distinct series colours on the progress chart (finding F4: three of four share one colour), shadows,
hover and focus, in a light and a dark mode. The mode follows the system setting until a person
picks one with the toggle (`themes/theme-toggle.js`, which sets `data-uxui-mode` on `<html>`).

uxui owns themes; styling is never solved by extending `ess-ui/1` (owner, 2026-11-24). The theme is
CSS over the stock app's `--ui-*` custom properties and `ui-` classes, so a generator applies it by
loading it after its own stylesheet.

Acceptance: manual. `ess-ui-test/1` has no step that reads a style. Check: the accent of buttons and
the active view switch is the theme's `#5b5bd6`; the four polylines and their legend swatches have
the theme's four series colours; a table row shows the focus colour on hover; with the system in
dark mode, or after the toggle, the page background is `#0e1020` (computed style).

| Way | Layer | What changes |
|---|---|---|
| stock | fork | `www/index.html` links the theme after `main.css` and loads the toggle in the head; generated, so re-applied after every `ess generate ui` (or a post-generate step). The mode cannot use `data-theme`: the stock runtime writes it (`runtime/theme.ts`) |
| stock | spec (alternative) | `tokens:` and `themes:` (a `dark` theme chosen by a shell preference state) cover colours, space, radius and type only: no series colours, shadows, hover or focus, and no system-setting default |
| own | own | the generator names its custom properties and classes as the stock app does and takes `--theme <file>`, once per file: a stylesheet after its own, a script in the head |

## C3. A custom cell for KPI values

Request: the frontier's KPI values read `effort.actions 6 · completion.miss 0`, not as JSON.

Acceptance:

```yaml
- open: {page: campaigns.climb, params: {campaign_id: cp-01}}
- expect: {at: pages/campaigns.climb/sections/frontier/rows/fe-01/columns/kpi_values, text: "effort.actions 6", not_text: "{"}
```

| Way | Layer | What changes |
|---|---|---|
| stock | spec | `as: list` is the nearest built-in, and prints each reading as JSON |
| stock | slot | a field host slot `KpiValues` on the column, and the component in `src/host-components.tsx` |
| own | own | a cell renderer for lists of `{metric_key, value}` |

## C4. The variant digest is the link; drop the separate Open button

Request: in "Frontier now", the variant digest itself opens the variant page; the `Open` row action
goes away.

Acceptance:

```yaml
- open: {page: campaigns.climb, params: {campaign_id: cp-01}}
- expect: {at: pages/campaigns.climb/sections/frontier/rows/fe-01, not_text: Open}
- act: pages/campaigns.climb/sections/frontier/rows/fe-01/columns/variant_digest
```

The last step is not in the format: `act` on a cell is refused (only rows, actions and column
headers act). Until then, a manual check that a click on the digest opens `variants.detail`.

| Way | Layer | What changes |
|---|---|---|
| stock | spec | remove the `open` row action; `as: link` treats the value as a URL and does not navigate to a page |
| stock | slot | a field host slot drawing a link to `variants.detail` |
| own | own | a column `navigate` in the generator, read from a page-local convention |

## C5. Hide columns on narrow screens

Request: below 700 px the step table drops `insufficient`, `best` and `at`.

Acceptance: manual. `ess-ui-test/1` has no viewport, and `visible` has no viewport root. Check: at
600 px width the step table has 4 columns; at 1200 px 7.

| Way | Layer | What changes |
|---|---|---|
| stock | theme | a media query in the theme; the header cell carries `data-ui-path`, the body cells carry nothing, so the rule counts columns (`td:nth-child(4)`) and breaks when a column moves |
| stock | fork | a column class or path on each cell in `runtime/composites/collection.tsx`, then the theme rule |
| own | theme | a media query in the theme on `[data-column="insufficient"]`; the own generator marks each cell with its column |

## C6. Change copy

Request: the page title reads `Climb`; the column `frontier size` reads `Frontier`; the stop confirm
reads `Stop this campaign?` and explains that running evaluations finish.

Acceptance:

```yaml
- open: {page: campaigns.climb, params: {campaign_id: cp-01}}
- expect: {at: pages/campaigns.climb/header, text: Climb}
- expect: {at: pages/campaigns.climb/sections/points/columns/frontier_size, text: Frontier}
- act: pages/campaigns.climb/header/actions/stop
- expect: {at: pages/campaigns.climb/header/actions/stop/confirm/overlay, text: running evaluations finish}
```

| Way | Layer | What changes |
|---|---|---|
| stock | spec | `title`, the column's `label`; the inline confirm has no `body` (only `title`, `show`, `confirm_label`), so the explanation needs a named confirm overlay with `body` |
| own | spec | the same edits; the own generator reads `label`, `title` and `body` |

## C7. A field end to end: the last variant of each step

Request: the step table shows the last variant a step evaluated (`last_variant`, on the entity
`ClimbPoint`, not on the view `Climb` nor on the event `CampaignProgressed`), also for live steps.

Acceptance:

```yaml
- open: {page: campaigns.climb, params: {campaign_id: cp-01}}
- expect: {at: pages/campaigns.climb/sections/points/rows/pt-03/columns/last_variant, text: "sha256:v-b2"}
- play: {event: uxsearch.search.CampaignProgressed, with: {climb_point_id: pt-04}}
- advance: 1s
- expect: {at: pages/campaigns.climb/sections/points/rows/pt-04/columns/last_variant, text: "sha256:v-d4"}
```

| Way | Layer | What changes |
|---|---|---|
| stock | spec | model: the view `Climb` and the event `CampaignProgressed` gain `last_variant`, the command's payload carries it; fixtures and script rows; the column |
| own | spec | the same; nothing in the generator when it reads columns generically |

## C8. A bespoke widget: the frontier as a scatter plot

Request: beside "Frontier now", a scatter plot of the frontier, `effort.actions` on x and
`completion.miss` on y, one dot per variant; a click on a dot opens the variant.

Acceptance: manual, partly expressible. A host slot is addressed by its path only:

```yaml
- open: {page: campaigns.climb, params: {campaign_id: cp-01}}
- expect: {at: pages/campaigns.climb/sections/frontier_plot, state: ready}
```

Check by hand: two dots at (6, 0) and (5, 1); a third appears when `FrontierChanged` admits
`sha256:v-d4` (after the refetch the fixture has no row for it, see own-generator.md, fixture
semantics).

| Way | Layer | What changes |
|---|---|---|
| stock | slot | a section host slot (`stands_in_for: none`, `reads: uxsearch.search.Frontier`), the component in `src/host-components.tsx` |
| own | own | a scatter renderer in the generator, or a page-local hook |

## C9. Group the target catalog by scope

Request: the catalog lists targets grouped by scope, in the order product, page, section, component,
widget.

Acceptance:

```yaml
- open: targets.catalog
- expect: {at: pages/targets.catalog/sections/list, rows: [tg-01, tg-02, tg-03, tg-05, tg-04]}
```

| Way | Layer | What changes |
|---|---|---|
| stock | spec | `group_by: scope`, `group_order: [product, page, section, component, widget]` on the list |
| own | own | `group_by` and `group_order` in the generator's collection (the own generator draws only the climb today) |

## C10. Relative times and grouped numbers in the step table

Request: `at` reads `20 s ago`, updated each second; `evaluated` reads `1,240`.

Acceptance: manual for the relative time (it depends on the wall clock; the test clock is virtual
and no step reads a time format). Grouped numbers are already the browser's behaviour, and
`expect text` compares numbers of four or more digits by value.

| Way | Layer | What changes |
|---|---|---|
| stock | slot | a field host slot on `at`; `as` offers `timestamp`, `time`, `date`, `duration`, no relative time |
| stock | fork | `formatValue` in `runtime/core.tsx` |
| own | own | a relative-time cell in the generator |

## Summary

| Item | stock: spec | stock: option | stock: slot | stock: fork | own |
|---|---|---|---|---|---|
| C1 sort by seq | table | none | chart (alt.) | chart | generator |
| C2 brand theme | (tokens: colours and type only) | none |  | link the uxui theme | link the uxui theme |
| C3 KPI cell |  | none | yes |  | generator |
| C4 digest as link | drop action | none | yes |  | generator |
| C5 narrow screens |  | none |  | cell marks, then theme | theme only |
| C6 copy | yes (confirm body via named overlay) | none |  |  | spec only |
| C7 field end to end | model, fixtures, column | none |  |  | spec only |
| C8 scatter plot |  | none | yes |  | generator |
| C9 group by scope | yes | none |  |  | generator |
| C10 relative time |  | none | yes | or here | generator |

Story 5 measures each item. Story 4 measured C1, C2 and C3 on both ways (see
[own-generator.md](own-generator.md#three-changes-on-both-ways)).
