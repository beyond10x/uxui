# The studio climb

Status: built and measured 2026-11-25 (`story:studio-climb`, roadmap step 5). The engine climbs its own
studio, `ess/ui/studio.ui.yaml`, on layout model v0 (ESS's terminal renderer at tag 0.99.4, in
process) and prints the ranked frontier. Method and terms: `docs/cost-spike.md` and
`engine/search/src/climb.rs`.

## Page

`campaigns.climb`, the live climb of one campaign, param `campaign_id=cp-01` (the running campaign of
`ess/ui/fixtures/studio.yaml`). With `variants.compare` it has the most sections of a studio page (6):
a record, a KPI list, two line charts, the step table and the frontier, five of them in one column, so
one column alone gives 120 orders. Its primary action is the header's `stop` (no section holds a
primary button).

## Axes

| axis | values | read by the terminal renderer | in the climb |
|---|---|---|---|
| theme | `light` (the one key of `themes`, also `theme.default`) | no: ESS 0.99.4 draws no theme | yes, 1 value |
| section order in column `side` | `[summary]`: 1! = 1 | yes, as the stack order | yes |
| section order in column `main` | `[hill_kpis, hill, progress, points, frontier]`: 5! = 120 | yes, as the stack order | yes |
| column order (`side`, `main`) | 2 | only through the stack order | no |
| column `width` (`narrow`, `wide`) | 3 each | no | no |
| column `frame: shared`, `collapsed` | not written by the document | yes (one frame for the column) | no |

The terminal renderer lacks `no_columns` and falls back to `stack`: it draws the page's `sections:` in
the order written and ignores the columns (`ess-ui-tui/src/profile.rs`, `app.rs` `visible_sections`).
A variant therefore writes each column's `sections: [...]` in its order and the page's `sections:`
column by column, so the stack is the columns read top to bottom, left to right, and a renderer that
draws columns reads the same variant. The id shows the columns, `|` between them:
`light:summary|hill_kpis,hill,progress,points,frontier` is the document as written. That holds because
the studio writes its `sections:` column by column already; on a page that does not, the variant with
every column in its written order still writes `sections:` column by column, so it differs from the
document in the stack the terminal renderer draws, and no variant is the document as written.

The column reader takes `columns:` as a block sequence, its entries deeper than the key or at its
indent, each entry a flow mapping (`- {name: side, sections: [summary]}`) or a block mapping starting
on the dash's line (`- name: side`), and each column's `sections:` a flow sequence on one line. It
refuses, with a message, a `layout:` or `columns:` written as a flow mapping or sequence, a column
whose `sections:` is a block sequence, and an entry that is a lone `-` with its mapping on the next
lines. A scalar layout (`layout: stack`) is a page without columns.

Variants: 1 theme x 1 x 120 = **120**, each checked (`ess_ui_check::check_source`, without the
`uxsearch` model: no axis touches a view, command or event name), opened fresh and drawn at 120x40 and
48x40.

## Command

From the repository root, after `cargo test --manifest-path engine/search/Cargo.toml --locked` built the
binary through the pool:

```console
engine/search/target/debug/ux-search climb --doc ess/ui/studio.ui.yaml --page campaigns.climb \
  --param campaign_id=cp-01 --sizes 120x40,48x40 > engine/search/tests/fixtures/studio.frontier.txt
```

That file is the golden output the test module `studio` holds the climb to (`engine/search/tests/studio.rs`).
A change to `ess/ui/studio.ui.yaml` or its fixtures turns that test red until the command is run again.

## Frontier

42 of 120 variants, two groups: 18 with `below_fold=1 cut=2` and 24 with `below_fold=20 cut=0`. The top
lines:

```text
frontier 42
   1 light:summary|frontier,hill,hill_kpis,points,progress errors=0 below_fold=1 cut=2 primary_rows=4
   2 light:summary|frontier,hill_kpis,hill,points,progress errors=0 below_fold=1 cut=2 primary_rows=4
   3 light:summary|frontier,hill_kpis,points,hill,progress errors=0 below_fold=1 cut=2 primary_rows=4
```

The ranking separates variants on `below_fold`: 1 to 20 regions left out at 48x40. The document as
written ranks 72nd (`below_fold=11 cut=2`), dominated by the top variant on that term alone. `cut` trades
against it. The 18 variants of the first group end `main` with a line chart (`progress`, or `hill`
right after `progress`): one region left out, two regions cut over both sizes. The 24 of the second group
end `main` with the step table (`points`): 20 regions left out, none cut.
`errors` (0 for every variant) and `primary_rows` (4: the header's `stop` sits above every section) do
not separate any two variants.

`below_fold` counts regions, so a section with many row regions (the step table) weighs more when it is
pushed out than a chart of one region. A term that weighs what a person on this page looks for first is
not in v0.

## Time

One climb, 2026-11-25 18:22 CET on the workstation (20 CPUs, load average 172 to 175 during the run),
the debug build with its dependencies at `opt-level = 2`: **32.5 s wall, 17.1 s user**, about 143 ms of
CPU per variant (check, open, four drawings). The test module runs the full climb once and two climbs
of 6 variants (cap 3) for the repeat; the whole test binary of `ux-search`, 29 tests, finished in 11.8 s
on the same host that evening.
