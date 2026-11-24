# Review: ESS specification `uxsearch` v1

Date: 2026-11-24. Reviewer: an agent that did not write the specification. Scope: `ess/` (system,
8 domains, `ui/studio.ui.yaml` with fixtures and tests, `GAPS.md`) against `docs/brief.md`,
`docs/concept.md`, `docs/state-space.md`, `docs/metrics.md`, `docs/ratchet.md`, `docs/roadmap.md` and
ratchet's design (`ratchet/docs/design.md` in the ratchet repository). Read-only on `ess/`.

**Verdict: needs revision.** 3 blockers, 19 major, 15 minor.

The specification validates, and its domain split follows the docs. Three of the brief's invariants
can still be broken through a path the model admits: frontier admission, trade-off acceptance and
unevaluable states. The ratchet boundary mirrors ratchet's state instead of exporting evidence, and
the evidence lacks fields ratchet's design requires.

## Checks run (ESS 0.98.0)

| command | result |
|---|---|
| `ess specify validate --path ess` | `uxsearch v1 — 10 file(s), valid`, exit 0 |
| `ess specify validate --path ess --format json` | completeness: 156 unscenarioed, 0 outside, 51 unanswered (26 mixed-caller witnesses, 17 `grant/denied` not synthesized, 6 refusals observed only through views, 1 unexercised type `NodeBox`, 1 command with no not-found answer: `ReweighKpi`) |
| `ess ui check --path ess/ui/studio.ui.yaml --model ess` | 0 errors, 1 warning (`unmapped_reported`, axes); coverage 26 of 85 views and commands |
| `ess ui test --path ess/ui/studio.ui.yaml [--model ess] ess/ui/tests/studio.yaml` | 3 passed, 0 failed, with and without `--model` |

## Findings

### Blockers

**R1 (blocker): the frontier admits a variant on empty or partial evidence.**
`ess/domains/search.yaml:680-692`: the `insufficient-evidence` guard asks only whether some listed
evaluation is not Computed. With `evaluation_keys: []` the guard is vacuous and `admitted`
(`:730-746`) runs. Nothing checks that the keys cover the campaign's journeys × environments
(`:73-79`), so a variant can enter on its one good environment. `kpi_values` is free input
(`:626-627, :737`), not derived from the evaluations. This breaks invariant 3 (insufficient evidence
never passes).
Fix: refuse an empty or incomplete key set (a count guard over `evaluation_keys`, external until ESS
can check coverage), and derive `kpi_values` from the evaluations or mark them UNMAPPED.

**R2 (blocker): a trade-off is accepted on insufficient evidence.**
`ess/domains/promotion.yaml:459-472` passes when `{any: [verdict == pass, trade_off_only == true]}`.
`RecordGateDecision` (`ess/domains/ratchet.yaml:555-605`) stores `trade_off_only: true` beside
`verdict: insufficient_evidence` or a failing hard limit without refusal, even though the comment at
`:572-573` says it means "nothing is insufficient". A gate decision `{verdict:
insufficient_evidence, trade_off_only: true}` makes `AcceptTradeOff` succeed.
Fix: guard `verdict == pass or (verdict == fail and trade_off_only)`, and refuse `trade_off_only`
with a verdict other than `fail` in `RecordGateDecision`.

**R3 (blocker): unevaluable states do not make an evaluation insufficient.**
`ess/domains/journeys.yaml:167-172` says "an evaluation over a state space holding such a state is
insufficient evidence". `RecordEvaluationResult` (`ess/domains/evaluation.yaml:484-573`) moves to
Insufficient only on `state_space_state == Truncated` or when the evaluator sends `refused`. A
`Complete` state space with `evaluable: false` states reaches `computed` if the evaluator omits
`refused`. This breaks invariant 3. The brief lists "a state not evaluable" explicitly
(`docs/brief.md:80-81`).
Fix: `CompleteStateSpace` records an unevaluable count (or closes into a third state, e.g.
`Incomplete`). The evaluation copies that value as it copies `state_space_state` (GAPS G3 pattern),
and `computed` is refused when it is > 0.

### Major

