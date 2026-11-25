---
format: aep.planning-md/3
id: story:ux-browser-overflow-hidden-content
kind: story
status: draft
title: Overflow ignores text that is not drawn
relations:
- decomposes: epic:engine-slices
revision: 2
---
# Story: Overflow ignores text that is not drawn

## Outcome

Text inside a closed `details` element, or any other content the page does not draw, is not counted as overflow.

## Context

Consumer: the second consumer (a control plane web application), run on 0.5.0 on 2026-11-25. On its Asks page a `p.full` inside a closed `details.more` reported "right edge 895 past its parent at 892" at 1536x754 and "911 past 907" at 1920x1080. Checked against the source at d060e30: `src/probes/overflow.js` skips a text run only when its parent has `visibility: hidden`; content of a closed `details`, `content-visibility: hidden` or `hidden=until-found` is measured.

## Acceptance

The overflow probe skips a text run or element for which `checkVisibility({contentVisibilityAuto: true, visibilityProperty: true})` is false; a fixture page with a long line inside a closed `details` counts 0 overflow, where 0.5.0 counted 1, and the same page with the `details` open counts 1; llm's documentation build gives its example document's values unchanged.
