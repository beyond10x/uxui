---
format: aep.planning-md/3
id: specification-blocker:generator-round-nouns
kind: specification-blocker
status: open
title: the generator round's friction ledger has no ESS home
relations:
- blocks: epic:generators-and-themes
revision: 3
---
# Blocker: the generator round's friction ledger has no ESS home

## What Is Blocked

Drafting story 1 of the round in `epic:generators-and-themes`. It introduces a typed record that no
ESS document in this repository declares, and the planning rule is that such a noun is modelled
before stories are written around it. The classification is kept here:

| story | acceptance | nouns | source |
|---|---|---|---|
| Friction ledger | One ledger holds a record per friction point (page and node path, the change asked for, what the generator did, the workaround, its cost) from the studio and from slot's file, in slot's format. | FrictionPoint | `docs/generator-round.md:22-24` |

Stories 3, 5 and 6 of the round (change list, both ways on it, slot case study) were held here
too when the store was created; the source has since recorded them as done
(`docs/generator-round.md:69-105`), so no story is held for them.

## What Would Clear It

An ESS document in this repository that declares FrictionPoint with the fields above, passing
`ess specify validate`; the story then cites it.

## Who Can Clear It

Whoever owns `ess/` once the hardening pass (`story:spec-hardening-pass`) has landed; `ess/` was
being changed by that pass on 2026-11-24, so the migration did not draft the domain.

## What We Are Doing Meanwhile

The other stories of `epic:generators-and-themes` need no friction record.
