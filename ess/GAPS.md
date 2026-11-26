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

`when_related: {via: ...}` reads one hop. `AddAxisValue` on an axis of a `Frozen` design space
(AxisValue → Axis → DesignSpace) is closed: it is a row set over `DesignSpace` whose `where` holds
an `exists` over `Axis` (outcome `frozen`, scenario `axis-value-is-added-while-the-space-is-open`).
Kept here because the earlier text said the Frozen space was enforced at `AddAxis`,
`BindComponent` and `StartCampaign` only.

Round 3 (the target adapter) has two more checks that sit two or three relations away, each an
`external` branch the engine answers:

- `AddAxisValue/adapter-mismatch`: operator applications on a value of a `browser` target, or source
  patches on an `ess-ui` target (AxisValue → Axis → DesignSpace → Target).
- `RequestEvaluation/renderer-of-another-adapter`: the renderer (`LayoutModel.adapter`) is not the
  adapter of the environment's target (Environment → Target).

One check is not made at all: `RecordMetricEvidence` does not compare the requirement's target
with the target of the evaluated variant (Evaluation → Variant → Target). The engine binds
requirements per target (UNMAPPED in `domains/ratchet.yaml`).

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

Round 2 (scenarios, 2026-11-26, ess 0.99.2) adds 37 scenarios and 17 refusing branches (16 over
an input an invariant bounds, and `DeclareJourney/script-refused`); the refusals go from 289 to
291. The time of `ess specify validate --format json` stays inside the spread of one loaded host:
44.9 s and 51.8 s on the model before, 46.5 s with the first 33 scenarios (three runs back to
back) and 46.3 s at the end of the round. The 50 authored scenarios are validated, compiled and
run on the interpreted target in about 1 s.

Closing G20 (2026-11-26, ess 0.99.2) rewrites ten guards over a command's own row as row sets
that select the row by its key, the construct the first paragraph names as the cost. Synthesis
time did not move with them. On one host, each measured once:
`ess verify conform synthesize --path ess --scenarios ess --suite-format 5` took 27.9 s before
and 24.9 s after; the same command writing the suite with `--out` took 64.8 s before and 46.6 s
after; `ess specify validate --format json` took 57.5 s before and 47.9 s after. The refusals
go from 291 to 286 and the synthesized scenarios stay 171.

### G15. Fixtures are not typed against the model

`ess ui check --model` and `ess ui test --model` pass fixtures whose rows contradict the view
types (a digest where a list is declared, a missing struct field, a renamed field). The fixtures
were corrected by hand (review R22); no check holds them.

### G16. One child row per parent and key

A promotion holds one `SourceDiff` per source file. Refusing a second diff of the same file for
the same promotion needs a row-set guard over `SourceDiff` (`promotion_id` and `path` equal to the
input), the construct G14 names as the cost of synthesis. `RecordSourceDiff` does not refuse it;
the engine writes each file once.

### G17. An aggregation over the rows a list names

`FrontierEntry.kpi_values` should hold, for each KPI of the campaign, the KPI's aggregation over
the metric values of the evaluations `evaluation_keys` names. No expression computes a value from
the rows a list of references names. The Searcher supplies the values; `AdmitToFrontier` refuses a
`kpi_values` that misses a KPI of the campaign (UNMAPPED in `domains/search.yaml`). Carried over
from the marker; not probed against the CLI in round 2.

### G18. A guard over a key inside each struct of a list

`Kpi.terms` is a list of structs, each naming a metric key. Every term's metric should be in the
catalogue, not deprecated, not a diagnostic, and valid for the target's scope. A list-of-references
guard reads a list of keys, not a key inside each struct of a list. `DeclareKpi` answers it as the
`external` branch `unknown-metric` (UNMAPPED in `domains/metrics.yaml`). Carried over from the
marker; not probed against the CLI in round 2.

The steps of a journey's script are the same shape: what the browser adapter refuses in a script
(no `state` step, a state named twice, a `type` step without text, a key it does not press) is a
rule over each struct of `JourneyScript.steps`. `DeclareJourney` answers it as the `external`
branch `script-refused`; the adapter's reader (docs/browser-adapter.md, Journeys) is the check.

### G19. The completeness count reads no authored scenario (intended)

`ess specify validate --format json` reports `completeness.unscenarioed`: the constructs synthesis
gives no scenario. An authored scenario that reaches such a construct and names it under `covers:`
(`ess-scenario/7`) does not lower the count. Measured with ess 0.99.2: 289 with the 13 scenarios
before round 2 and 289 with 46; 291 with the 50 at the end of round 2, whose refusing branches
below add constructs synthesis reaches no route to; 286 once G20 was closed.

