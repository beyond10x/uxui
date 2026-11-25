# `ux-static@1`

Status: version 1, 2026-11-24. Roadmap step 4 (`docs/roadmap.md`). Crate `engine/static`, binary
`ux-static`; it reads with `engine/doc` (`uxui-doc`: the YAML reader, the JSON writer, SHA-256 and the
document's node tree). No browser, no layout model, no third-party crate.

```sh
ux-static --document ess/ui/studio.ui.yaml --journeys ess/ui/tests/studio.yaml \
  --declarations ess/ui/studio.uxui.yaml --project uxui --assessment studio-static \
  [--root .] [--out studio-static.evidence.json] [--class local]
```

It writes one `ratchet-evidence/1` document per run (ratchet `docs/evidence-format.md`, revision 2),
the same bytes for the same inputs. Exit status, as ratchet's assessor contract (§ 10) reads it:

| exit | means | document |
|---|---|---|
| 0 | the run ended, whatever the document holds: the gate decides, not the assessor | written |
| 1 | refused inputs: a file is unreadable or malformed, or the inputs do not belong together (a `document` key naming another file, a journey declared that the test file lacks, an unknown declaration key) | none |
| 2 | wrong invocation: an unknown argument, a missing one (`--project`, `--assessment` and the three files are required), an argument given twice or without a value, a `--class` ratchet does not know, an empty `--project` or `--assessment`, a `--root` that is not a directory | none |
| 3 | infrastructure: the document could not be written to `--out`; ratchet records `stopped_by_infrastructure` | none |

`ux-static --version` (or `-V`) prints `ux-static <version>` and a newline on stdout, the crate
version a run records as `evidence.conditions.tools.ux-static`, and exits 0 without reading a file;
`ux-static --help` (or `-h`) prints the usage. The release checks the answer of the binary it packages
against the tag (`tools/release-versions.sh`).

Any of the three files may lie outside `--root`, as a generated variant in a scratch directory does.
Such a file has no path in the evidence; its digest identifies it. The journeys and declarations must
still name the document in their `document` key.

The declaration of the assessor, `ratchet-assessor/1` (ratchet § 10), is
`engine/static/ux-static.assessor.yaml`: its inputs, outputs, scopes, units, coverage unit and item
fields. A test checks it against the CLI's arguments and the documents the assessor writes.

## Metrics

All three are exact counts, lower is better, one result per metric and scope item.

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

Typed text counts as one action, not one per key. Any other keyword makes the result incomplete
(`coverage_below_plan`; the step is `unsupported`). So does a step naming a node path the document does
not declare (`rows/<key>` dropped): ESS fails that step with `no node at <path>`, so the journey cannot
be followed (`incompatible_target`; the step is `not_run`). A step on a node where the document alone
cannot say what is drawn is `unsupported`.

An unsupported step can only keep or raise the count, so the actions of the known steps are a lower
bound: `value` with `at_least: true` when that sum is above 0, no value when it is 0. A journey with a
`not_run` step has no bound: it cannot be followed on this document, so it has no number of actions.
When both apply, the cause is `coverage_below_plan`.

### Accessible names

The rule follows the ACT Rules Format: per node, passed, failed or cantTell; a page with none of these
nodes is inapplicable. The name is the document's text, else the default the `ess-ui/1` schema or ESS's
renderers give; each node's entry in `details.items` says which (`declared`, `derived`, `default`).

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

## Evidence

One `ratchet-evidence/1` document per run, under revision 2 of ratchet's `docs/evidence-format.md`:
E1 to E6 decided A, E7 to E9 on their default A. The results are exact, so the document has no `run` member and the same
inputs give the same bytes. JSON, compact, one result per line. Fields:

| field | value |
|---|---|
| `format` | `ratchet-evidence/1` |
| `evidence.project`, `.assessment` | from `--project`, `--assessment` |
| `evidence.assessor` | `ux-static@1` |
| `evidence.candidate` | the document's digest: SHA-256 of its bytes |
| `evidence.target` | `{kind: document, class, selection: candidate, identifiers: {path, digest, digest_form: bytes}}`; class from `--class`, default `local`; no `path` for a document outside `--root`, which its digest identifies |
| `evidence.inputs` | `journeys`, `declarations`, each `{value: <path>, digest, digest_form: bytes}`; no `value` for a file outside `--root` |
| `evidence.completion` | `completed`; a run that cannot complete is an error, not evidence |
| `evidence.conditions` | `{tools: {ux-static: <crate version>}}` |
| `evidence.coverage` | unit `check`: one rule on one declared node, one step or one node that needs a name. `executed`, `skipped` (always 0), `not_run`, `unsupported`, `passed`, `failed`; `not_executed` lists each check that is not executed, `{id, state, reason}` |
| `evidence.results[]` | a `number` per metric and scope item, then a `findings` identity set per scope item of a rule (variant, journeys, pages) |

Paths are relative to `--root` (default the working directory), with `/`. A file outside it is read
and has no path: `identifiers.path` or the input's `value` is left out, and the digest identifies the
file. A node declared twice for one scope item (a signal that is also required) is one check.

A result: `output`, `kind: exact`, `scope` (`variant`, `journey`, `page`), `item` (`document`, the
journey's name, `pages/<page>`), `value_type`.

- `number`: `value` as text and `unit`. An incomplete result has `incomplete: {cause, detail}` and no
  value, or a lower bound: `value` with `at_least: true` when failures or actions were already
  counted.
- `identity_set` (output `findings`): `key: [rule, node_path]`, `items` of `{identity: {rule,
  node_path}, fields: {reason}}`, one per failed check; empty when nothing failed. Incomplete with the
  same cause as the number of its scope item.
- `details`: structure and names: `outcomes`, counts of passed, failed, cantTell (and inapplicable
  for names); actions: `items`, each step that took an action, `{step, keyword, at}`; names: `items`,
  every applicable node, `{node_path, outcome, source, name}`.

ACT outcomes: passed and failed are executed checks; cantTell is `unsupported`, listed in
`not_executed`, and makes its scope item's results incomplete (`coverage_below_plan`); inapplicable is
no check. Nothing declared for a scope item: incomplete, `coverage_below_plan`, no value, no `details`.
A result that cannot be computed is incomplete, never a pass.

Not filled: `target.revision` (the assessor reads files, not a revision), `protocol_version`, `run`.
The document digest is of its bytes, not of a canonical form (ESS gap (b) of `docs/roadmap.md`).
