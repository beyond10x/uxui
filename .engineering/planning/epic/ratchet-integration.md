---
format: aep.planning-md/3
id: epic:ratchet-integration
kind: epic
status: draft
title: Ratchet integration
tags:
- roadmap-step-7
revision: 1
---
# Epic: Ratchet integration

## Outcome

uxui's metric catalogue gates merges and promotions through ratchet, and one of the organization's
UI-loop copies runs on ratchet and uxui instead of its own baseline file.

## Why Now

uxui offered to be the first consumer of ratchet's "ESS UI behavior" integration
(`docs/roadmap.md:13-14`). It waits on ratchet's format (`docs/roadmap.md:46`,
`dependency-blocker:ratchet-format`).

## Scope

Roadmap step 7 (`docs/roadmap.md:46-48`): presets from `presets/web-ux.ratchet.yaml`, gates on merge
and promote, the migration of the smallest UI-loop copy.

## Out of Scope

The evidence records the engine writes (`story:ux-static-slice`); the bench's results as ratchet
evidence (held, `specification-blocker:bench-nouns`).

## Risks

ratchet is in design (`docs/roadmap.md:13`); its format may change under the presets.

## Done When

The smallest UI-loop copy runs on ratchet and uxui with no baseline file of its own.

## Provenance

Migrated from `docs/roadmap.md` (step 7, lines 46-48; state, lines 13-14). First written 2026-11-24 ·
last touched 2026-11-24 · 1 git revision.