**R4 (major): `ratchet` is an external system modelled as a domain, and it makes a cycle.**
`ess/domains/promotion.yaml:366-379, 459-472` guard on `uxsearch.ratchet.GateDecision`;
`ess/domains/ratchet.yaml:185, 214, 577-585, 621-639` reference and guard on
`uxsearch.promotion.Promotion`. That is a cycle between the two domains. `RequirementBinding.comparison`
and `gate_stage` (`ratchet.yaml:119-122`) restate ratchet's own requirement and gate policy, so ratchet
and the spec both own that fact. Ratchet's design names the engine an assessor
(`docs/ratchet.md:19`) and keeps decisions in ratchet's records (`ratchet/docs/design.md:318`).
Fix: keep only the engine-owned export (`EvidenceRecord`, `ReferenceProposal`). Model gate and
reference decisions as imported facts at an external boundary (provided entities, or a component
for ratchet), and drop `comparison`.

**R5 (major): a hard rule has three homes and two deciders.**
- `rules.DesignRule` with `strength: hard`: the engine prunes on its own verdict, and its limit sits
  in the unspecified `RuleCheck` (`ess/domains/rules.yaml:20-26, 54-93, 271-357`).
- `metrics.MetricRole.hard_rule` (`ess/domains/metrics.yaml:37-41`).
- A ratchet requirement with `absolute_limit` (`ess/domains/ratchet.yaml:36-40`).

Nothing links a `DesignRule` to a `MetricKey`. The engine prunes on a limit it keeps itself, and
ratchet gates on a limit it keeps. The two can disagree. The brief says hard rules are ratchet
requirements (`docs/brief.md:78-79`).
Fix: a hard rule is one record, a metric key plus the limit read from the ratchet preset. Pruning and
the gate read that same limit.

**R6 (major): rule verdicts are not scoped to target, rule state, environment or layout model.**
`RecordRuleVerdict` (`ess/domains/rules.yaml:271-314`) does not check that the rule is Active or that
its target is the variant's target. A hard violated verdict of any rule, even a Retired one, prunes
the variant everywhere: `journeys.yaml:373-386`, `evaluation.yaml:441-454`, `search.yaml:649-662`,
`promotion.yaml:273-286`. The verdict has no environment and no layout-model version (`rules.yaml:95-131`),
so a dynamic hard rule checked under `layout-model/0` prunes forever. `Finding` (`:45-51`) has no
environment, which contradicts its own comment at `:43` and finding identity (`docs/metrics.md:28`).
Fix: guard the rule's state and target; add environment, layout-model version and stage to the
verdict and to `Finding`.

**R7 (major): a variant with no hard-rule verdicts passes the hard-rule guard.**
`ess/domains/promotion.yaml:273-286, 351-364, 444-457` refuse only when a non-pass verdict exists. A
variant that no hard rule was ever checked against passes. GAPS G4 (`ess/GAPS.md:46-52`) states this
and leaves it to ratchet, but no gate checks that every hard rule is bound (see R8).
Fix: keep a per-variant count of hard rules passed against the target's active hard-rule count, or
require the promote decision to carry a verdict for every bound hard requirement (ratchet coverage
counts, `ratchet/docs/design.md:101-105`).

**R8 (major): a gate decision is tied to neither the variant nor the evidence, and any past pass counts.**
`GateDecision` (`ess/domains/ratchet.yaml:163-190`) holds `promotion_id` only. `evidence_ids` are not
checked to belong to the promotion's variant. `verdict: pass` is stored next to failing
`requirement_verdicts`. The accept guards (`ess/domains/promotion.yaml:366-379`) ask whether any pass
exists, so a later `fail` does not revoke an earlier `pass`.
Fix: the decision carries the variant digest. `RecordGateDecision` updates the latest decision on the
Promotion row, and acceptance reads that field.

**R9 (major): the evidence lacks what ratchet needs, and its fields are free input.**
`ResolvedTarget` and `EvidenceRecord` (`ess/domains/ratchet.yaml:57-80, 136-158`) have no scope, no
measurement kind (exact or sampled), no stage, no coverage or bound counts, no base-document revision
or ESS version, and no unit or aggregation. `docs/ratchet.md:63-74` and `ratchet/docs/design.md:318`
require them. Every resolved field except `variant_digest` is Optional. `RecordMetricEvidence`
(`:428-488`) takes `value` and `resolved_target` as input without checking them against the
evaluation: its variant, layout model, environment, the key itself, or the metric's value in
`values`.
Fix: add the missing fields as required, copy them from the evaluation row (`related:`) rather than
the input, and refuse a resolved target that differs from the evaluation.

