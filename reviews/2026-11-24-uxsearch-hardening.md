# Hardening: ESS specification `uxsearch` v1

Date: 2026-11-24. Input: the review `reviews/2026-11-24-uxsearch-spec-review.md` (3 blockers, 19
major, 15 minor) and its 10 hardening targets. ESS 0.98.0, format `ess/37`.

## Result

- Blockers R1, R2 and R3 are fixed with guards. Each guard has a scenario, and a defect planted in
  it was caught.
- 16 of the 18 majors R5–R22 are fixed. R17 and R21 are marked `UNMAPPED`. R4 is a proposal
  (below); the domains were not restructured.
- All 10 hardening targets have scenarios in `ess/scenarios/`. All 24 planted defects were caught.
- The gate passes in about 2.4 s.
- Synthesis went from 2 s to 1000 s. This is open, see "Synthesis time".

## Blockers

| | Guard | Scenario |
|---|---|---|
| R1 | `ess/domains/search.yaml:989` `no-evidence`: empty or repeated `evaluation_keys`<br>`:1000` `kpi-values-repeat`<br>`:1039` `kpi-values-mismatch`: `kpis.count != input.kpi_values.count` | `frontier-needs-evidence.yaml` |
| R2 | `ess/domains/promotion.yaml:442` `trade-off-inconsistent`: trade-off only beside a non-`fail` verdict or with no loss<br>`:455` `pass-inconsistent`: a `pass` beside a loss or a failing requirement<br>`:778` `AcceptTradeOff` `gate-not-passed` (only a trade-off-only `fail`)<br>`:794` `no-gains` | `trade-off-needs-a-clean-fail.yaml` |
| R3 | `ess/domains/journeys.yaml:659` `CompleteStateSpace` `unevaluable`: closes `Incomplete` when a state is not evaluable<br>`ess/domains/evaluation.yaml:602` `state-not-evaluable`: a result over an `Incomplete` space is `Insufficient` | `unevaluable-state-is-insufficient.yaml` |

`hard-rule-unchecked` (`promotion.yaml:682`, `:839`) refuses an accept while an Active hard rule
of the target has no Recorded `pass` for the variant. This closes R7 and GAPS G4.

## Majors

| | Status | Change |
|---|---|---|
| R4 | proposal | below |
| R5 | fixed | A hard rule names its `metric_key` and must name its `requirement_id`; a soft rule must not (`rules.yaml` `hard-unbound`, `soft-bound`). The limit lives in the ratchet requirement only. |
| R6 | fixed | `RecordRuleVerdict` refuses a non-Active rule (`rule-retired`) and a rule of another target. The verdict carries environment, layout-model version and stage. `RetireRule` voids the rule's Recorded verdicts. |
| R7 | fixed | `hard-rule-unchecked` |
| R8 | fixed | `GateDecision` entity dropped. `RecordGateDecision` (now in `promotion`, granted to `ratchet.Ratchet`) overwrites the gate fields on the Promotion and checks the variant and the evidence's variant. Accept reads that field. |
| R9 | fixed | `EvidenceRecord` copies metric version, unit, scope and kind, and the evaluation's variant, stage, environment, layout model, state count, bound and digests, all through `related:`. `value-mismatch` refuses a value the evaluation did not compute. |
| R10 | fixed | Losses and gains (`RequirementChange`) come from ratchet's gate decision, not the proposer. `ReferenceProposal` carries trade-off, losses, gains, evidence, `authorized_by` and `decided_by`. |
| R11 | fixed | `PinPolicy` records `policy_digest`, layout-model version and catalogue digest. `StartCampaign` refuses retired journeys and KPIs and empty member lists. |
| R12 | fixed | `state-space-of-another-variant`, `key-collision` (evaluation and state space). Bounds are read from the Journey. `not-deterministic` covers Insufficient rows. |
| R13 | fixed | `first_route` and `routes`; the design space must be Frozen and of the target. |
| R14 | fixed | `stage` on StateSpace, Evaluation and Evidence. `WithdrawFromFrontier` needs a Recorded hard failure. |
| R15 | fixed | `accepted_by` and the judgement author come from the caller (`{caller: principal}`). `AcceptancePolicy` and `ReviewAgent` are not granted `AcceptTradeOff`, and a scenario pins that. |
| R16 | fixed | Promotion `Applied` and `Superseded`. `ApplyPromotion` needs a successor base document. |
| R17 | `UNMAPPED` | Deriving axis values from an inner campaign's frontier is stated, not modelled (`design.yaml:300`). |
| R18 | fixed | `ClimbReading` per (campaign, step, KPI). The climb page charts `best` per KPI. |
| R19 | fixed | `PromotionById` publishes the gate, losses and gains. The page offers only the accept that applies. |
| R20 | fixed | `campaigns.start(target_id)` with choices filtered by target. Setup happens outside the studio (header comment). |
| R21 | `UNMAPPED` | No view holds a laid-out result per state and environment (studio warning). |
| R22 | fixed | Fixtures corrected: no hard failure on the frontier, `kpi_id`-keyed values, `message`, `max_wall_time_s`. Fixture typing is GAPS G15. |

