# `ux-static@1`

Status: version 1, 2026-11-24. Roadmap step 4 (`docs/roadmap.md`). Crate `engine/static`, binary
`ux-static`; it reads with `engine/doc` (`uxui-doc`: the YAML reader, the JSON writer, SHA-256 and the
document's node tree). No browser, no layout model, no third-party crate.

```sh
ux-static --document ess/ui/studio.ui.yaml --journeys ess/ui/tests/studio.yaml \
  --declarations ess/ui/studio.uxui.yaml [--out records.jsonl] [--class local]
```

It writes one JSON line per record, the same bytes for the same inputs. Exit 0 when the records are
written, whatever they hold: the gate decides, not the assessor. Exit 1 when an input is unreadable or
the inputs do not belong together (a `document` key naming another file, a journey declared that the
test file lacks, an unknown declaration key). Exit 2 on a usage error.

## Metrics

All three are exact counts, lower is better, one record per metric and scope.

| id | scope | value | catalogue (`docs/metrics.md`) |
|---|---|---|---|
| `structure.required_present@1` | journey; variant | declared nodes the document lacks: required(J) and signal(J) per journey; `always` and `cue` for the variant (scope id `document`) | the static half of `completion.miss`: the node exists at all, before any layout says it is visible |
| `effort.actions_per_journey@1` | journey | actions the journey's `ess-ui-test/1` steps take | the authored path's count; `effort.actions_min` is the shortest path over the action graph, `effort.actions` adds layout |
| `a11y.accessible_names@1` | page | interactive nodes with no accessible name | `a11y.name_missing`, with an ACT outcome per node and scoped per page |

### Actions per journey

| step | actions |
|---|---|
| `act`, `select`, `mark`, `type`, `choose`, `move`, `upload`, `page` | 1 |
| the first `open` | 0: where the journey starts |
| a later `open` | 1: a navigation |
| `expect`, `expect_command`, `expect_export`, `play`, `advance` | 0: a check, or the environment acting |

Typed text counts as one action, not one per key. Any other keyword makes the record insufficient. So
does a step naming a node path the document does not declare (`rows/<key>` dropped): ESS fails that step
with `no node at <path>`, so the journey cannot be followed.

### Accessible names

The rule follows the ACT Rules Format: per node, passed, failed or cantTell; a page with none of these
nodes is inapplicable. The name is the document's text, else the default the `ess-ui/1` schema or ESS's
renderers give; each node's entry in `items` says which (`declared`, `derived`, `default`).

| node | name | failed when |
|---|---|---|
| action (any action list, a section's empty-state action) | `label`, else its name (ESS's renderers) | `label` declared empty; `as: icon` without `label` |
| inline confirm, `component: confirm` | `confirm_label`, else `Delete` (schema default) | `confirm_label` declared empty |
| form field (in a form, its groups and tabs) | `label`, else its name with `_` read as a space | `label` declared empty |
| form submit (unless `save: on_change`) | `submit.label`, else `Submit` (ESS's renderers) | `submit.label` declared empty |
| filter choice, choice section | `label`, else its name with `_` read as a space | `label` declared empty |
| primitive `button`, `icon`, `toggle` / `image` | `label` / `alt`, required | missing or empty |
| primitive `link` | `text`; cantTell when it is an expression (`row.`, `params.`, ...) | missing or empty |
| primitive `input` | `placeholder` | missing |

cantTell: a node whose drawn content the document alone does not give — a host slot, a widget use
(ESS expands its body; this reader does not), a `component` that is not an `ess-ui/1` composite, a page
kind neither built in nor declared. Not covered by version 1: the filter bar's built-in search input,
navigation entries and the page switcher.

## Records

Ratchet has no evidence schema yet. The record follows its design's "Assessment run" (ratchet
`docs/design.md` § 6) and the field names of its § 3 example. Fields:

| field | value |
|---|---|
| `format`, `record` | `ratchet-evidence/draft`, `assessment-run` |
| `id` | `ux-static@1/<12 hex of the document digest>/<metric>/<scope kind>/<scope id>` |
| `assessor` | `ux-static@1` |
| `candidate` | `{kind: ui-document, digest}`: SHA-256 of the document's bytes |
| `target` | `{kind: ui-variant, class}`; class from `--class`, default `local` |
| `resolved_target` | `document`, `journeys`, `declarations` (each `{path, digest}`), `journey` (journey scope only), `tool`, `tool_version` |
| `observation` | `{kind: snapshot, runs: 1, conditions: {}}`: deterministic, no conditions |
| `measurement` | `{output, kind: exact, unit, direction: lower, scope, aggregation: count}` |
| `scope` | `{kind: variant \| journey \| page, id}` |
| `result`, `value` | `measured` and the count, or `insufficient` and `null` |
| `value_at_least` | on an insufficient record that already counted failures |
| `insufficient` | `{cause, detail}`; cause `coverage` (an item ux-static cannot evaluate), `input` (a step names no node), `declaration` (nothing declared to measure) |
| `coverage` | `{executed, skipped, not_run, unsupported}` |
| `outcomes` | structure and names: counts of passed, failed, cantTell (and inapplicable for names) |
| `findings` | each node not passed: `{rule, node_path, outcome, reason}`; identity (rule, node path), no condition cell in a static run |
| `items` | actions per journey: each step that took an action, `{step, keyword, at}`; accessible names: every applicable node, `{node_path, outcome, source, name}`, the ACT outcome per node |

A record that cannot be computed is insufficient evidence, never a pass.

## What ratchet's draft could not express

- No evidence schema or format name: `ratchet-evidence/draft` and every field name above are ours.
- A design variant or document as target (`docs/ratchet.md` § 8, gap 5): `target.kind: ui-variant`.
- ACT outcomes: ratchet's coverage has passed and failed but no cantTell or inapplicable; cantTell
  counts as `unsupported` and makes the record insufficient.
- A list of findings with their identity: the plan declares `identity`, the evidence has no field for
  the items; `findings` is ours.
- Causes of insufficient evidence are prose in § 5, not a vocabulary; `insufficient.cause` is ours.
- A partial count ("at least 2 failed, more unknown"): `value_at_least` is ours.
- `direction` on a measurement.
- Timestamps (§ 6 asks for them): omitted, so the same inputs give the same bytes; the digests
  identify the run.
- The document digest is of its bytes, not of a canonical form (ESS gap (b) of `docs/roadmap.md`).
