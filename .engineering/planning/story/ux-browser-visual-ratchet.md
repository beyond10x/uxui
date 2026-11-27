---
format: aep.planning-md/3
id: story:ux-browser-visual-ratchet
kind: story
status: implemented
title: Visual ratchet of a page's screenshots
summary: ux-browser checks compares each page, viewport and mode's screenshot with a baseline PNG within a declared tolerance, writes the differing share and pixels and a diff image, and updates the baseline only on an explicit flag
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
scope:
- confidence: cited
  path: engine/browser/src/png.rs
- confidence: cited
  path: engine/browser/src/visual.rs
- confidence: cited
  path: engine/browser/tests/fixtures/visual
- confidence: cited
  path: engine/browser/tests/visual.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T08:54:49Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"approval":1}}}
- {from: "proposed", to: "active", at: "2026-11-27T08:54:49Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"approval":1}}}
- {from: "active", to: "implemented", at: "2026-11-27T09:13:04Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1,"approval":1}}}
---
# Story: Visual ratchet of a page's screenshots

## Outcome

A consumer that declares a `visual` section in its checks configuration gets, per page, viewport and mode, the share and the number of pixels that differ from a committed baseline screenshot, each unit judged against its declared tolerance, with a diff image of the changed pixels for the round report, and replaces the baseline only when it asks for that.

## Context

Three consumers that run UI improvement rounds (a control panel generated from an ESS UI spec, a run page, a simulation page) asked for a screenshot diff with a tolerance per page and viewport (1366x768, 1920x1080, 390x844), dark and light. `ux-browser render --screenshot` and `checks --capture` exist; nothing compares screenshots against a baseline. The skill's ratchet (`plugin/skills/ui-iteration-loop/SKILL.md` § 6) has no visual unit.

Decided 2026-11-27: the owner's go, consumer gap of three UI-round consumers.

## Acceptance

1. A configuration with a `visual` section (the directory of baseline, shots and diffs, full page on or off, mask selectors, a default tolerance and tolerances per page and viewport, each a share of differing pixels and a per-channel threshold) reads; a section with an unknown key, a share outside 0 to 1, a threshold outside 0 to 255 or a tolerance for an undeclared viewport is refused with exit 1.
2. On a fixture page and a changed copy with one moved block, `visual.diff_px@1` is above 0, `visual.diff_share@1` is above the page's tolerance, the unit counts as failed with a finding, and a diff PNG of the unit's size is written in which the changed pixels are marked.
3. On a fixture page and a changed copy with one colour change, the differing pixels are counted at a threshold below the colour's channel difference and are 0 at a threshold above it.
4. On a fixture page with a live clock whose element is masked, two runs give 0 differing pixels; the same clock unmasked differs.
5. A unit without a baseline file counts `not_run` (a coverage entry, the numbers incomplete), never a pass; a shot whose size differs from its baseline counts failed with a reason naming both sizes.
6. `checks --visual update` writes the run's shots as the new baseline; a run without it never writes the baseline; `--visual` without a `visual` section is refused before a browser starts.
7. The values file carries both outputs per page; `visual.diff_share` folds to the largest value over the gating viewports.
8. A configuration without `visual` writes the documents, captures and values files it wrote before, byte for byte (the existing golden files unchanged).
