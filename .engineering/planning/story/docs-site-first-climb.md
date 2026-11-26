---
format: aep.planning-md/3
id: story:docs-site-first-climb
kind: story
status: implemented
title: 'First real target: climb the documentation landing page'
tags:
- roadmap-step-5
relations:
- decomposes: epic:targets
- depends_on: story:browser-adapter
- depends_on: story:docs-site
- depends_on: story:toy-climb
scope:
- confidence: cited
  path: website
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-25T13:54:14Z", actor: "human:operator", revision: 6, decided_on: {"recorded":{"review_outcome":7}}}
- {from: "proposed", to: "active", at: "2026-11-25T13:54:14Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"review_outcome":7}}}
- {from: "active", to: "implemented", at: "2026-11-26T11:59:19Z", actor: "human:operator", revision: 8, decided_on: {"recorded":{"test_result":3,"review_outcome":7}}}
---
# Story: First real target: climb the documentation landing page

## Outcome

The engine improves a UI other than its studio: the documentation landing page, through a person's
choice among ranked variants.

## Context

Roadmap step 5 (`docs/roadmap.md:41-42`); the worked example of `docs/targets.md:58-75`: 144
variants, six environments, three journeys, hard rules, metrics and KPIs on the browser adapter. The
worked example names two targets, the page `/uxui/` and its hero section (`docs/targets.md:60`);
this story claims the page, and the hero section is a second campaign of the same example, not yet
drafted. Gating the site's next build on the promoted values (`docs/targets.md:74-75`) is ratchet's
gate, `story:ratchet-presets-and-gates`. Every ranking here carries the roadmap's counter to
best-of-n: tie bands, the Pareto frontier, the smallest change, a human calibration set
(`docs/roadmap.md:56-60`).

## Acceptance

The variant a person chose from the ranked frontier of a campaign over the worked example's design
space, run on the browser adapter, is promoted as a pull request carrying a diff to
`website/product.json` and the theme CSS and ratchet evidence that names the campaign.

## Provenance

Migrated from `docs/roadmap.md` (lines 41-42, an uncommitted edit of 2026-11-24) and
`docs/targets.md` (lines 58-75 and 89-93; untracked on 2026-11-24, `Status: design, 2026-11-24`).

## State on 2026-11-24

Exists: PR #3 applied the chosen variant of run 2 of the campaign over the landing page (288
variants, six environments, three journeys, hard rules, a content rule and six ranked KPIs, stage 2
in Chromium) as a diff to `website/product.json` and `website/src/css/landing.css` (new, loaded from
`website/docusaurus.config.ts`), with screenshots and the record in
`docs/campaign-2026-11-24-docs-landing.md`.

Missing: ratchet evidence that names the campaign. The variant is the campaign's own pick by its
tie rule (stage-2 rank 4, within 2 % of the frontier's best score, one source change fewer), one step
off the frontier, not a person's choice among the frontier's entries. Stage 2 ran on spike code, not
on the browser adapter of `story:browser-adapter`.