**R10 (major): the trade-off shape depends on a ratchet concept ratchet declined.**
The spec uses `trade_off_group` and `worse`, both supplied by the proposer
(`ess/domains/promotion.yaml:61-68, 243-246`), and the `trade_off_only` flag
(`ess/domains/ratchet.yaml:572-575`). Ratchet declines a standing requirement group
(`ratchet/docs/design.md:386`). It defines a trade-off as one reference change that lists losses and
gains and that the CLI checks (`:298`). `ReferenceProposal` (`ratchet.yaml:194-226`) carries no
losses, gains, frontier or authorization (`docs/ratchet.md:124-148`).
Fix: `AcceptTradeOff` produces one trade-off reference change for ratchet, with losses, gains,
evidence and `authorized_by`. `worse` comes from ratchet's result, not from the proposer.

**R11 (major): a campaign does not pin the policy it is judged by.**
`Campaign` stores ids only (`ess/domains/search.yaml:59-104`).
- `ReweighKpi` says "running ones keep theirs" (`ess/domains/metrics.yaml:411`), but nothing keeps
  them.
- A new rule "applies to every variant evaluated from now on" (`ess/domains/rules.yaml:245`).
- The catalogue, the layout-model version and the journey bounds are not pinned.

`docs/ratchet.md:108-110` says a campaign cannot change its rules, weights, KPI set or bounds.
`StartCampaign` also accepts Retired journeys and KPIs (`search.yaml:421-470`).
Fix: record a policy digest at start (KPI terms and weights, active rules, bounds, catalogue,
layout model), and refuse retired members.

**R12 (major): caller-supplied keys are not checked against their fields.**
- `RequestEvaluation` (`ess/domains/evaluation.yaml:400-480`) pairs any `variant_digest` with any
  state space; the state space's own `variant_digest` is never compared.
- `cached` answers a key whose stored fields differ from the input.
- `EnumerateStateSpace` (`ess/domains/journeys.yaml:338-407`) takes `bounds` that need not equal
  `Journey.bounds`.
- `NotDeterministic` fires only for Computed rows; a differing result on an Insufficient row is
  silently `already-recorded` (`evaluation.yaml:508-525`).

GAPS G6 covers the digest computation, not these field checks.
Fix: a `when_related` predicate for `variant_digest == input.variant_digest` on the state space, a
key-collision refusal when the fields differ, and journey bounds read from the Journey.

**R13 (major): `MaterializeVariant` drops the first route.**
`ess/domains/design.yaml:996-1010` creates the variant with `routes: []`, though its summary says
"reached by its first route". The route that made the variant is lost, and the origin
(`base`/`search`/`manual`) is lost with it. Nothing checks that `route.design_space_id` is Frozen and
belongs to `target_id`.
Fix: `routes: [input.route]`, and guard the route's design space.

**R14 (major): evaluation stages are missing, and stage 3 cannot remove a frontier entry.**
No entity carries a stage (static, journey paths, full); `stage` does not occur in `ess/domains/`. So
the merge gate (journey-path states) and the promote gate (full state space,
`docs/ratchet.md:97-101`) read evidence that cannot be told apart. `docs/state-space.md:366-367` says
a variant that fails a hard rule in stage 3 leaves the frontier, but `FrontierEntry` only goes
OnFrontier → Dominated with a dominating variant (`ess/domains/search.yaml:195-204, 748-795`).
Fix: add `stage` to StateSpace, Evaluation and Evidence, and a `withdraw` transition for an entry
that fails later.

**R15 (major): D3 depends on actor names alone.**
`Owner` (`ess/domains/promotion.yaml:143-150`) and `ReviewAgent` (`:163-168`) are both non-system
actors, so the model cannot tell a person from an agent. `accepted_by` is free text
(`:408-414`; the studio has the person type it, `ess/ui/studio.ui.yaml:350, 357`), and `author_kind`
is input (`promotion.yaml:527-530`). Validate completeness reports that no scenario reaches the
accepting branch of `AcceptTradeOff` or `AcceptPromotion`. Their `grant/denied` scenarios are
therefore not synthesized: no check refuses an `AcceptancePolicy` or `ReviewAgent` caller.
Fix: take `accepted_by` and `author` from the session actor, mark agent actors apart from people,
and add a seed that reaches `accepted` so the grant checks are generated.

