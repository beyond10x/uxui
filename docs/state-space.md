# The two spaces, the layout model, and search

Status: design, 2026-11-23. Terms as in `docs/brief.md` § Glossary; the worked example is the ticket
page of `concept.md` § 5. "[Inference]" marks a claim no source states. Paths starting `ess/website/`
or `ess/crates/` are in the ESS repository (github.com/beyond10x/ess).

## 1. Two spaces

| space | element | made of | who chooses |
|---|---|---|---|
| design space | a variant | axes x mutation operators applied to the base document | the search |
| UI state space | a UI state | fixtures x actions x overlays x focus x virtual time, per journey | the user, simulated |

Evaluation crosses them with the environments; sizes multiply, so each space is reduced on its own first.

## 2. Sizes

### 2.1 Design space

For axes A_1 ... A_k and operators O applied at most d times, |D| ≤ (Π_i |A_i|) x Σ_{j=0..d} |O|^j.
Valid variants meet every operator precondition and pass `ess ui check`
(`ess/website/docs/reference/cli.md:1823`); distinct variants are valid ones up to the canonical
digest. Worked example (d = 0): 11,664 by product, 5,184 valid, 3,456 after the static hard rule.

### 2.2 UI state space

A UI state is (page, page state, section lifecycle per section, rows shown, selection, open overlay
and its input, focus, applied live events, virtual time). For a page with fixture sets F:

|S| = Σ_{f ∈ F} Σ_{q ∈ Q(f)} (1 + |R(f,q)| x |O|) x |L| (+ loading states)

with Q(f) the reachable page-state values (search, sort, page), R(f,q) the selectable representative
rows, O the overlay states, L the live-event phases. Focus is no factor: keyboard mode sweeps each
state once per focus stop, linear in the stops.

Worked example (detail as a section):

| dimension | values | n |
|---|---|---|
| fixture set F | empty (0 rows), one (1), page (25), paged (60, 3 pages of 25), failed (`fails:`), slow (25, 2 s latency) | 6 |
| search | blank, hit, miss | 3 |
| sort | default, subject asc/desc, updated asc/desc | 5 |
| page | 1-3, only for "paged" with a blank search | ≤ 3 |
| selection R | none; one row; or first, last, longest subject | 1-3 rows |
| overlay O | none, edit clean, edit invalid, edit refused, close confirm | 5 |
| live L | before / after one inserted ticket at 2 s | 2 |

| F | query states shown / miss | states per live phase | x L |
|---|---|---|---|
| empty | 15 (rows: none) | 15 | 30 |
| failed | 15 (rows: none) | 15 | 30 |
| one | 10 / 5 | 10 x (1 + 1 x 5) + 5 = 65 | 130 |
| page | 10 / 5 | 10 x (1 + 3 x 5) + 5 = 165 | 330 |
| paged | 20 / 5 | 20 x 16 + 5 = 325 | 650 |
| slow | as "page", plus 1 loading state | 165 | 330 + 1 |
| **total** | | | **1,501** |

A re-read keeps the rows shown (`refreshing: keep_rows`, `ess/website/docs/reference/ess-ui.md:892`): no extra states.

### 2.3 Sizes per target scope

| scope | design space | states | environments | evaluations | exhaustive? |
|---|---|---|---|---|---|
| component: the ticket `collection` in a harness | 144 = style 3 x row-action `as` 3 x space 2 x body size 2 x column order 4 | 97 | 16 = widths 320/480/768/1200 x text scale 2 x mode 2 | 223,488 | yes |
| widget or section | (axes of its body) | states of its composites and read | as component | between the two rows | usually |
| page: the ticket page | 3,456 | 1,501 | 24 | 124,498,944 | staged (§ 6) |
| composition (n pages on one journey) | Π page spaces x shell axes | Σ page states on the journey | 24 | | no; bottom-up |
| product: 3 such pages | 3,456^3 ≈ 4.1 x 10^10 | Σ | 24 | | no; top 3 per page = 27, x theme 3 x navigation 2 = 162 |

Component states: failed 1, loading 1, 0 rows 5 (sort), 1 row 5 x 2 (selection), 25 rows 5 x 4,
60 rows 5 x 3 pages x 4: 1 + 1 + 5 + 10 + 20 + 60 = 97. Layout calls: 144 x 8 geometric
environments x 97 = 111,744, about 5.6 s at an assumed 50 µs per call.

