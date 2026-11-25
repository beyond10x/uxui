---
format: aep.planning-md/3
id: story:toy-climb
kind: story
status: implemented
title: First climb on a toy page
tags:
- roadmap-step-5
relations:
- decomposes: epic:engine-slices
- depends_on: story:cost-spike
scope:
- confidence: cited
  path: AGENTS.md
- confidence: inferred
  path: docs/roadmap.md
- confidence: cited
  path: engine/search
- confidence: cited
  path: tools/gate.sh
- confidence: cited
  path: website/docs/concepts/studio.md
- confidence: cited
  path: website/docs/index.md
- confidence: cited
  path: website/docs/limitations.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:16Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:16Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-25T16:47:32Z", actor: "human:operator", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
# Story: First climb on a toy page

## Outcome

The engine is shown to search end to end: variants, evaluations on layout model v0, a ranked
frontier.

## Context

Roadmap step 5, first half (`docs/roadmap.md:39-40`). Needs the cost measured by `story:cost-spike`
and ESS gaps 1 and 2 (`docs/roadmap.md:30, 62`).

Surface: not known yet; records `Cargo.toml` as an inferred surface until the engine's code home is
named.

## Acceptance

An exhaustive search over two axes (theme × section order) on an ESS example page, evaluated on
layout model v0, prints a ranked frontier as text.

## Provenance

Migrated from `docs/roadmap.md`, step 5, lines 39-40. First written 2026-11-24 · last touched
2026-11-24 · 1 git revision.

## State on 2026-11-24

Exists: no engine. The scaffold spike searched exhaustively on layout model v0 and printed a
ranking: stage 1 (`tools/scaffold/stage1.mjs`, `tools/scaffold/campaign.mjs`, the Rust cell reader
`tools/scaffold/cells/` over the ESS crates) over 144 variants of the lifted docs hero and over 288
variants of the docs landing page (`docs/scaffold-spike.md`,
`docs/campaign-2026-11-24-docs-landing.md`; PR #1, PR #3). Neither is theme x section order on an
ESS example page.

Missing: the engine doing the search; the spike drivers are Node (`docs/scaffold-spike.md`, "Language
of the spike code"). `story:cost-spike`, which this story depends on, is partly met.

## Scope

Derived 2026-11-25 by `story-scoper`, decided by the coordinating session the same day. Every line is
**cited**, **decided** (a choice the coordinator made, reversible) or **inferred**.

- **Primary surface:** `engine/search/` (crate `ux-search`, the one `story:cost-spike` creates): a
  search module and a `ux-search climb` command — decided
- **Layout model v0:** ESS's terminal renderer `ess_ui_tui::App` (`from_text`, `open_page`,
  `render_text`, `regions` giving `Region {path, row, area, text}`), at ESS tag 0.99.4 — cited
  (`docs/roadmap.md:41`, `docs/state-space.md:99, 128-138`)
- **Variants:** a section-order permutation is a rewritten document; a theme is the document's
  `theme.default` set to each of its `themes` keys; each variant is opened fresh with `from_text`, so
  the toy needs one state per variant and no UI state API — decided
- **Evaluation:** per variant: `ess_ui_check` errors, then the drawn regions at fixed sizes (for
  example 120x40 and 48x40): rows before the primary action, regions below the fold, regions cut by
  the frame; a theme term only if the renderer or the style resolver (`ess-ui/src/style.rs`) gives a
  number that differs by theme — inferred, the implementor measures and writes which terms move
- **Output:** the ranked frontier as text, byte-identical on two runs (determinism), checked by a test
  against a committed golden file — decided
- **Reuses:** `uxui-doc` is not needed (YAML is read and written through ESS's loader); `ux-static`
  is not called — inferred
- **Documents:** `docs/roadmap.md` step 5, `website/docs/limitations.md` ("The engine does not
  exist"), `website/docs/index.md:15`, `website/docs/concepts/studio.md:18`, `CHANGELOG.md` — cited
  (2026-11-25 read)
- **Confidence:** medium — theme cannot change a v0 layout if the renderer reads colour only
  (`docs/state-space.md:85-88, 135-136`)
- **Would collide with:** `story:cost-spike` (same crate, done first in the same unit),
  `tools/gate.sh`, `AGENTS.md`
- **Safety fact:** the crate is outside the root workspace; no existing caller changes — unproven
