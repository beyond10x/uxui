---
format: aep.planning-md/3
id: dependency-blocker:ratchet-format
kind: dependency-blocker
status: open
title: ratchet's format has not settled
relations:
- blocks: story:ratchet-presets-and-gates
revision: 1
---
# Blocker: ratchet's format has not settled

## What Is Blocked

`story:ratchet-presets-and-gates`: roadmap step 7 starts "when ratchet's format settles"
(`docs/roadmap.md:46`).

## What Would Clear It

A ratchet release whose requirement, evidence and preset formats are declared stable, answering the
four gaps uxui sent it: trade-offs, result scope, exact vs sampled, finding identity
(`docs/roadmap.md:13-14`).

## Who Can Clear It

The ratchet maintainers (`beyond10x/ratchet`, in design on 2026-11-24).

## What We Are Doing Meanwhile

`story:ux-static-slice` writes evidence in ratchet's draft shape; `presets/web-ux.ratchet.yaml` stays a
draft.