Measured 2026-11-24 (the scaffold spike, `docs/scaffold-spike.md` on branch `uxui-scaffold`, in process over
the ESS 0.98.0 crates, host load average about 100, so the times are high): check a variant 2.6 ms, open a
page 1.2 ms, draw one 80x24 state 0.4 ms. A layout call therefore costs about 400 µs, eight times the
assumption, plus 3.8 ms per variant and page. The component example becomes about 45 s of drawing plus
144 x 8 x 3.8 ms = 4.4 s; the call budget of § 6 shrinks from 2.4 million calls to about 300,000 per 120 s
on one core. Through the `ess` CLI a variant costs about 190 ms (check and test as processes), so the engine
calls ESS in process. The cost of one step between states (replayed from open: ESS 0.99.4 has no UI state
API, gap P0 1) is measured by `ux-search cost`: `docs/cost-spike.md`.

### 2.4 Evaluation count

E = |V| x |Env| x Σ_J |S_J|. The layout model runs fewer times: G = |V / ≡geo| x |Env / ≡geo| x |S|,
where ≡geo merges inputs that differ only in what does not move a box: colour tokens, theme mode,
input mode, reduced motion. Ticket page: E = 3,456 x 24 x 1,501 = 124,498,944; G = 2,304 x 6 x 1,501
= 20,749,824 (theme `calm` is colour-only, so 3 themes are 2 geometric classes).

## 3. Evaluation without I/O

| need | ESS today | source |
|---|---|---|
| data of every read | fixture files per view, `derived` ById views, answers `by_params`, `fails:` for a failed read | `ess-ui.md:2143-2200`, `ess/website/docs/reference/ess-ui-test.md:279-312` |
| command outcomes | `commands.<c>: {refuses: <code>}` per test; with `--model` a confirmed command changes the fixture rows "the way the model says" | `ess-ui-test.md:312`, `cli.md:1886` |
| domain behaviour | ESS's interpreter: state in, command in, every outcome the model allows out | `ess/crates/verify/ess-conformance/src/interpret/execute.rs:1-7,400` |
| live events | channel scripts on one virtual clock from zero; `advance`, `play`; looping capped at 10,000 cycles per move | `ess-ui-test.md:225-251,349-353`, `ess/crates/ui/ess-ui-test/src/runner.rs:27` |
| loading states | per-view latency on the virtual clock | `ess-ui-test.md:54,340` |
| driving the UI in-process | `ess_ui_tui::App`: `from_text`, `open_page`, `open_page_with_state`, `open_overlay`, `keys`, `advance`, `section_state`, `regions` | `ess/crates/ui/ess-ui-tui/src/app.rs:434,501,681,706,761,785,798,674,657` |

File I/O left: fixtures are read once at load; file-placed UI state lives under `Options::state_dir`
(`app.rs:27-31`) [Inference: the engine needs an in-memory store for it, in ESS or its own].

## 4. The layout-model contract

**Inputs.** (1) the canonical variant document, widgets expanded (`ess-ui.md:1412-1420`); (2) a UI
state as in § 2.2; (3) an environment: viewport width and height in CSS px, text scale, theme mode,
input mode, reduced motion; (4) a font-metrics table (advance widths, line heights per family,
size and weight) as data with its own digest; (5) the model version.

**Outputs**, per state, keyed by canonical node path (`ess-ui.md:228-297`):

- box (x, y, w, h), clip box, visible fraction in the viewport, the node covering it, layer;
- text as laid out: lines, truncated or not, full text (as `Region::text` does today);
- resolved style: text colour, background colour, font size and weight, tone;
- accessibility node: role, accessible name, states, focusable, tab position;
- document extents (scroll width and height) and the focus order;
- refusals: constructs the model does not lay out (for example `graph_editor`, `chart`), each with
  its node path. A refusal makes every metric that reads that node "insufficient evidence".

**Determinism and versioning.** The same inputs give byte-identical output: no clock, no randomness,
no host fonts, coordinates in fixed point of 1/64 px [Inference: a choice, for digests stable across
machines]. Each output names `layout-model/<n>` and its font-table digest; a new version starts a new
evidence series in ratchet (`ratchet/docs/design.md:126`). The `ux-walk@1` browser assessor
(`research/2026-11-23-build-proposal.md:32-34`) measures fidelity as box deltas per node path and
environment [Inference].

**What ESS's terminal renderer already gives.** It records, per frame and node path, the cells each
node occupies (`Region {path, row, area, text}`, `ess/crates/ui/ess-ui-tui/src/app.rs:53-73`) for the
page header and its actions, section boxes and children, the open overlay, a table's headers, row
lines, cells and row actions (`ess-ui-test.md:395-400`). It renders into a test backend of any size
(`ess/crates/ui/ess-ui-tui/src/view.rs:197-201`; tests use 160 x 240 cells,
`ess/crates/ui/ess-ui-test/src/screen.rs:15-16`; `--screen-once` up to 1000 x 1000,
`cli.md:404`). That is a deterministic layout model for a cell grid: enough for in-view, overflow,
truncation and the action graph. It lacks pixels, tokens (the terminal "reads no space, type or
radius", `ess-ui.md:2599`), an accessibility tree, a Tab focus model (it is driven by letter keys,
`ess-ui-test.md:383-394`), and primitives inside widgets. [Inference: it can serve as
`layout-model/0` to prove the contract and the evidence shape before a pixel model exists.]

