# Brief: the UX search engine (working title)

Status: brief for the first specification round, 2026-11-23. Source of the idea: the owner, 2026-11-23:
"imagine a tight integration with ESS UI specs [...] programmatically, without AI, simulating variants of the
UI or the layout, like, you know, mutating it, brute forcing it [...] more or less without any I/O, brute force
all combinations. And because we're able to programmatically evaluate KPIs, metrics and stuff, we can [...]
faster find the best composition of UI [...] You have themes, you have templates, you can switch them easily,
and then you can iterate, like, in real time, scan through all the combinations [...] there are obviously some
rules [...] and you can observe it climbing that hill. [...] the current approach [...] is pretty slow [...]
bound to these scenarios [...] with ESS and all of the nice tooling there, we should be able to just simulate
all of that [...] the idea of exhausting the state space. [...] part of this [...] will be defining general or
common KPIs and metrics with the ratchet thing."

First step: documents and an ESS specification of the outcome. No code.

## What it is

A system that takes a UI written as an `ess-ui/1` document, generates its design variants by rule (themes,
templates, layouts, component choices, interaction patterns), evaluates every variant against every reachable
UI state of every journey with no I/O (fixtures, scripted live events, a virtual clock, a deterministic layout
model), computes a fixed catalogue of UX metrics as pure functions, and searches the design space for the best
compositions: exhaustively where the space is small, by pruned or guided search where it is not. A person
watches the search climb in a live studio, switches themes and templates by hand, compares candidates, and
promotes one. Promotion is a reviewed change to the `ess-ui/1` document, gated by ratchet.

## What it replaces in the colleague's loop (compared in a private note)

| colleague's loop | here |
|---|---|
| one hand-built UI, A/B behind URL flags, a few variants | the design space generated from the spec; thousands of variants |
| hermetic replays of a handful of recorded calls | every reachable state of the UI model, enumerated within bounds, from ESS fixtures |
| a browser run per check (minutes, a shared build pool) | a pure evaluation function per (variant, state, environment), cached by digest |
| 18 metrics in one TypeScript file | a declared metric catalogue, each metric a typed definition, ratchet measurements |
| a ratchet per scalar; trade-offs need a decision id | a Pareto front over declared objectives; a KPI is a declared weighting; ratchet guards the promoted choice |
| hermetic metrics cannot rank designs (colleague's own finding) | still true for aesthetics; the engine ranks what is declared and leaves taste to people, shown side by side |

## Glossary (use these words in the spec and the docs)

- **Base document**: the `ess-ui/1` document being improved.
- **Design space**: the set of variants a base document admits. Declared as **axes**.
- **Axis**: one dimension of variation with a finite value set. Kinds: `theme` (a token set), `template` (a
  shell, page or section template), `layout` (region placement, section order, density), `component` (an
  alternative composite kind for the same data, e.g. table or cards), `interaction` (e.g. inline confirm or
  overlay; save on change or submit), `copy` (label variants).
- **Mutation operator**: a deterministic, validity-preserving transform of an `ess-ui/1` document (reorder
  sections, move an action to the header, swap a composite kind, split into tabs, change a density token).
  Declares its preconditions; its result passes `ess ui check`.
- **Variant (candidate)**: base document plus a choice per axis and a sequence of operator applications.
  Identified by the digest of its canonical form; two routes to the same document are one variant.
- **Design rule**: a constraint on a variant. **Hard** rules prune (a variant breaking one is never evaluated
  further: always-visible nodes, one primary action per page, keyboard reachability). **Soft** rules score.
- **Journey**: a task through the UI, from `ess-ui-test/1` steps plus the ESS system's commands and views.
- **UI state**: one reachable configuration of a page (data from fixtures, open overlays, selection, focus,
  virtual time). The **state space** of a journey is all states reachable within declared bounds.
- **Environment**: viewport, theme mode, input mode (pointer, keyboard), text scale, reduced motion. An
  evaluation condition, not a design choice.
- **Layout model**: a deterministic function from (variant, state, environment) to geometry and an
  accessibility tree. No browser, no network, no clock.
- **Evaluation**: computing every metric for (variant, journey, environment) over the state space. Pure; its
  result is cached by the digests of its inputs.
- **Metric**: a typed, pure function over evaluation data with unit, direction (lower or higher is better),
  scope (state, journey, page, variant), and kind (exact count or modelled time). Becomes a ratchet measurement.
- **KPI**: a project's declared objective: one metric or a declared aggregation of metrics, with a weight. A
  set of KPIs defines the **Pareto front**.
- **Search campaign**: a run of the engine over a design space x journeys x environments with a **strategy**
  (exhaustive, beam, hill climbing, Pareto evolution), a budget (variants, time) and hard rules. Emits progress
  events: the climb.
- **Frontier**: the non-dominated variants found so far.
- **Promotion**: choosing a frontier variant; produces the `ess-ui/1` diff, the ratchet evidence and a
  proposed protected reference. A person (or an authorized policy) accepts it.
- **Studio**: the live UI of the engine: the climb, the frontier, side-by-side variants, manual axis switches.
  Itself specified as an `ess-ui/1` document.
- **Judgement record**: a person's or a review agent's pairwise preference or finding on variants. Never a gate
  input; it becomes a design rule, a KPI weight or a finding.

## Ratchet (beyond10x/ratchet, design phase: its docs/design.md and vision.md)

The engine does not keep its own baseline. Metrics are ratchet measurements, hard rules and gates are ratchet
requirements, the promoted variant's values become protected references, evidence records carry the resolved
target (variant digest, layout-model version, environment). "Insufficient evidence" (bounds hit, a state not
evaluable) is never a pass. A metric catalogue ships as a ratchet preset family (`web-ux@1`).

## Out of scope for this round

Code; the layout model's algorithm (specified by its contract only); aesthetics beyond declared proxies;
replacing ESS's own renderers.