The ESS maintainers answered that this is by design: `validate` does not read authored scenarios.
Not a gap to close in ESS; kept here because the count is not the measure of this specification.

The count that reads `covers:` is the last line of
`ess verify conform synthesize --path ess --scenarios ess --suite-format 5`, which exits 0 when
authored scenarios cover every refusal:

```
221 selected scenario(s), 50 authored source(s), 286 refusal occurrence(s), 286 covered by authored
```

Instead: round 2 is measured by that line. The 286 refusals name 286 constructs, and authored
scenarios cover all of them. Before G20 was closed the line read 291 refusals with 286 covered,
and the command exited 1.

Nine refusals `ESS-SYNTH-025` were open in the model before, not gaps of ESS: a branch copied an
input into a row, no guard kept the input inside an invariant of the entity, and the command
declared no refusal for it. The interpreted target ended such a call as an error (`ESS-CF-TARGET`,
"the model's own outcome leaves ... violating its invariant"), not as a refusal. Round 2 closed
them: every input an invariant bounds has a refusing branch that names it, under one error per
domain (`MeasureNotPositive` in `DeclareEnvironment`; `InvalidVersion` in `DefineMetric`;
`ValueOutOfRange` in `EnumerateStateSpace`, `RecordUiState`, `CompleteStateSpace` and
`TruncateStateSpace`; `NegativeCount` in `RecordProgress` and `RecordClimbReading`). No
`ESS-SYNTH-025` is left. Synthesis writes the scenarios of five of the sixteen branches itself;
authored scenarios witness all sixteen.

### G20. A missing-row branch that validation demands and no input reaches (closed in the model)

Closed on 2026-11-26 with ess 0.99.2, by the form at the end of this section. What was open:

A command that moves the row its input key addresses and guards on that row's fields
(`when_related: {via: input.<key>, predicate: ...}`) must declare the branch for a missing row.
Without it:

```
error[ESS-COMMAND-005]: `uxsearch.journeys.CompleteStateSpace` reads the related row's fields, and a missing row makes every predicate over it unknown: no branch answers when the row does not exist
  help: declare the branch taken when it does not: `when_related: {via: input.state_space_key, exists: false}`
```

The interpreted target (`ess verify conform run --target interpreted`, 0.99.2) never takes that
branch: for a key no row has, the command answers `wrong-state` with the entity's state-conflict
error before it reads a related row. `unknown_instance: true` is the branch that answers there
(`ReuseStateSpace/no-state-space`, `PinPolicy/no-campaign`, `ReweighKpi/no-kpi`). It does not stand
in for the demanded branch (the refusal above stays), and not beside it on one branch:

```
[conflicting_declaration] command.uxsearch.journeys.CompleteStateSpace.outcomes.no-state-space.when_related: outcome `no-state-space` declares `when_related` beside `unknown_instance`; a branch has one selection authority (hint: keep the related guard and an optional `when:` over the input, or the other key)
```

Five branches were declared that way and had no scenario: `CompleteStateSpace/no-state-space`,
`TruncateStateSpace/no-state-space`, `RecordGateDecision/no-promotion`,
`AcceptPromotion/no-promotion` and `AcceptTradeOff/no-promotion`. A caller that named an unknown
key got `StateSpaceStateConflict` or `PromotionStateConflict`, not the not-found error the branch
declares, and the synthesized suite said so: its scenario `<command>/outcome/wrong-state` sent "an
identity no record carries" and expected `wrong-state`.

The ESS maintainers' answer: the model can reach the not-found outcome. The command reads its own
addressed row through `when_related`; write that read as `when_subject: {predicate: ...}` and
declare `unknown_instance: true` in place of the `exists: false` branch. On a reduction with one
entity and one guard over the addressed row this holds with 0.99.2: the model validates, synthesis
writes the not-found scenario and the interpreted target passes it. Two things keep that form out
of these five commands:

- Each of the five also guards on a row set (`count-below-recorded`, `unevaluable`,
  `evidence-of-another-variant`, `hard-rule-traded`, `not-on-frontier`, `hard-rule-not-passed`,
  `hard-rule-unchecked`), and a `when_subject` guard beside a row set is refused:

  ```
  error[ESS-COMMAND-009]: `uxsearch.journeys.CompleteStateSpace` selects on a row set (`when_related: {entity, where, …}`) and on a `when_subject` guard; one command reads one kind of related row in this cut
    help: guard the command on the row set alone, or move the other guard to a command of its own
  ```