## 5. Bounds and pruning

1. **Hard rules first.** Static hard rules (`metrics.md`, column "static") run on the document alone,
   in microseconds, before any state is built. A variant breaking one is never laid out. Dynamic hard
   rules run on journey-path states before the full state space.
2. **Symmetry by canonical digest.** A variant's digest is taken over its canonical form (sorted map
   keys, defaults written out, widgets expanded). A state's digest is taken over its canonical tuple.
   Rows with the same shape (field lengths in the same bucket, same null pattern, same tones) are
   interchangeable; only representatives are enumerated (first, last, longest text, one per tone)
   [Inference].
3. **Memoisation by input digest.** An evaluation is cached under H(variant digest, state digest,
   geometric environment, layout-model version, font-table digest, fixture digest). Metrics are
   cached under the evaluation digest and the metric's version.
4. **Incremental re-evaluation.** Each axis value and operator declares the node paths it changes.
   When one axis changes, only states that show one of those paths are laid out again; with a
   compositional model (columns laid out independently at a given width) only the changed column is
   [Inference: needs the layout model to expose per-region results].

## 6. Strategies

| strategy | use when | parameters |
|---|---|---|
| exhaustive | G ≤ the call budget (about 300,000 calls = 120 s at the measured 400 µs on one core; 2.4 million at the assumed 50 µs) | none |
| beam | operators dominate: structural moves of unbounded depth | width w, depth d |
| hill climbing with restarts | one scalar KPI (a declared weighting), a large axis product | restarts r, steps |
| Pareto evolution | several KPIs, a large space, a frontier wanted [Inference: NSGA-II style; not in the sources] | population p, generations g |

Every strategy takes a seed; the same seed and inputs give the same frontier and the same climb, so
the studio can replay it. Every campaign evaluates in stages:

| stage | evaluates | ticket page |
|---|---|---|
| 1 static | all valid variants, document only | 5,184 checks, 1,728 pruned |
| 2 journey paths | the states on the shortest path of each journey (J1 7, J2 9, J3 9) | 2,304 x 6 x 25 = 345,600 calls, ≈ 17 s |
| 3 full state space | the frontier only (say 50 variants) | 50 x 6 x 1,501 = 450,300 calls, ≈ 23 s |

Stage 3 can change the frontier: a variant can fail a hard rule in a state no journey path visits.
Such a variant leaves the frontier, and stage 2 results of the next candidates are taken in.

## 7. Insufficient evidence

An evaluation that cannot finish is "insufficient evidence", never a pass (`ratchet/vision.md:32-33`):
(1) a journey hits its state bound (default 65,536, the bound Canon puts on a protocol's state space,
`canon/.engineering/planning/story/canon-check.md:172-175`; the record names each dimension and its
value count, as Canon's refusal does); (2) the layout model refuses a node the metric reads (§ 4);
(3) the terminal path refuses a step (`ess-ui-test.md:402-427`) or a clock move passes 10,000
cycles; (4) an accessibility rule says `cantTell`; (5) a view or channel has no fixture
(`script_per_channel`, `ess-ui.md:2259`). A frontier variant with an insufficient result cannot be
promoted until the bound is raised, the model learns the construct, or a policy change drops the
metric from the campaign.

## 8. ESS: what exists and what is missing

Exists, besides § 3 and § 4: the section lifecycle as a state machine (`ess-ui.md:832-851`), renderer
capabilities and `--lacks` (`ess-ui.md:1622-1664`, `cli.md:1836`), and a seeded explorer of domain
command sequences with shrinking (`ess/website/docs/guides/verify/explore.md:9-50`), not of UI states.
Missing, to raise with ESS:

1. UI state enumeration: the actions available in a state, and a successor function. `App` has no
   `Clone` (`app.rs:433-434`), so a state cannot be forked; each state is replayed from `open`.
2. A canonical state digest.
3. Pixel geometry, tokens in layout, an accessibility tree, Tab focus order (`ess-ui.md:2599`;
   `research/2026-11-23-internal-building-blocks.md:24-26`).
4. A harness to render one composite or widget alone at a given width (component scope).
5. Declarations the metrics need: required nodes, always-visible nodes, the completion signal, a cue
   node (`research/2026-11-23-internal-building-blocks.md:178-184`).
6. Default props for built-in composites outside widgets and page kinds (`ess-ui.md:710`).
7. A per-step trace in `ess-ui-test-report/1`, which holds pass or fail only
   (`ess-ui-test.md:355-376`).
8. An in-memory store for file-placed state (§ 3).
