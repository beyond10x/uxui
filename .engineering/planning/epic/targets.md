---
format: aep.planning-md/3
id: epic:targets
kind: epic
status: draft
title: 'Targets: the documentation site and the trial target'
tags:
- roadmap-step-5
revision: 2
---
# Epic: Targets: the documentation site and the trial target

## Outcome

The engine improves UIs other than its own studio: first this repository's documentation site on the
browser adapter, then one page or component of an admin console generated from an `ess-ui/1`
document.

## Why Now

The documentation site is the first real target: it is ours, public, small, and has a declared
source (`product.json`) (`docs/targets.md:89-93`; `docs/roadmap.md:41-42`).

## Scope

`docs/targets.md` (the worked example, lines 58-75) and roadmap steps 5 and 6
(`docs/roadmap.md:41-45`).

## Out of Scope

The browser adapter itself (`story:browser-adapter`); results of the trial target stay in its own
repository, and only generic findings come back here (`docs/roadmap.md:43-45`).

## Risks

At about one second per render, the worked example's 2,600 renders take about 45 minutes
exhaustively (`docs/targets.md:71-73`).

## Done When

A variant of the documentation landing page chosen through the engine is promoted as a pull request
with a diff to `website/product.json` and the theme CSS and ratchet evidence, the site's next build
is gated on the promoted values through `story:ratchet-presets-and-gates` (`docs/targets.md:74-75`),
and the trial target has run.

## Provenance

Migrated from `docs/targets.md` and `docs/roadmap.md` (steps 5 and 6, lines 41-45).

- `docs/targets.md`: not in Git yet (untracked on 2026-11-24); dates itself `Status: design,
  2026-11-24` (line 3).
- `docs/roadmap.md`: first written 2026-11-24 · last touched 2026-11-24 · 1 git revision; lines
  41-42 are an uncommitted edit of 2026-11-24.
