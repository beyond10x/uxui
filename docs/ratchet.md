# The engine on ratchet

Status: design, 2026-11-23. Ratchet is in its design phase; its format is a proposal
(`ratchet/docs/design.md:56`). Field names below follow its § 3 example (`design.md:58-95`); fields it
does not have are marked **ext** and listed in § 8. The preset family is
`presets/web-ux.ratchet.yaml`. "[Inference]" marks a claim no source states.

The engine keeps no baseline of its own (`docs/brief.md` § Ratchet). Metrics are ratchet
measurements, hard rules and KPIs are ratchet requirements, the promoted variant's values become
protected references.

## 1. Mapping

| engine | ratchet | note |
|---|---|---|
| campaign target (component, widget, section, page, composition, product) | **subject** (`design.md:12`) | `kind: ui-component`, `ui-page`, `user-interaction` (a journey) |
| who a journey and an environment stand for | **audience** | see § 2 |
| metric | **measurement** (`output`, `unit`, `aggregation`) | plus `scope`, `kind`, `direction` (ext) |
| evaluation of a metric over a variant | **assessment plan** with assessor `ux-eval@1` or `ux-static@1` | `design.md:19-20` |
| variant | **target**, `kind: ui-variant` (ext), `selection: candidate` | resolved as in § 4 |
| environment matrix | **observation conditions** | `conditions` (ext); ratchet's word "environment" means a target class (`design.md:52`), so the engine's environments are called conditions here |
| hard rule | **requirement** with an absolute `condition` | `design.md:117-118` "absolute limits" |
| KPI | **requirement** with a protected `reference` (ext) | `design.md:22` |
| merge and promotion | **gates** `merge`, `promote` | `design.md:23` |
| promoted variant's values | **protected references**, adopted by a reference change | `design.md:126,146` |
| "insufficient evidence" (`state-space.md` § 7) | **insufficient evidence** | `design.md:124` |

## 2. Subjects and audiences

One subject per target. A page campaign binds the family once per page; each journey is also a
subject of kind `user-interaction`, so a journey's results are not summed into the page's
(`metrics.md` § 1, result scope). A component's subject is used by every subject whose document
expands it (§ 5).

Audiences tie a requirement to the people it protects [Inference: a proposal for using ratchet's
audience field]:

| audience | conditions it is evaluated under |
|---|---|
| pointer user | `input_mode: pointer`, every viewport |
| keyboard user | `input_mode: keyboard` |
| low-vision user | `text_scale: 2.0`, both theme modes |
| small-screen user | `viewport: 390x844` (or the smallest harness width) |
| screen-reader user | none in the engine: the walker by role and name stands in; real screen readers stay outside (`research/2026-11-23-external-machine-checkable-ux.md:35-37`) |

The page's ESS `actor` (`ess/website/docs/reference/ess-ui.md:558`) names whose grants decide what is
visible; the fixtures sign that actor in (`ess/website/docs/reference/ess-ui-test.md:343-345`).

## 3. Assessments

| assessor | reads | observation | runs in |
|---|---|---|---|
| `ux-static@1` | the document and the campaign declarations | snapshot, no conditions | stage 1, microseconds per variant |
| `ux-eval@1` | document, journeys, fixtures, layout model | snapshot; every condition cell; `stage: journey-paths` or `full` (ext) | stages 2 and 3 |
| `ux-walk@1` | journeys in a browser | controlled run, N samples (`design.md:47`) | after promotion, against a build; measures layout-model fidelity |

An evaluation of a deterministic variant is a **snapshot** ("what holds for this revision",
`design.md:46`), not a controlled run: the same inputs give the same value, so one sample is the
measurement. Only `ux-walk@1` measures wall-clock values and needs a sample count.

## 4. Evidence records

An assessment-run record (`design.md:144`) for `ux-eval@1` holds:

- **candidate**: the variant digest (canonical form, `docs/brief.md`, Variant);
- **resolved target** (`design.md:52`): variant digest, the base document's repository revision, the
  operator sequence and axis values, `layout-model/<n>` and its font-table digest, the fixture
  digest, the ESS version, the metric version;
- **condition cell**: viewport, theme mode, input mode, text scale, reduced motion;
- **bounds**: states evaluated per journey, the bound, and whether it was hit, with the dimension
  counts (`state-space.md` § 7);
- **values** with unit and the aggregation used; for finding-type metrics the list of findings by
  identity (rule or metric id, node path, condition cell);
- **pointers**: the evaluation cache keys, so the studio can show the state behind a value.

Volume: the ticket page has 1.2 x 10^8 (variant, condition, state) evaluations (`concept.md` § 5).
They are not records. Ratchet receives one record per (requirement, subject, condition cell) for
the candidate, with the digest of the evaluation set it aggregates [Inference].