Minors done: R23, R24, R25, R26, R27, R29 (`none` direction refused; the diagnostic metric is an
external check), R31–R37. Not done: R28 (metric key with version: changes every key in the
model and in ratchet; owner decision), R30 (judgement state, environment and producer: not on a
target's path).

Coordinator fixes:
- `as: json` became `as: map` / `as: list`.
- The climb points have `sort: {by: seq, dir: asc}`.
- The climb channel has `scope: param` and `session: {per: campaign_id}` again, and the
  `only_if: matches(params)` guards are gone. `fixtures/scripts/climb.yaml` names
  `session: {campaign_id: cp-01}`.
- The studio test "live climb" passes with `session:` and fails without it. There is no second
  cause. The GAPS note is removed.

## R4 proposal: ratchet as an external system

Today `promotion` reads `ratchet.EvidenceRecord` (`promotion.yaml:119, 492`), and
`ratchet.ReferenceProposal` reads `promotion.Promotion` (`ratchet.yaml:182`). That is still a cycle.

If ratchet became an external system:

1. `ratchet` becomes an export domain the engine owns: `EvidenceRecord` and `ReferenceProposal`.
   It has no decision of ratchet's. `RequirementBinding` stays: it is the engine's map from a
   requirement id to a metric, and the limit stays in ratchet.
2. Ratchet becomes a second component (or system) that is `reached_by: network`. Its gate decision
   arrives as `promotion.RecordGateDecision` from an external caller. That is already the shape: the
   command lives in `promotion` and is granted to the `Ratchet` actor.
3. The cycle breaks when `ReferenceProposal` stops referencing `Promotion`. It would be emitted as
   an event (`ReferenceProposed` with losses, gains, evidence and `authorized_by`) that ratchet
   consumes, and not stored as a row that ratchet reads back.
4. Costs:
   - the `already-proposed` and `not-accepted` guards move to the promotion side (`Promotion.proposed_at`);
   - the evidence check in `RecordGateDecision` (`evidence-of-another-variant`) stays, because it
     reads the engine's own export.
5. Nothing in the studio changes.

## Hardening targets

Each defect was planted in a copy of `ess/`. Each copy was then validated, and the scenario was
authored and run on the interpreted target.

| # | Scenario | Planted defect | Result |
|---|---|---|---|
| 1 | `frontier-needs-evidence` | dropped `evaluation_keys.count == 0` | caught (ESS-AUTHOR-041) |
| 1 | | dropped the `distinct` check | caught (ESS-AUTHOR-041) |
| 1 | | `kpis.count != …` → `<` | caught (run) |
| 2 | `trade-off-needs-a-clean-fail` | dropped `verdict != fail` | caught (ESS-AUTHOR-041) |
| 2 | | `gate-not-passed` → pending/insufficient only | caught (run) |
| 2 | | dropped the failing-requirement check of `pass` | caught (ESS-AUTHOR-041) |
| 3 | `unevaluable-state-is-insufficient` | `Incomplete` → `Truncated` | caught (run) |
| 3 | | `evaluable == false` → `depth > 100` | caught (run) |
| 4 | `evaluation-key-matches-its-inputs` | variant predicate replaced | caught (run) |
| 4 | | dropped the catalogue disjunct of `key-collision` | caught (run) |
| 5 | `verdicts-of-live-rules-only` | `rule-retired` predicate → `strength == hard` | caught (run) |
| 5 | | `RetireRule` voids only Void verdicts | caught (run) |
| 5 | | dropped `state == Recorded` in `EnumerateStateSpace` `variant-pruned` | caught (run) |
| 6 | `evidence-copies-its-evaluation` | dropped `v.value == input.value` | caught (run) |
| 6 | | dropped `layout_model_version` copy | caught (run) |
| 6 | | dropped `states_count` copy | caught (run) |
| 7 | `latest-gate-decides` | `recorded-fail` does not set `gate: fail` | caught (run) |
| 7 | | `AcceptPromotion` `gate-not-passed` → `gate == pending` | caught (run) |
| 8 | `only-the-owner-accepts-a-trade-off` | `AcceptTradeOff` granted to `AcceptancePolicy` | caught (ESS-AUTHOR-038) |
| 8 | | dropped `accepted_by: {caller: principal}` | caught (run) |
| 9 | `complete-count-matches-its-states` | `max_states <` → `<=` | caught (run) |
| 9 | | `seq >=` → `>` | caught (run) |
| 10 | `variant-keeps-every-route` | `known` sets `routes: []` | caught (run) |
| 10 | | `known` overwrites `first_route` | caught (run) |

Caught 24 of 24, survived 0. The 10 scenarios are listed in `ess/ess-inputs.yaml` and run in
about 1 s.

Techniques run: design review (the input review) and a hand-planted mutation audit.

Not run:
- `ess verify conform mutate`, random sequences, guard analysis, determinism and metamorphic
  relations. Each needs synthesis, which takes 1000 s (below).
- Spec diff: there is no released revision to diff against.

## Completeness

| | Before | After |
|---|---|---|
| Unscenarioed constructs | 156 | 258 |
| Outcomes without a synthesized scenario | 95 of 161 | 160 of 264 |
| Unanswered questions | 51 | 45 |
| Commands with no wrong-actor check (`grant/denied` not synthesized) | 17 | 31 (AcceptTradeOff pinned by hand) |
| Authored scenarios | 0 | 10 (27 outcomes and 1 refusal pinned) |

The hardening added 103 outcomes. Synthesis completeness does not count authored scenarios.
The system-issued refusals (`ESS-SYNTH-023`, 34 outcomes) are gone. Outcomes that synthesis
could not arrange (`ESS-SYNTH-003`, 8 → 55) and could not reach (`ESS-SYNTH-004`, 32 → 62)
rose with the new row-set guards.

## Synthesis time

`ess verify conform synthesize`, and with it `ess specify validate --format json`, took 2 s
before. On the hardened specification `synthesize` did not finish within 580 s (`timeout` exit
124), and `validate --format json` took 1000 s.

What is known:
- perf puts the time in the arrangement search.
- No single domain causes it, and neither do the non-system actors.
- Removing the row-set guards of every refusal branch brings it to 59 s.

The repository gate does not synthesize. `AGENTS.md` asks for a generator jump to be fixed in the
same change. The fix belongs to ESS's synthesis (GAPS G14): removing the row-set guards would
remove the blockers' fixes.

## Gate

| Command | Exit | Time | Output |
|---|---|---|---|
| `ess specify validate --path ess` | 0 | 0.2 s | 10 files, 10 scenarios, valid |
| `ess compile --path ess` | 0 | 0.2 s | 222 declarations |
| `ess ui check --path ess/ui/studio.ui.yaml --model ess` | 0 | 0.4 s | 0 errors, 3 warnings (UNMAPPED) |
| `ess ui test --path ess/ui/studio.ui.yaml --model ess ess/ui/tests/studio.yaml` | 0 | 0.5 s | 5 passed |
| `ess verify conform author` + `run --target interpreted` over `ess/scenarios` | 0 | 1.1 s | 10 passed |

## Findings

```findings
- file: ess/GAPS.md
  category: ess-limit
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: conformance synthesis went from 2 s to 1000 s with the hardening's row-set guards, so completeness takes 17 minutes and the mutation audit does not fit the 10-minute job limit (GAPS G14)
- file: ess/domains/journeys.yaml
  line: 615
  category: spec-gap
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: CompleteStateSpace refuses a count below the recorded states but accepts a count above them, because ESS cannot count related rows
- file: ess/domains/search.yaml
  line: 1122
  category: ess-limit
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: frontier coverage of every journey and environment needs a product of list counts, which no expression reads, so it stays an external branch (GAPS G12)
- file: ess/ui/fixtures/studio.yaml
  category: ess-limit
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: ess ui check and ess ui test with --model do not type fixture rows against the view types (GAPS G15)
- file: ess/ui/fixtures/scripts/climb.yaml
  line: 2
  category: ess-limit
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the terminal renderer of ess ui test drops the events of a script without a top-level session, silently; the script now names one
- file: ess/domains/promotion.yaml
  line: 204
  category: ess-limit
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a system actor cannot carry the attribute a caller-recorded field needs, so AcceptancePolicy and the engine's workers are non-system actors (GAPS G13)
- file: ess/domains/ratchet.yaml
  line: 182
  category: spec-gap
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: R4 not done; ReferenceProposal references Promotion and promotion reads EvidenceRecord, so the promotion-ratchet cycle remains
- file: ess/domains/design.yaml
  line: 300
  category: spec-gap
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: R17 is UNMAPPED; deriving an inner campaign's frontier into axis values is not modelled
- file: ess/ui/studio.ui.yaml
  category: spec-gap
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: R21 is UNMAPPED; no view holds a laid-out result per state and environment, so compare shows tables
- file: ess/domains/metrics.yaml
  line: 91
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: R28 not done; the metric key carries no version
- file: ess/domains/promotion.yaml
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: R30 not done; judgement records lack state, environment and producer family
```
