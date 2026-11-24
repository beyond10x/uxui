---
format: aep.planning-md/3
id: story:ux-browser-values-output
kind: story
status: implemented
title: ux-browser writes the consumer's values file
relations:
- decomposes: epic:engine-slices
- depends_on: story:browser-adapter
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T19:17:27Z", actor: "human:operator", revision: 3, decided_on: {"asserted":{"test_result":1,"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-24T19:17:28Z", actor: "human:operator", revision: 4, decided_on: {"asserted":{"test_result":1,"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-24T19:17:28Z", actor: "human:operator", revision: 5, decided_on: {"asserted":{"test_result":1,"review_outcome":1}}}
---
# Story: ux-browser writes the consumer's values file

## Outcome

A consumer's docs UX gate reads its per-scenario values and its hard-gate failures straight from
ux-browser, with no conversion script of its own, and its round mode runs on a second checks
configuration of the same tool.

## Context

First consumer: llm (story:docs-ux-layout-checks-from-ux-browser on llm's side). Wave 3 delivered a
patch on uxui 0.2.0 in which `ux/page-checks.sh` (POSIX sh + jq) turns ux-browser's
ratchet-evidence/1 document into llm's `page-checks.json`. llm's rules forbid that script; llm picked
option A on 2026-11-24: ux-browser writes the file (uxui 0.3.0). llm's further conditions:

1. The viewport list and the settle time come from the configuration (already true in 0.2.0).
2. Round mode gets a second configuration: viewports 1440, 1280, 1024, 768, 390; settle 750 ms; a
   `layout_shift.load` row. A part that ux-browser cannot do is named with its cause.
3. One pin file for tag and SHA-256 on llm's side (the revised patch, not this repository).

## Acceptance

On the recorded evidence of wave 3 (`ux/tests/fixtures/page-checks.evidence.json` of the patch),
`ux-browser values` writes a file byte-equal to the one `page-checks.sh` wrote from it, and exits 1
on the same problems; `ux-browser checks` with the values output does the same in one run; a checks
configuration declaring `layout_shift.load` reports it per page. Released as uxui 0.3.0.
