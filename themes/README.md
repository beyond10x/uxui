# Themes

uxui owns themes and templates (`AGENTS.md`, Direction). Styling and layout are solved here, never by a
new `ess-ui/1` key. Every theme has a light and a dark mode: the system setting until a person picks one
with `theme-toggle.js`, which sets `data-uxui-mode` on `<html>`. An app may set `data-uxui-mode` itself
instead of loading the toggle.

| file | what |
|---|---|
| `theme-toggle.js` | the mode toggle; load it in `<head>` so the page never flashes |
| `studio.css` | the studio's theme, over the `--ui-*` variables the stock `ess-ui/1` generator writes |
| `dense.css` | dense tables and records over the class hooks below: column widths, one line per cell, a label beside its value, a colour per value |
| `slot-jobs.css` | slot's job list and job detail, over `dense.css`: a width per column, a colour per toolchain, job state and priority |
| `slot-jobs.md` | what slot's web view changes to use `dense.css` and `slot-jobs.css` |
| `examples/slot-jobs.html` | the markup of the class hooks for slot's two pages, to open in a browser |
| `fixtures/slot.ui.yaml` | slot's `ess-ui/1` document, which `tools/check-themes.sh` checks `slot-jobs.css` against |
| `fixtures/README.md` | the source and sha256 of each fixture |

`tools/check-themes.sh` (a gate step) checks every theme: a light and a dark mode that set the same
properties, no `var()` of an unset property, this table, and what `dense.css` and `slot-jobs.css` promise.

## Class hooks

The template a generator writes so that `dense.css` and an app theme over it can style each column,
field and value. Every hook comes from what the `ess-ui/1` document already says: the field name, its
`as`, and the value of the row.

| hook | on | from the document |
|---|---|---|
| class `uxui-dense` | the `<table>` of a collection, the `<dl>` of a record | the composite (`collection`, `record`) |
| class `uxui-dense-frame` | the element around the table; the column queries of an app theme measure it | the section |
| class `uxui-field-<field>` | each `<th>` and `<td>` of a column, each `<dt>` and `<dd>` of a record field, a header badge | `field` of the column or field; for the header badge the last segment of `badge.text` |
| class `uxui-as-<as>` | the same elements | `as` of the column or field, `text` when it has none |
| attribute `data-uxui-value` | each `<td>` and `<dd>`, a header badge | the row's value of the field as text, when it is a string, a number or a boolean; absent for a list, a map or no value |
| class `uxui-badge` | the element that draws a value `as: badge` (inside its `<td>` or `<dd>`), a header badge | `as: badge`, `header.badge` |
| class `uxui-empty` | the one row of a table without rows, that holds the empty message | `states.empty` |

A record may wrap each `<dt>` and `<dd>` pair in a `<div>`, as the stock generator does.

What the hooks give an app theme:

- Column widths: `table.uxui-dense .uxui-field-<field> { width: … }`. The table has a fixed layout, so a
  column without a width shares what is left.
- One-line cells: every cell of a dense table stays on one line and is cut with an ellipsis.
- A label beside its value: a dense record is a two-column grid.
- A colour per value: on `.uxui-field-<field>[data-uxui-value="<value>"]` set `--uxui-value-color`
  (the colour mark), `--uxui-value-fill` (a badge's fill) and `--uxui-mark: ""` to draw the colour mark
  before the value; set `color` and `font-weight` of the value's text directly, a badge inside inherits
  them.
- Sticky headings below an app's own header: `--uxui-dense-sticky-top`. Rows that reach into the page
  margin on each side: `--uxui-dense-reach` (0 by default).

## Using a theme

uxui's own generator takes each file with `--theme`: `uxui-gen … --theme themes/studio.css --theme
themes/theme-toggle.js` (`docs/own-generator.md`). Over the stock generator's app a stylesheet loads after
its own (`docs/change-list.md`). A generator that writes the class hooks loads `dense.css`, then the app
theme. slot: `slot-jobs.md`.
