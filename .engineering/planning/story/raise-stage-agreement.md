---
format: aep.planning-md/3
id: story:raise-stage-agreement
kind: story
status: draft
title: Raise stage agreement on the docs-landing campaign
relations:
- decomposes: epic:engine-slices
- depends_on: story:stage-agreement
revision: 1
---
# Story: Raise stage agreement on the docs-landing campaign

## Outcome

Stage 1 (cell model) orders the docs-landing variants like stage 2 (browser).

## Context

`story:stage-agreement` made the campaign record how far the two stages agree. Run 2 of the campaign
has Kendall's tau-b 0.059 (run 1: 0.476), over the 31 variants both stages scored
(`docs/campaign-2026-11-24-docs-landing.md`, section "Stage agreement"). The cell renderer cuts text
to one line, so stage 1 ranks the short lede below the long one and the browser the other way round;
it does not see density or heading scale (same document, "What the funnel got wrong", items 1 and 2).
The document's lessons name wrapping at the target width as the first fix to the layout model and
drawing spacing and heading size as the second.

## Acceptance

On run 2's 31 variants measured in both stages, Kendall's tau-b between stage 1's score and stage 2's
score is at least 0.6, recorded in the campaign document.
