---
format: aep.planning-md/3
id: story:ratchet-presets-and-gates
kind: story
status: draft
title: Ratchet presets and gates
tags:
- roadmap-step-7
relations:
- decomposes: epic:ratchet-integration
scope:
- confidence: cited
  path: presets/web-ux.ratchet.yaml
revision: 3
---
# Story: Ratchet presets and gates

## Outcome

uxui's metric catalogue holds merges and promotions to ratchet's gates.

## Context

Roadmap step 7 (`docs/roadmap.md:46-47`), once ratchet's format settles
(`dependency-blocker:ratchet-format`). The preset family is drafted in
`presets/web-ux.ratchet.yaml`.

## Acceptance

A promotion whose metric is worse than the limit a preset of `presets/web-ux.ratchet.yaml` sets is
refused by ratchet's gate at promote and fails ratchet's gate at merge, and the same promotion within
the limit passes both.

## Provenance

Migrated from `docs/roadmap.md`, step 7, lines 46-47. First written 2026-11-24 · last touched
2026-11-24 · 1 git revision.