- A `when_subject` refusal answers before `wrong-state`. On the reduction a closed row sent a
  count over its bound answered the bound error, where the `when_related` form answered
  `wrong-state`. The scenarios `state-space-closes-once`,
  `accepted-promotion-is-applied-as-the-next-revision` and
  `promotion-is-rejected-or-superseded-once` claim the state conflict there. Naming the source
  state in the predicate (`all: [state == Enumerating, ...]`) keeps the order.

Instead, the form the refusal's help names: each of the five commands declares
`unknown_instance: true` for a key no row has, and each of its guards over the addressed row is a
row set over the row's entity that selects the row by its key:

```yaml
      - name: no-promotion
        unknown_instance: true
        error: uxsearch.promotion.PromotionNotFound
      - name: variant-mismatch
        when_related:
          entity: uxsearch.promotion.Promotion
          where:
            all:
              - promotion_id == input.promotion_id
              - variant_digest != input.variant_digest
          exists: true
        error: uxsearch.promotion.VariantMismatch
```

Ten guards changed (`over-bound` twice, `variant-mismatch` three times, `trade-off`,
`gate-not-passed` twice, `not-a-trade-off`, `no-gains`). No outcome and no error changed its name.
A key no row has now answers `no-state-space` with `StateSpaceNotFound` or `no-promotion` with
`PromotionNotFound`; a row set answers after `wrong-state`, so a closed state space and a decided
promotion answer their state conflict first, as before. Synthesis writes the five not-found
scenarios itself, in place of the five `<command>/outcome/wrong-state` scenarios above, and the
interpreted target passes them; the `wrong-state` branches are witnessed by the authored scenarios
that claim `<entity>/state/<State>/refuses/<command>`. No other command of `domains/` reads its
own addressed row through `when_related`.

## UNMAPPED markers left in `ess/`

Every marker left after round 2, by file and line, with the gap that holds it or the question the
owner answers. Round 2 resolved six from the repository's documents and code: the binding table
(one selector and one element per node), the base of a `browser` target (the source files the
design space patches), the `composition` scope, the page of a browser state (its URL path), the
source patch (`SourcePatch` follows the engine's patch: RFC 6902 operations on a JSON source,
custom-property values per selector on a CSS source) and the scripted journey (`JourneyScript`
follows the browser adapter: a page, its steps and a wait limit).

| marker | what is open | held by |
|---|---|---|
| `domains/design.yaml:78` | the canonical form of an ess-ui/1 document | G7 |
| `domains/design.yaml:442` | what locates a section or a component of a `browser` target on its page | owner |
| `domains/design.yaml:501` | the top-k frontier entries of a source campaign become an axis and its values at `FreezeDesignSpace` | G2 |
| `domains/design.yaml:587` | an operator's preconditions over the document, held as text | G9 |
| `domains/metrics.yaml:78` | `MetricValue.at` holds four kinds of subject in one String | owner |
| `domains/metrics.yaml:146` | every term of a KPI names a usable metric | G18 |
| `domains/promotion.yaml:157` | the form of one file's diff | owner |
| `domains/promotion.yaml:159` | `Promotion.diff` and the one `SourceDiff` of an `ess-ui` target state one change twice | owner |
| `domains/promotion.yaml:236` | who authorizes an acceptance policy, and for which targets | owner |
| `domains/ratchet.yaml:559` | the requirement's target is the evaluated variant's target | G8 |
| `domains/rules.yaml:27` | the rule catalogue: its checks and their arguments | owner |
| `domains/search.yaml:242` | each KPI value is the KPI's aggregation over the evaluations | G17 |
| `domains/search.yaml:666` | a journey, environment or KPI the input names exists (`unknown-member`) | G1 |
| `domains/search.yaml:1173` | an evaluation key exists and is of the campaign's journeys and environments (`evaluation-missing`) | G1 |
| `domains/search.yaml:1185` | one evaluation per journey and environment (`evidence-incomplete`) | G12 |
| `domains/search.yaml:1197` | no OnFrontier entry dominates the variant (`dominated`) | G5 |
| `ui/studio.ui.yaml:175` | no view lists only the campaign's KPIs (a filter over a list field) | this table; the Studio section below has no entry for it yet |
| `ui/studio.ui.yaml:270` | no view lists the axis values of the design space a variant was reached in | Studio document, below |
| `ui/studio.ui.yaml:309` | no view holds a variant's laid-out result per UI state and environment | Studio document, below |

## Studio document (ess/ui/studio.ui.yaml)

- No view lists the axis values of the design space a variant was reached in, so the variant page
  lists every axis value (UNMAPPED in the document). A view with a parameter over the variant's
  routes would need a read through a list field.
- Compare shows verdict and evaluation tables, not the variants in one state and environment: no
  view exposes a laid-out result (UNMAPPED in the document).