**R16 (major): the promotion lifecycle ends at Accepted.**
`ess/domains/promotion.yaml:78-91`:
- Nothing supersedes a proposal whose variant was dominated or whose base document changed.
- No link leads to the next base-document revision; `BaseDocument` has one state and no
  predecessor (`ess/domains/design.yaml:173-200`).
- "Its promoted variant, once one is accepted" (`design.yaml:305-309`) has no link.
- The re-evaluation that propagation causes (`docs/concept.md:86-91`) is not modelled.

Fix: give `BaseDocument` a `supersedes`, and give Promotion an `Applied` and a `Superseded` state.

**R17 (major): "fixed vs axes" is declared but not wired.**
`ComponentBinding` (`ess/domains/design.yaml:310-333`) names `top_k` but no source campaign. Nothing
turns an inner target's top-k frontier entries into an Axis and its AxisValues
(`docs/concept.md:93-96`).
Fix: the binding names the inner campaign, and `FreezeDesignSpace` derives the axis values from its
frontier, or marks the step UNMAPPED.

**R18 (major): the studio does not show the climb up the hill.**
`ess/ui/studio.ui.yaml:155-162` charts `evaluated, pruned, insufficient, frontier_size` per step. The
best KPI values ("best KPI values per generation", `docs/concept.md:54`) appear only as a raw column
(`:167`).
Fix: chart `best` per KPI over `seq`. This needs a view row per (climb point, KPI).

**R19 (major): the Owner accepts a trade-off without seeing what gets worse.**
`PromotionById` (`ess/domains/promotion.yaml:664-686`) publishes neither `trade_off_group` nor
`worse`. `promotions.detail` (`ess/ui/studio.ui.yaml:333-336`) shows neither, and offers Accept and
Accept trade-off together whatever the promotion is (`:328-331`). D3 asks a person to accept the named
trade-off, but the page never shows it.
Fix: publish the losses and gains (from R10), and show only the accept action that applies.

**R20 (major): the studio cannot set up the campaign for a picked target.**
- The row action `start` (`ess/ui/studio.ui.yaml:87`) does not bind the row's `target_id`.
- `design_space_id` lists every design space (`:97`).
- `journeys`, `environments` and `kpis` are free-text id tags (`:98-100`), because the views
  `Journeys`, `Environments` and `Kpis` are not used.
- No studio page covers declaring a target, design space or axes, or freezing a space
  (`DeclareTarget`, `DeclareDesignSpace`, `AddAxis`, `AddAxisValue`, `FreezeDesignSpace`).

Fix: bind the target, filter the choices by target, and add choice views for the members. Either add
setup pages or state that setup happens outside the studio.

**R21 (major): compare shows lists, not the variants.**
`variants.compare` (`ess/ui/studio.ui.yaml:258-300`) shows verdict and evaluation tables side by
side. Concept § 2.5 asks for variants "side by side in the same state and environment"
(`docs/concept.md:54-55`), and taste judgement needs to see them (`:105`). No state or environment can
be picked, and no view exposes layout output: `LayoutOutput` is unused (`ess/domains/evaluation.yaml:117-129`).
Fix: a view keyed by (variant, UI state, environment) with the rendered or laid-out result, or mark
the comparison UNMAPPED.

**R22 (major): the studio fixtures contradict the model and the invariants, and no check notices.**
- `ess/ui/fixtures/studio.yaml:46` puts a variant with `completion.miss = 1` on the frontier, which
  is a hard rule `H = 0` (`docs/metrics.md:65`).
- `:58, :65` promote it as a trade-off with `trade_off_only: true`. That is exactly "hard rules never
  trade" (`docs/ratchet.md:142-143`).
- Type drift:
  - `best` is a digest string (`:40-42`), but the model has `List<KpiReading>` (`ess/domains/search.yaml:151-152`).
  - `kpi_values` is keyed by `metric_key` instead of `kpi_id` (`:45-46`; `search.yaml:48-54`).
  - A finding has `note` instead of `message` (`:50`; `ess/domains/rules.yaml:45-51`).
  - `budget` lacks `max_wall_time_s` (`:36-37`).

