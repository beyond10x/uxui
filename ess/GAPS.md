# What ESS 0.98.0 (format ess/37) cannot express in this specification

Each gap names the exact refusal or the missing construct, and what the specification does instead.
The owner decides each one before anything is generated.

## Refused constructs

### G1. A guard over a list of references beside another related guard

`ess specify validate` refused `StartCampaign` and `AdmitToFrontier`:

```
[conflicting_declaration] command.uxsearch.search.StartCampaign.outcomes.unknown-journey.when_related: `uxsearch.search.StartCampaign` guards on the rows the list `input.journeys` names and on another related row; a command guarding on a list of references reads no other related row in this cut (hint: split the guards across commands, or drop one)
error[ESS-COMMAND-004]: `uxsearch.search.AdmitToFrontier` guards on the rows the list `input.evaluation_keys` names and on another related row; a command guarding on a list of references reads no other related row in this cut
```

Instead: a missing journey, environment, KPI or evaluation key is an `external` branch
(`unknown-member`, `evaluation-missing`). The row-set guards (`entity`, `where`, `exists`) that
check the members belong to the target, and that the listed evaluations are Computed, stay in the
specification.

### G2. A secondary effect beside a create

```
[unsupported_construct] command.uxsearch.rules.RecordRuleVerdict.outcomes.pruned.affects: outcome `pruned` creates its subject and declares `affects:`; a secondary effect is admitted beside `moves:` and `updates:`, and, from ess/23, `deletes:`, whose subject exists before the outcome (hint: move `affects:` to the branch that moves, updates or deletes an existing row)
```

Instead: the variant has no `Pruned` state. "Pruned" is the `Recorded` hard `violated` verdict
itself: every later step (enumeration, evaluation, frontier, promotion) refuses on it, and
`RetireRule` voids the rule's verdicts, so a retired rule prunes nothing.

### G3. A related-row guard beside a held-state guard

```
[conflicting_declaration] command.uxsearch.evaluation.RecordEvaluationResult.outcomes.bound-hit.when_related: outcome `bound-hit` declares `when_related` beside `when_subject_state`; a branch has one selection authority (hint: keep the related guard and an optional `when:` over the input, or the other key)
```

Instead: `Evaluation` copies the state of its state space (`state_space_state`) when it is
requested, and `RecordEvaluationResult` decides "bound hit means insufficient" from the
evaluation's own field. This works because a state space no longer changes after `Truncated` or
`Complete`, and `RequestEvaluation` refuses an `Enumerating` state space.

## Missing constructs

### G4. Universal quantification over related rows (closed)

"Promotion requires every hard rule to pass" is "no Active hard rule of the target without a
Recorded `pass` verdict for this variant". ESS 0.98.0 states it as a row set over `DesignRule` whose
`where` holds `not: {exists: {in: {entity: RuleVerdict}, ...}}` (`hard-rule-unchecked` in
`AcceptPromotion` and `AcceptTradeOff`). Kept here because the earlier text said it could not be
expressed.

### G5. Pareto dominance

`AdmitToFrontier` and `MarkDominated` need "at least as good on every KPI, better on one". ESS has
no comparison across list elements of two rows. Dominance is an `external` branch (`dominated`) of
the Searcher; `MarkDominated` records the dominating variant.

### G6. Evaluation is a pure function of its input digests

The evaluation key is the digest of its inputs (variant, state space incl. fixtures, environment,
layout model and font table, metric catalogue, UX declarations). The specification states it by
identity (one Evaluation per key, a repeat request preserves) and by refusal (a second result with
a different digest is `NotDeterministic`). ESS cannot state that a digest is computed from named
inputs, nor check it.

### G7. Canonical form of an ess-ui/1 document

"Two routes to one canonical document are one variant" is expressed by identity: `Variant` is
keyed by `variant_digest`, and a second route is appended to `routes`. ESS defines no canonical
form or digest of an ess-ui/1 document (`ess ui load` prints the resolved form, not a normative
canonical byte form). Marked UNMAPPED in `domains/design.yaml`.

### G8. A guard two relations away

`AddAxisValue` should be refused when the axis's design space is `Frozen` (AxisValue → Axis →
DesignSpace). `when_related` reads one hop. Marked UNMAPPED; the Frozen space is enforced at
`AddAxis`, `BindComponent` and `StartCampaign` only.

Round 3 (the target adapter) has two more checks that sit two or three relations away, each an
`external` branch the engine answers:

