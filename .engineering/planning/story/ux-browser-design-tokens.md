---
format: aep.planning-md/3
id: story:ux-browser-design-tokens
kind: story
status: implemented
title: Design-token checks on a live page
summary: ux-browser checks counts font sizes, colours, gaps and radii off a declared token set, and the lowest text contrast, per page, viewport and mode
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-values-output
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T08:31:58Z", actor: "human:operator", revision: 2, decided_on: {"recorded":{"approval":1}}}
- {from: "proposed", to: "active", at: "2026-11-27T08:31:58Z", actor: "human:operator", revision: 3, decided_on: {"recorded":{"approval":1}}}
- {from: "active", to: "implemented", at: "2026-11-27T08:49:10Z", actor: "human:operator", revision: 4, decided_on: {"recorded":{"test_result":1,"approval":1}}}
---
# Story: Design-token checks on a live page

## Outcome

A consumer that declares its design tokens (the type scale, the theme's colours, the spacing unit, the allowed radii) learns, per page, viewport and mode, how many font sizes, colours, gaps and radii the page uses off those tokens and the lowest text contrast, each with the offending nodes named.

## Context

Two consumers that run UI rounds asked for a check uxui does not have: "a page uses only the scale's font sizes, the theme's colours and the spacing unit". One is a control panel generated from an ESS UI spec, the other a run page with a dark theme and a clear type scale.

`docs/metrics.md` § 11 "Design proxies" defines `proxy.type_scale`, `proxy.spacing_units` and `proxy.radii`, and § 8 `a11y.contrast_min`, as a catalogue for the layout model; no engine computes them on a live page.

Decided 2026-11-27: the owner's go, consumer gap of two UI-round consumers.

## Acceptance

1. A configuration with a `tokens` section (font sizes as a list or a base and ratio, colours as custom properties read from the page's root in each mode and/or explicit values, the spacing unit in px, the allowed radii) reads; a section with an unknown key, an empty list or a value that is not a number is refused with exit 1.
2. On a fixture page whose font sizes, colours, gaps, paddings and radii all lie on its declared tokens, in light and dark mode, `proxy.type_scale@1`, `proxy.colours@1`, `proxy.spacing_units@1` and `proxy.radii@1` are 0 and `a11y.contrast_min@1` is at least 4.5.
3. On a fixture page with one planted offender of each kind, in light and dark mode, each of the four counts is above 0, `a11y.contrast_min@1` is below 4.5, and each result's details name the offending node and its value, at most a fixed number per result.
4. A token kind the section does not declare, or a declared custom property the page does not define in a mode, gives an incomplete result (`coverage_below_plan`, no complete 0) and a `not_run` coverage entry, never a pass.
5. The values file (`ux-browser values`, `checks --values`) carries the five outputs like the others; `a11y.contrast_min` folds to the smallest value over the gating viewports.
6. A configuration without `tokens` writes the documents, captures and values files it wrote before, byte for byte (the existing golden files unchanged).
