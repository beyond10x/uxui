# ESS gaps uxui needs, in priority order

Status: 2026-11-24. Every gap uxui has found in ESS 0.98.0, ordered by what blocks the engine first. The
owner, 2026-11-24: "make sure that all ESS gaps are implemented ASAP". Filed with the ESS maintainers
through the host operator. Styling is not an ESS gap: themes and templates belong to uxui (`AGENTS.md`,
Direction), and so do the UX declarations (`uxui-declarations/1`).

Sources: `ess/GAPS.md` (G1-G11), `docs/roadmap.md` step 2, `docs/generator-round.md` (F1, F3).

## P0: without these the engine cannot exist

1. **UI state API** (roadmap 2a). For a UI state of an `ess-ui/1` document: the list of actions available
   in it, and a cloneable state (or a replay-free way to reach it). Exhaustive search over the state space
   is impossible without it.
2. **Canonical form and digest** (G7, roadmap 2b). A normative canonical byte form and digest of an
   `ess-ui/1` document and of a UI state. "Two routes to one document are one variant" and the evaluation
   cache both rest on it.
3. **Resolved document** (F3). `ess ui load --format json` printing `ess-ui-resolved/1` with every default
   filled in (filed as a draft 2026-11-24; the draft recommends the loader fill in every default).

## P1: needed for evidence and for the studio

4. **Per-step trace** (roadmap 2c). An `ess-ui-test` report with one record per step: the state reached,
   the nodes shown, the geometry read. The engine's evidence and the bench's coverage count read it.
5. **Component harness** (G11, roadmap 2d). Render one component alone at given widths, with fixtures.
   Targets below page scope need it.
6. **Chart order** (F1). `chart` takes `sort: {by, dir}` and live inserts keep the order (filed as a draft
   2026-11-24).
7. **Param-scoped live channel in tests.** A channel with `scope: param` and `session: {per: campaign_id}`
   delivered no scripted event in `ess ui test` (terminal renderer); the studio works around it with
   `only_if: matches(params)`. A defect or a missing feature; not investigated further.
8. **A view read through a list field.** A view with a parameter over a row's list field (a variant's
   routes), so the studio can show the axis values a variant was reached by.
8a. **Tab focus order** (added 2026-11-24). The terminal renderer exposes no keyboard focus order of a UI
   state. Without it focus reachability, focus traps and focus order against reading order (WCAG 2.4.3, 2.1.2, 2.4.11) cannot be
   checked; no existing tool automates them, and on a declared model they are exact graph checks.

## P2: invariants the specification can only state by construction today

9. **For all over related rows** (G4). "Every Active hard rule of the target has a `pass` verdict."
10. **A list-reference guard beside another related guard** (G1).
11. **A related-row guard beside a held-state guard** (G3).
12. **A secondary effect beside a create** (G2).
13. **A guard two relations away** (G8).
14. **Comparison across list elements of two rows** (G5), for Pareto dominance.
15. **A digest computed from named inputs** (G6), for "evaluation is a pure function of its inputs".
16. **A "never reads" invariant across domains** (G10), for "a judgement never changes a gate verdict".
17. **Predicates over an `ess-ui/1` document** (G9), for mutation operator preconditions.
