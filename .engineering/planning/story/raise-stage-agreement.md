---
format: aep.planning-md/3
id: story:raise-stage-agreement
kind: story
status: implemented
title: Raise stage agreement on the docs-landing campaign
relations:
- decomposes: epic:engine-slices
- depends_on: story:stage-agreement
scope:
- confidence: cited
  path: docs/campaign-2026-11-24-docs-landing.md
- confidence: cited
  path: tools/scaffold/agreement.mjs
- confidence: cited
  path: tools/scaffold/campaign.mjs
- confidence: cited
  path: tools/scaffold/lift/docs-landing/cells-model.mjs
- confidence: cited
  path: tools/scaffold/lift/docs-landing/space.json
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:15Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:15Z", actor: "human:operator", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-25T14:06:36Z", actor: "human:operator", revision: 5, decided_on: {"recorded":{"test_result":1}}}
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