**Insufficient evidence** (`design.md:124`, `ratchet/vision.md:32-33`) is returned when: a journey hits
its state bound; the layout model refuses a node the measurement reads; the terminal path refuses a
step; an ACT rule says `cantTell`; a fixture is missing; or the evidence names another layout-model
version, font table or metric version than the reference (incomparable; `design.md:126` "a new
evidence series"). Only a pass clears a gate.

## 5. Requirements and gates

| requirement | ratchet form | gate |
|---|---|---|
| hard rule, e.g. `completion.miss` = 0 | `condition: {operator: eq, value: 0}` per condition cell | merge, promote |
| hard limit with a value, `a11y.contrast_min` ≥ 4.5 | `condition: {operator: gte, value: 4.5}` | merge, promote |
| KPI, e.g. `effort.actions` | `reference: {kind: protected, comparison: no_regression, tolerance: 0}` per cell (ext) | merge, promote |
| finding-type KPI, `layout.clipping` | `reference: {comparison: no_new_findings}` by identity (`design.md:111`) | merge, promote |
| `a11y.act_cant_tell` | `insufficient_when: {operator: gt, value: 0}` (ext) | merge, promote |
| diagnostic | measurement only, no requirement (observe mode, `design.md:117`) | none |

- **merge** (`stage: pre-merge`): every hard rule in every condition cell, every KPI no worse than
  its protected reference, no new findings, nothing insufficient. Evaluated on the journey-path
  states (stage 2), which keeps it inside the 2-minute test budget (`state-space.md` § 6).
- **promote** (`stage: pre-promotion`): the merge gate's requirements over the **full** state space
  of every journey (stage 3), plus a reference change record (§ 6).
- Tolerance is 0 for exact and modelled kinds: the values carry no noise
  (`research/2026-11-23-external-machine-checkable-ux.md:71`). `ux-walk@1` timing uses a band and
  N samples, kept apart (§ 8, gap 3).
- **Component propagation.** A merge that changes a widget or a component default changes the
  digest of every subject that expands it. The merge gate evaluates all of them; one regression in
  any using subject fails the merge [Inference: ratchet needs subject dependencies, gap 6].
- Policy comes from the trusted revision (`design.md:126`, `vision.md:36`): a campaign cannot change
  the rules, weights, KPI set or bounds it is judged by. A changed KPI weight from judgement records
  (`concept.md` § 4) is a policy change.

## 6. Promotion, adoption and trade-offs

A promotion adopts the chosen variant's values as the new protected references
(`ratchet reference propose`, `design.md:136`). Because the values are exact, one evaluation with
the same layout-model version is the evidence; ratchet's "repeated comparable results"
(`design.md:122`) applies to sampled values only [proposal].

A frontier variant is better on some KPIs and worse on others. R-01 is the case: the navigation cut
one value from 23 to 0 and made 20 of 168 values worse; it passed only with a decision id
(`reviews/2026-11-23-r01-explainer-round.md:19-21`). Here a Pareto choice is an **authorized
reference change that names the trade-off**:

```yaml
reference_change:
  subject: desk/tickets.list
  trade_off: effort-attention-load
  candidate: sha256:4f1c...
  previous: sha256:9a07...
  frontier: {campaign: c-2026-11-23-01, size: 14, dominated_by: none}
  better:
    - {requirement: attention.glance_cost.reference, cell: 1440x900/light/pointer/1.0, from: 2, to: 1}
    - {requirement: attention.offscreen_needed.reference, cell: 390x844/light/pointer/1.0, from: 3, to: 0}
  worse:
    - {requirement: effort.actions.reference, cell: all, from: 6, to: 7}
  unchanged_limits: all hard rules pass in every cell
  evidence: [run:ux-eval/2026-11-23T21:04Z/…]
  judgement: [jr-0012, jr-0015]
  authorized_by: {decision: D-…, policy_revision: …}
```

Rules for it [proposal]: (1) only requirements in a declared trade-off group may get worse (the
preset's `trade_offs`, ext); hard rules never; (2) the record names every requirement and cell that
got worse with both values, and at least one that got better; (3) the chosen variant is on the
frontier at the time of the decision (`dominated_by: none`); (4) the decision comes from a person or
an authorized policy, never from the build agent (critique item 15,
`reviews/2026-11-23-ui-iteration-loop-critique.md:48-49`); (5) the references stay fixed until the
next adoption (`design.md:122`).

## 7. Order of work with ratchet

uxui offers to be the first consumer of ratchet's "ESS UI behavior" integration (`design.md:156`;
`ratchet/research/consumer-input-ux-ui.md:8-13`): `ux-static@1` first (no layout model, proves the
record shape), then `ux-eval@1` on the terminal layout model as `layout-model/0`
(`state-space.md` § 4), then a pixel model.

## 8. Gaps to raise with ratchet

The first four were sent as consumer input (`ratchet/research/consumer-input-ux-ui.md:15-27`).

1. **Trade-offs.** Per-requirement references cannot express "worse here, better there". Proposal:
   a declared trade-off group and a reference change that names gains and losses (§ 6).
2. **Result scope.** A measurement needs a scope (state, journey, page, variant) and an aggregation
   that respects it; a page-level finding must not be summed per journey (R-01: 12 times).
3. **Noisy and exact measures.** Exact counts and modelled values: one sample, tolerance 0.
   Wall-clock values: median of N, a declared sample count, a band. The format should state the
   kind, so a gate cannot treat a sampled value as exact.
4. **Finding identity for UI.** (rule or metric id, `ess-ui` node path, condition cell), for "no new
   findings" (`design.md:111`).
5. **A design variant as target.** Ratchet's targets are revisions, builds, deployments and
   telemetry (`design.md:21`). A variant is a digest that may never be committed; its resolved
   target needs the fields of § 4.
6. **Subject dependencies.** A component used by many subjects: a change to it must re-evaluate and
   gate each using subject, and one reference change on the component may imply changes on them.
7. **Condition matrix.** A requirement evaluated per condition cell, the worst cell reported, every
   cell kept; and a word other than "environment" for it.
8. **Evidence volume.** 10^8 evaluations aggregate to one record per requirement, subject and cell;
   the record carries the digest of what it aggregates.