`ess ui check --model` and `ess ui test --model` both pass, so fixtures are not checked against the
model.
Fix: correct the fixtures, and raise fixture type-checking against `--model` as an ESS gap.

### Minor

**R23 (minor): not-found refusals are missing.**
- `ReweighKpi` (`ess/domains/metrics.yaml:381-418`; named by validate).
- `AddAxisValue`, with no axis guard (`ess/domains/design.yaml:911-947`).
- `DeclareTarget`, with no base-document guard (`:636-686`).
- `RequestAxisSwitch`, with no variant guard (`:1024-1054`).
- The bound requirement in `RecordMetricEvidence` (UNMAPPED, `ess/domains/ratchet.yaml:489-491`).

Fix: one `when_related … exists: false` per command.

**R24 (minor): an axis switch has no failure end.**
`AxisSwitch` (`ess/domains/design.yaml:487-494`) is Requested → Resolved only. When materialization
fails after the request (`operator-defect`, `:990-994`), the switch stays Requested forever.
Fix: a `Failed` terminal state.

**R25 (minor): state spaces and evaluations cannot be abandoned.**
A crashed enumeration leaves the state space `Enumerating` for good (`ess/domains/journeys.yaml:128-141`),
and every evaluation over it is refused (`ess/domains/evaluation.yaml:430-438`). `cached` also answers
for a key that is still Enumerating (`journeys.yaml:404-407`).
Fix: an `Abandoned` state that counts as insufficient.

**R26 (minor): `MarkDominated` checks too little.**
It has no guard that the campaign is Running, which contradicts "the frontier is final"
(`ess/domains/search.yaml:539`). The input `campaign_id` and `variant_digest` are not compared with
the entry row, and an entry can be dominated by its own variant (`:748-795`).
Fix: guard each of these.

**R27 (minor): bounds and counts are not enforced during enumeration.**
`RecordUiState` accepts a depth or virtual time beyond the bounds and a duplicate `state_digest`.
`CompleteStateSpace` and `TruncateStateSpace` take `states_count` and `bound_hit` unchecked
(`ess/domains/journeys.yaml:409-537`).
Fix: invariants or guards against the stored bounds, and uniqueness of `state_digest` per key.

**R28 (minor): metric versions are ambiguous.**
The identity is `metric_key` without a version (`ess/domains/metrics.yaml:91-123`), yet a new
function "is a new version under a new key" (`:190-194`), and evidence carries an optional
`metric_version` (`ess/domains/ratchet.yaml:75-76`).
Fix: key the metric as `key@version`, as ratchet does (`docs/metrics.md:9-10`).

**R29 (minor): a KPI can have no direction or use a diagnostic metric.**
A KPI may have `direction: none` and use a diagnostic metric, and term scopes are not checked
against the target's scope (`ess/domains/metrics.yaml:32-35, 320-377`). A KPI with no direction has
no Pareto order.
Fix: refuse `none` for KPIs, and refuse metrics whose role is `diagnostic`.

**R30 (minor): judgement records lack state, environment and producer.**
`docs/concept.md:110-121` describes a judgement as "B over A in state s, environment e" and counts
one model family as one source. `JudgementRecord` (`ess/domains/promotion.yaml:93-131`) has neither,
and `preferred` is not constrained to left or right. A disposition does not name the rule or KPI it
became. Judgements also live in `promotion`, though they feed `rules` and `metrics`.
Fix: add the state, the environment and the producer family; constrain `preferred`; link the
outcome of a disposition.

**R31 (minor, simplicity): `Variant.Pruned` and `PruneVariant` are never read.**
Every guard reads `RuleVerdict` instead (see R6). `PruneVariant` writes another domain's entity
(`ess/domains/rules.yaml:344-352`), and the comment at `ess/domains/design.yaml:438` names the wrong
command.
Fix: drop the state and the command, or make the guards read the state.

**R32 (minor, simplicity): the layout-contract types are unused.**
`LayoutInput`, `LayoutOutput`, `NodeBox` and `AccessibleNode` (`ess/domains/evaluation.yaml:69-129`)
are referenced by nothing; validate notes that the `NodeBox` invariants never run. `NodeBox` is in
px, while v0 works in cells (`:62-67`).
Fix: move them to the docs, or use them in a view (R21).

