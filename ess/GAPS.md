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

"A hard-rule violation prunes" is two steps: `RecordRuleVerdict` records the verdict and
`PruneVariant` (refused without a hard violated verdict) moves the variant to `Pruned`. Between the
two, the variant is not yet pruned. Every later step (enumeration, evaluation, frontier, promotion)
refuses on the hard violated verdict itself, so nothing proceeds in the gap.

### G3. A related-row guard beside a held-state guard

```
[conflicting_declaration] command.uxsearch.evaluation.RecordEvaluationResult.outcomes.bound-hit.when_related: outcome `bound-hit` declares `when_related` beside `when_subject_state`; a branch has one selection authority (hint: keep the related guard and an optional `when:` over the input, or the other key)
```

Instead: `Evaluation` copies the state of its state space (`state_space_state`) when it is
requested, and `RecordEvaluationResult` decides "bound hit means insufficient" from the
evaluation's own field. This works because a state space no longer changes after `Truncated` or
`Complete`, and `RequestEvaluation` refuses an `Enumerating` state space.

## Missing constructs

### G4. Universal quantification over related rows

"Promotion requires every hard rule to pass" needs "for every Active hard rule of the target a
`pass` verdict exists for this variant". ESS has `exists: true|false` over a row set, not "for
all". Expressed: no hard `violated` verdict exists (row set, `exists: false`), plus a `pass`
`GateDecision` from ratchet at stage `promote`. A hard rule with no verdict at all is not caught by
the engine; ratchet's gate catches it if every hard rule is bound as a requirement. Owner decision.

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
- The layout model is only a contract here (`LayoutInput`, `LayoutOutput`); v0 is ESS's terminal
  renderer with cell geometry (decision D1 A), a pixel model later.

## Studio document (ess/ui/studio.ui.yaml)

- No view lists the axis values of the design space a variant was reached in, so the variant page
  lists every axis value (UNMAPPED in the document). A view with a parameter over the variant's
  routes would need a read through a list field.
- A channel with `scope: param` and `session: {per: campaign_id}` delivered no scripted event in
  `ess ui test` (the terminal renderer). The document uses `only_if: matches(params)` on each live
  section instead; the server-side scoping is not declared. Not investigated further.
