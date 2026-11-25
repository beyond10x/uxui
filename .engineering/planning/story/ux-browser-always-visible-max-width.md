---
format: aep.planning-md/3
id: story:ux-browser-always-visible-max-width
kind: story
status: implemented
title: Always-visible items limited to narrow viewports
relations:
- decomposes: epic:engine-slices
- depends_on: story:ux-browser-always-visible-by-css
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T03:12:02Z", actor: "human:operator", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-25T03:12:03Z", actor: "human:operator", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-25T03:20:41Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
# Story: Always-visible items limited to narrow viewports

## Outcome

An always-visible item that a page shows only on narrow viewports (a phone menu button) is checked
at those viewports and not counted as a miss at wider ones.

## Context

`always_visible` items take `min_width` (the item is checked from that viewport width up). There is no
upper bound. Found on 2026-11-25 in the manager-v3 demo run with ux-browser 0.5.0 (12 pages x 4
viewports): the phone layout shows an "Open menu" button at 390 px and hides it from 1024 px, where
"Toggle navigation" takes its place. Declaring "Open menu" as always visible gives a miss at 1024,
1280 and 1440 on every page; leaving it out leaves the phone navigation unchecked.

## Acceptance

An item may set `max_width`; it is checked only at viewports with `min_width <= width <= max_width`;
`max_width` below `min_width`, zero or a non-integer is refused with the item's name; a fixture page
with a button shown below 1024 px and `max_width: 1023` counts 0 misses at 390 and 1440, where 0.5.0
counted 1 at 1440; a configuration without `max_width` gives 0.5.0's document.