**R33 (minor): journey declarations and the Designer actor are duplicated.**
`UxDeclarationsDocument.journeys` (`ess/domains/design.yaml:61-72`) names journeys by `test_file` and
`test_name`, as `Journey` does (`ess/domains/journeys.yaml:60-77`): two homes for a journey's
required nodes and completion signal. Two actors are both displayed as "Designer"
(`design.yaml:497`, `promotion.yaml:134`).
Fix: put `required_nodes` and `completion_signal` on `Journey`, and use one Designer actor.

**R34 (minor, simplicity): there are three weights, and the soft-rule weight reaches nothing.**
`DesignRule.weight` (`ess/domains/rules.yaml:75-79`) feeds no KPI. "A soft rule's score enters the
variant's KPIs" (`:314`) has no path. Beside it are `KpiTerm.weight` and `Kpi.weight`
(`ess/domains/metrics.yaml:67-75, 152-155`).
Fix: make a soft rule a metric that a KPI term names, and drop the rule weight.

**R35 (minor): `ContainerMismatch` is checked in one direction only.**
A wider target with a container width is accepted (`ess/domains/evaluation.yaml:292-296, 365-374`).
Fix: add the converse guard.

**R36 (minor): the merge gate has no subject.**
`GateStage.merge` exists (`ess/domains/ratchet.yaml:42-44`), but a decision is keyed by
`promotion_id` (`:170-171`). The merge gate judges a merged revision (`docs/ratchet.md:97-99`), not a
promotion.
Fix: drop `merge` from the engine, or key that decision by revision.

**R37 (minor): the UI tests cover 3 of the person's jobs.**
`ess/ui/tests/studio.yaml:4-24` tests stop, the live climb and axis switch. Frontier, compare, judge,
propose, accept, trade-off and disposition have no test.

## By question

**1. Domain boundaries.**
- `design`: holds 9 entities; still one responsibility (what can vary).
- `journeys`: one responsibility, but it reads `rules` for pruning.
- `rules` and `metrics`: overlap on hard rules (R5).
- `evaluation`: one responsibility.
- `search`: one responsibility.
- `promotion`: holds judgements, which belong beside rules and KPIs (R30).
- `ratchet`: an external boundary, in a cycle with `promotion` (R4).

Cross-domain writes: `rules.PruneVariant` moves `design.Variant` (R31). A merge worth considering: the
judgement record into a small `review` domain, or into `rules`.

**2. Model vs concept.** Every glossary term of `docs/brief.md:39-74` has a home, with these
exceptions:
- "design rule" has three homes (R5).
- "journey" has two (R33).
- "finding" means a rule finding (`rules.Finding`) and a judgement disposition (`Disposition.finding`)
  without saying which.

Target scopes (`TargetScope`), fixed vs axes (`ComponentBinding`, not wired, R17), judgement records
and the climb (`ClimbPoint`) are present. Missing from the spec:
- evaluation stages (R14);
- audiences and conditions (`docs/ratchet.md:28-47`), acceptable as ratchet-side;
- the geometric equivalence ≡geo, acceptable as an algorithm;
- promotion propagation (R16).

Spec entities the docs do not motivate: `Target.host_slot` and `RouteOrigin`, both minor and harmless.

**3. Invariants.**

| invariant | stated | enforced by | status |
|---|---|---|---|
| pure evaluation | `evaluation.yaml:1-6, 69-72` | identity by key, `NotDeterministic` (`:508-518`) | by construction; key fields unchecked (R12); G6 |
| hard rules prune | `rules.yaml:3-5` | verdict guards in 4 domains | holes: target, rule state, environment (R6); no-verdict passes (R7) |
| insufficient evidence never passes | `evaluation.yaml:54-60`, `ratchet.yaml:46-50` | `search.yaml:680-692`, `promotion.yaml:366-379`, `ratchet.yaml:452-470` | broken: R1, R2, R3 |
| promotion needs hard rules and the gate | `promotion.yaml:308-309` | `:273-286, :351-379` | partial: R7, R8 |
| judgements never change a gate | `promotion.yaml:5-6`, `ratchet.yaml:160-162` | none; no command reads `JudgementRecord` (checked by grep) | by construction only (GAPS G10) |
| two routes are one variant | `design.yaml:409-410, 974-975` | identity by digest | by construction; the canonical form is UNMAPPED (G7); first route lost (R13) |
| D3, only a person accepts a trade-off | `promotion.yaml:61-64, 400-403` | `may` lists, `trade-off` refusal (`:337-349`) | actor naming only; no grant scenario (R15); acceptance on insufficient evidence (R2) |

