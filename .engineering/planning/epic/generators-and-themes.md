---
format: aep.planning-md/3
id: epic:generators-and-themes
kind: epic
status: draft
title: 'Generators and themes: the stock generator against an own generator'
revision: 3
---
# Epic: Generators and themes: the stock generator against an own generator

## Outcome

For each UI, uxui knows whether the stock generator (`ess generate ui`) or a generator of its own
serves it better, from measured adjustment costs, and every theme it owns has a light and a dark mode
over generated output.

## Why Now

The owner's position (2026-11-24): a UI generated from `ess-ui/1` is hard to adjust visually and
functionally, and most applications will write their own generator from the specifications
(`docs/generator-round.md:3-6`). `AGENTS.md:40-43` makes the reassessment standing.

## Scope

The round of `docs/generator-round.md` (stories 1-7, findings F1-F5, the slot case study, the studio
decision, the standing question) and `themes/`. Case studies: the studio (`ess/ui/studio.ui.yaml`)
and slot's web UI (`docs/generator-round.md:8-11`).

The source records stories 3, 5 and 6 of the round as done on 2026-11-24: the slot case study and
its classification (`docs/generator-round.md:69-92`), and the change list with both ways measured on
it, which decided the studio for an own generator under two conditions
(`docs/generator-round.md:94-105`). They carry no story here. The slot classification gives uxui
the styling and layout part (`docs/generator-round.md:87-89`): `story:slot-styling-in-themes`.

## Out of Scope

Requirements on `ess-ui/1` are filed with the ESS maintainers and tracked in
`epic:ess-dependencies`. Held until its noun has an ESS home
(`specification-blocker:generator-round-nouns`): the friction ledger (story 1 of the round).

## Risks

Every theme a host adds over generated output has to learn which names the generator already uses;
nothing declares them (F5, `docs/generator-round.md:64-67`).

## Done When

`docs/generator-round.md` records, for the studio and for slot's web UI, stock or own generator with
its reason (`docs/generator-round.md:107-111`).

## Provenance

Migrated from `docs/generator-round.md` and the `themes/` row of `AGENTS.md:26`.

- `docs/generator-round.md`: not in Git yet (untracked on 2026-11-24); dates itself `Status: plan,
  2026-11-24` (line 3); lines 69-105 were added on 2026-11-24 while this migration ran.
- `AGENTS.md`: first written 2026-11-24 · last touched 2026-11-24 · 1 git revision; lines 26 and
  35-43 are uncommitted edits of 2026-11-24.
