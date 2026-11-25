---
format: aep.planning-md/3
id: story:ux-browser-overflow-hidden-content
kind: story
status: implemented
title: Overflow ignores text that is not drawn
relations:
- decomposes: epic:engine-slices
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T03:12:02Z", actor: "human:operator", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-25T03:12:02Z", actor: "human:operator", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-25T03:23:42Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
# Story: Overflow ignores text that is not drawn

## Outcome

Text inside a closed `details` element, or any other content the page does not draw, is not counted as overflow.

## Context

Consumer: the second consumer (a control plane web application), run on 0.5.0 on 2026-11-25. On its Asks page a `p.full` inside a closed `details.more` reported "right edge 895 past its parent at 892" at 1536x754 and "911 past 907" at 1920x1080. Checked against the source at d060e30: `src/probes/overflow.js` skips a text run only when its parent has `visibility: hidden`; content of a closed `details`, `content-visibility: hidden` or `hidden=until-found` is measured.

## Acceptance

The overflow probe skips content the page does not draw: an element for which `checkVisibility({visibilityProperty: true})` is false, and a text run whose own `visibility` is not `visible` or whose nearest ancestor with a box fails `checkVisibility()`, has `content-visibility: hidden`, or is a closed `details` (its `::details-content` hidden); a fixture page with a long line inside a closed `details` counts 0 overflow, where 0.6.0 counted 1, and the same page with the `details` open counts 1; llm's documentation build gives its example document's values unchanged.

Revised at implementation, 2026-11-25: the earlier text named `checkVisibility({contentVisibilityAuto: true, visibilityProperty: true})` on a text run's parent. Measured on Chrome 149 it is wrong in 4 cases: text directly inside a closed `details` (the `details` itself reports visible), text directly inside a `content-visibility: hidden` element, text in a `display: contents` element (no box, yet drawn) and content below the fold under `content-visibility: auto` (drawn once scrolled to). On the fixture pages details / details-open / hidden / drawn it gave 0 / 1 / 2 / 2 where 0 / 1 / 0 / 4 is right.