**4. Lifecycles.**
- Missing terminal or failure states: AxisSwitch (R24), StateSpace (R25), FrontierEntry withdraw
  (R14), Promotion applied or superseded (R16), BaseDocument revision (R16).
- Missing refusals: R23 and R26.
- Every command emits or preserves, and every event has a command.
- `RequirementBinding` cannot be bound again after Unbind: `same` preserves the Unbound row
  (`ratchet.yaml:397-400`).

**5. Ratchet boundary.** The engine stores no numeric baseline or reference. It does keep policy that
ratchet should own:
- the hard-rule limits inside `RuleCheck` (R5);
- `comparison` and `gate_stage` (R4);
- the regression set `worse`, supplied by the proposer (R10).

Evidence lacks scope, kind, stage, coverage, base revision and ESS version, and is not tied to its
evaluation (R9).

**6. Studio.** Covered: picking a target (partly, R20), start and stop, the live climb (counts only,
R18), the frontier, axis switches (with every axis value listed, GAPS), compare (lists only, R21),
judging, proposing, accepting and accepting a trade-off (blind, R19). Of the 59 uncovered views and
commands, 19 belong to system actors and are correctly absent. The person-facing ones that matter:
- `Journeys`, `Environments`, `Kpis` and `Axes` (R20);
- `DeclareTarget`, `DeclareDesignSpace`, `AddAxis`, `AddAxisValue`, `FreezeDesignSpace` (setup);
- `DeclareRule` and `ReweighKpi` (what a disposition leads to);
- `Evidence` and `ReferenceProposals` (what the Owner accepts).

The studio covers 9 of the 29 commands a person may send.

**7. Simplicity.** These can be removed without losing a stated outcome:
- `Variant.Pruned` and `PruneVariant` (R31);
- the unused layout types (R32);
- the soft-rule weight (R34);
- `RequirementBinding.comparison` (R4);
- `GateStage.merge` (R36);
- `UxDeclarationsDocument.journeys`, by merging it into `Journey` (R33).

## Hardening targets

These are the places where a planted defect would most likely survive the current specification
and any suite synthesized from it:

1. `ess/domains/search.yaml:680-692`: frontier admission with `evaluation_keys: []` or a subset of
   journeys × environments (R1).
2. `ess/domains/promotion.yaml:466` with `ess/domains/ratchet.yaml:587-605`: a gate decision
   `insufficient_evidence` with `trade_off_only: true` accepts a trade-off (R2).
3. `ess/domains/evaluation.yaml:561-573`: `computed` over a Complete state space that holds
   `evaluable: false` states (R3).
4. `ess/domains/evaluation.yaml:456-480`: an evaluation of variant A over the state space of
   variant B, or `cached` on a colliding key (R12).
5. `ess/domains/rules.yaml:297-314`: a verdict from a Retired rule or another target's rule prunes
   the variant (R6).
6. `ess/domains/ratchet.yaml:472-488`: metric evidence whose `value` or `resolved_target` differs
   from the evaluation it names (R9).
7. `ess/domains/promotion.yaml:366-379`: acceptance on an earlier `pass` after a later `fail`, or on
   a `pass` whose `requirement_verdicts` hold a failure (R8).
8. `ess/domains/promotion.yaml:143-161, 404-414`: `AcceptTradeOff` granted to `AcceptancePolicy` or
   `ReviewAgent`; no `grant/denied` scenario is synthesized (R15).
9. `ess/domains/journeys.yaml:478-537`: `CompleteStateSpace` with a `states_count` above
   `max_states`, or below the recorded rows (R27).
10. `ess/domains/design.yaml:996-1010`: the route of a new variant, already lost; any planted change
    to `routes` on the `new` branch is invisible today (R13).
