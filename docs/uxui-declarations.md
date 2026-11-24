# `uxui-declarations/1`

Status: version 1, 2026-11-24. Read by `ux-static@1` (`docs/ux-static.md`).

An `ess-ui/1` document says what a UI shows and does. It does not say which nodes a journey needs, which
nodes stay visible, what shows that a journey is done, or which node tells a person what to do next.
The metrics of `docs/metrics.md` need these four (§ 1: required(J), always, signal(J), cue;
`docs/state-space.md` § 8). This file declares them by node path, beside the document. It is the
proposal for ESS gap (e) of `docs/roadmap.md`, "UX declarations as constructs".

## File

One YAML file beside the `ess-ui/1` document, named `<document>.uxui.yaml` by convention
(`ess/ui/studio.uxui.yaml` declares for `ess/ui/studio.ui.yaml`).

```yaml
format: uxui-declarations/1
document: studio.ui.yaml
always:
  - pages/campaigns.climb/header/actions/stop
journeys:
  stopping a running campaign asks first and sends the command:
    required:
      - pages/campaigns.list/sections/list/row_actions/stop
      - pages/campaigns.list/sections/list/row_actions/stop/confirm/overlay
    signal: pages/campaigns.list/sections/list/columns/state
```

| key | type | required | meaning |
|---|---|---|---|
| `format` | `uxui-declarations/1` | yes | the format and its version |
| `document` | path | yes | the `ess-ui/1` document, relative to this file; a tool refuses a document other than the one it was given |
| `always` | list of node paths | no | nodes visible in every state of their page (`docs/metrics.md`, `always`) |
| `cue` | node path | no | the node that tells a person what to do next; the steering metrics run only where it is declared (`docs/metrics.md` § 10) |
| `journeys` | map of journey name to a journey | no | one entry per journey of the document's `ess-ui-test/1` file, keyed by the test's `name` |
| `journeys.<name>.required` | list of node paths | no | nodes the journey needs filled or shown at its end, required(J) |
| `journeys.<name>.signal` | node path | no | the node that shows the journey is complete, signal(J) |

No other key is allowed, at the top or in a journey: a misspelt key is an error, not a silent gap.

## Node paths

A node path is the canonical path of `ess-ui/1` (ESS `ess-ui.md`, § NodePath): container keys and
names from the document root, joined with `/`, with no list index. A declaration names a declared node,
so it holds no `rows/<key>` segment. A tool drops such a segment if one is written: a node in a row is
the row template's node.

## Rules

- A journey named here and missing from the `ess-ui-test/1` file is an error: the declaration is stale.
- A journey of the `ess-ui-test/1` file with no entry here, or an entry with neither `required` nor
  `signal`, has nothing to measure: its `structure.required_present` record is insufficient evidence,
  never a pass.
- The same holds for the variant scope when the file declares neither `always` nor `cue`.
