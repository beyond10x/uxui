---
format: aep.planning-md/3
id: story:own-generator-smallest
kind: story
status: draft
title: An own generator, smallest form
tags:
- in-flight
relations:
- decomposes: epic:generators-and-themes
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: docs/own-generator.md
- confidence: cited
  path: ess/ui/studio.ui.yaml
- confidence: cited
  path: generators/own
revision: 5
---
# Story: An own generator, smallest form

## Outcome

The cost of owning a generator is known from one working page, with a list of what ESS does not hand
an own generator.

## Context

Story 4 of the generator round (`docs/generator-round.md:29-31`). Under way on 2026-11-24 in a
separate worktree (`uxui-own-gen`), which holds the crate `generators/own/` (binary `uxui-gen`),
`docs/own-generator.md`, and a new root `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml`, and
edits `Taskfile.yml` and `AGENTS.md`. Its measurements are recorded in
`docs/generator-round.md:94-105` (one page, 1,807 lines, 1,296 of them re-implementing what ESS does
not hand over). ESS has no resolved form of an `ess-ui/1` document (F3,
`docs/generator-round.md:51-58`; gap 3 of `docs/ess-gaps.md:18-19`), so the generator re-implements
resolution.

Shared surface: `Taskfile.yml` and `AGENTS.md`, which `story:docs-site` and
`story:themes-light-dark` also edit; the root `Cargo.toml`, which every engine story will extend
(`story:cost-spike` waits for this story). It reads `ess/ui/studio.ui.yaml` as its worktree's base
commit holds it, before `story:spec-hardening-pass` lands; the list is taken on that version.

## Acceptance

Running the command `docs/own-generator.md` gives writes the live climb page (`pages/campaigns.climb`)
from `ess/ui/studio.ui.yaml` and its model, and that document lists each piece of ESS semantics the
generator re-implements to do so (resolution of defaults and references, model binding, live channel
semantics, fixture semantics).

## Provenance

Migrated from `docs/generator-round.md`, story 4, lines 29-31. Not in Git yet (untracked on
2026-11-24); dates itself `Status: plan, 2026-11-24` (line 3). Paths read from the story's worktree.