- `AddAxisValue/adapter-mismatch`: operator applications on a value of a `browser` target, or source
  patches on an `ess-ui` target (AxisValue → Axis → DesignSpace → Target).
- `RequestEvaluation/renderer-of-another-adapter`: the renderer (`LayoutModel.adapter`) is not the
  adapter of the environment's target (Environment → Target).

### G9. Preconditions over the document

Mutation operators have preconditions over the base document (a node exists, a node is not a host
slot). ESS has no predicate over an ess-ui/1 document. Held as text on `MutationOperator`; an
operator that fails its precondition is refused as an external defect at `MaterializeVariant`.

### G10. A judgement never changes a gate verdict

Expressed by construction only: `RecordGateDecision` takes no judgement input, and no command reads
`JudgementRecord`. ESS has no "never reads" invariant across domains to assert it.

### G11. Not in ess-ui/1 today

- UX declarations (required nodes, always-visible nodes, completion signal, cue node): kept in a
  `uxui-declarations/1` file beside the ess-ui/1 document (decision D2 A), registered by
  `RegisterUxDeclarations`; to be proposed to ESS later.
- A component harness (a component rendered alone at given widths): `Target.harness_widths_px`;
  ess-ui/1 has no harness construct.
- The layout model is named by version only (`LayoutModel`); v0 is ESS's terminal renderer with
  cell geometry (decision D1 A), a pixel model later. Its input and output are not modelled.

### G12. A product of list counts

"The campaign's evidence covers every journey × environment cell" needs
`input.journeys.count * input.environments.count` in an expression; `{expr: ...}` reads no count of
an input list. `AdmitToFrontier` refuses an empty or repeated `evaluation_keys` and a
`kpi_values` that misses a KPI of the campaign; full coverage is the external branch
`evidence-incomplete` (UNMAPPED in `domains/search.yaml`).

### G13. A system actor with attributes

A command that records its caller (`accepted_by: {caller: principal}`) needs every granted actor to
declare the attribute, and a `system: true` actor declares none. `AcceptancePolicy` is therefore a
non-system actor. The engine's workers (`Enumerator`, `Evaluator`, `Searcher`, `Assessor`,
`Ratchet`) are non-system too: synthesis refuses a system-issued command it cannot await
(`ESS-SYNTH-023`), which left their commands without a scenario.

### G14. Synthesis time

`ess verify conform synthesize` (and so `ess specify validate --format json`, which reports
completeness) took 2 s on the specification before the 2026-11-24 hardening and takes 1000 s
after it. Profiling shows the time in the arrangement search (scenario steps cloned).
No single domain causes it; removing the row-set guards of every refusal branch brought it to
59 s. The repository gate does not run synthesis. An ESS performance gap, not yet filed upstream;
the authored scenarios in `scenarios/` run in under 2 s.

Round 3 (the target adapter, 2026-11-24) adds no row-set guard. On one host synthesis went from
11 s (135 scenarios, 286 refusals) to 16 s (161 scenarios, 291 refusals); the five new refusals sit
behind routes that were refused before (`AddAxis/added`, `RequestEvaluation/requested`,
`ProposePromotion/proposed`). A first draft guarded `RegisterBaseDocument` on an Optional lift
input; synthesis then reached no `registered` branch, refused every route behind it and took 24 s.
The lift registration became its own command (`RegisterLiftedDocument`), as the cache lookups did.

### G15. Fixtures are not typed against the model

`ess ui check --model` and `ess ui test --model` pass fixtures whose rows contradict the view
types (a digest where a list is declared, a missing struct field, a renamed field). The fixtures
were corrected by hand (review R22); no check holds them.

### G16. One child row per parent and key

A promotion holds one `SourceDiff` per source file. Refusing a second diff of the same file for
the same promotion needs a row-set guard over `SourceDiff` (`promotion_id` and `path` equal to the
input), the construct G14 names as the cost of synthesis. `RecordSourceDiff` does not refuse it;
the engine writes each file once.

## Studio document (ess/ui/studio.ui.yaml)

- No view lists the axis values of the design space a variant was reached in, so the variant page
  lists every axis value (UNMAPPED in the document). A view with a parameter over the variant's
  routes would need a read through a list field.
- Compare shows verdict and evaluation tables, not the variants in one state and environment: no
  view exposes a laid-out result (UNMAPPED in the document).
