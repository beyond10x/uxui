---
format: aep.planning-md/3
id: story:ux-search-version-names-formats
kind: story
status: draft
title: ux-search --version names the campaign formats
relations:
- decomposes: epic:engine-slices
scope:
- confidence: inferred
  path: engine/search/src/main.rs
revision: 3
---
## Outcome

`ux-search --version` tells apart two engines whose campaign formats differ.

## Context

The second trial run of `ux-search campaign` (`docs/trial-2026-11-26-outside-target.md`) found the
same `--version` line for two engines with different measurements formats. The campaign follow-ups
(`story:campaign-run-follow-ups`) renamed the formats (`ux-search-campaign/2`,
`ux-search-campaign-measurements/3`) but left `--version` as it was; a studio test pins its first
line.

## Acceptance

- `ux-search --version` prints the version on its first line as before, and on further lines the
  declaration and measurements formats it writes and the ones it reads.
- The pinned first line stays; a test holds the format lines to the constants in the source.
