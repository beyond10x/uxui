---
format: aep.planning-md/3
id: specification-blocker:funnel-nouns
kind: specification-blocker
status: cleared
title: the funnel's lift and projection have no ESS home
relations:
- blocks: epic:engine-slices
- blocks: story:stage-agreement
revision: 5
transitions:
- {from: "open", to: "cleared", at: "2026-11-24T13:14:33Z", actor: "human:operator", revision: 5}
---
# Blocker: the funnel's lift and projection have no ESS home

## What Is Blocked

Drafting the funnel's stages 0, 2 and 3 as stories in `epic:engine-slices`. They introduce nouns no
ESS document in this repository declares, and `docs/targets.md:77-85` does not list them among round
3's changes:

| story | acceptance | nouns | source |
|---|---|---|---|
| Funnel stage 0: lift | A page is lifted, per component, into an `ess-ui/1` document with fixtures of the page's real content and a binding table (node path to CSS selector), and a lift whose render does not reproduce the component's structure is refused, not searched. | LiftedComponent, BindingTable | `docs/targets.md:42, 47-49, 53-54` |
| Funnel stage 2: project | The top k variants of stage 1 are applied to the real page through the binding table and measured on the browser adapter. | BindingTable | `docs/targets.md:44` |
| Funnel stage 3: judge | The top few variants of stage 2 are shown as screenshots side by side, with a model's judgement and a person's choice recorded as judgement records, never as a gate. | (stage 2's projected variants) | `docs/targets.md:45` |

## What Would Clear It

An ESS document (the `uxsearch` specification, or one beside it) that declares the lifted component
and its binding table, passing `ess specify validate`.

## Who Can Clear It

Whoever owns `ess/` once the hardening pass has landed; `story:spec-round-3-target-adapter` is the
nearest place for it.

## What We Are Doing Meanwhile

`story:scaffold-spike` measures whether a lift is faithful at all, without the product's nouns.

## How It Was Cleared

Cleared by `story:spec-round-3-target-adapter`. `ess/domains/design.yaml` declares
`uxsearch.design.LiftedComponent` (a component of a browser page lifted into an `ess-ui/1`
document, with fixtures, a `BindingTable` and the fidelity mismatches), the types `BindingTable`,
`Binding` (node path to `CssSelector`), `FidelityAspect` and `FidelityMismatch`, and the commands
`ProposeLift`, `RecordLiftFidelity` and `RegisterLiftedDocument`. A lift whose render does not
reproduce the component's structure moves to `Refused`, and `RegisterLiftedDocument` refuses it
(`lift-not-verified`), so no target is declared on it and no campaign searches it. The scenario
`ess/scenarios/lift-must-reproduce-the-component.yaml` holds both, and
`ess specify validate --path ess` answers `uxsearch v1 — 10 file(s), 12 scenario(s), valid`.

Still open for the owner, marked `UNMAPPED:` in the specification: whether one node binds one
element or several, and how a browser journey's required nodes are named (a CSS selector, or a
node path through the binding table). Stage 3's judgement records already exist
(`uxsearch.promotion.JudgementRecord`).
