# Metric and KPI catalogue

Status: design, 2026-11-23. Terms as in `docs/brief.md` § Glossary. "[Inference]" marks a claim no
source states. This catalogue ships as the ratchet preset family `web-ux@1`
(`presets/web-ux.ratchet.yaml`, `docs/ratchet.md`).

## 1. Evaluation data

Every metric is a pure function of these inputs. A metric names its version (`effort.klm_time@1`); a
new formula or new constants is a new version and a new evidence series.

| symbol | meaning | from |
|---|---|---|
| v, e, J, s | variant, environment, journey, UI state | campaign |
| S_J | the states of journey J within its bounds | state space (`state-space.md` § 2.2) |
| π_J | the cheapest path s_0 -a_1-> s_1 ... -a_n-> s_n to J's end: fewest actions, ties by `effort.klm_time` | action graph |
| target(a) | the node path an action acts on | action graph |
| box(n), vis(n), cov(n) | box in CSS px; visible fraction in the viewport (0-1); the node covering it | layout model |
| text(n), trunc(n) | laid-out text; truncated or not | layout model |
| fg(n), bg(n), size(n), tone(n) | resolved text colour, background, font size, tone | layout model |
| role(n), name(n), F(s) | accessible role and name; focus order of state s | layout model |
| W(s), (vw, vh) | document width; viewport size | layout model, environment |
| required(J), always, signal(J), cue | required nodes; always-visible nodes; the completion signal; the cue node | campaign declarations (not in `ess-ui/1` today, `state-space.md` § 8) |
| region(n) | the framed region holding n: a section, a column or area with `frame: shared`, an overlay | layout model |

Fixed rules for every metric:

- **Scope of a result.** A metric's result scope is state, journey, page or variant. A page-scope
  result is counted once per **finding identity** = (rule or metric id, node path, environment), never
  once per journey. R-01 counted one accessibility rule 12 times (`research/2026-11-23-build-proposal.md:48`).
- **Pooled, not median of medians.** Per-step values are pooled over all journeys before a median
  or a p95 is taken (the colleague's rule, private comparison with the colleague's repository).
- **Worst over environments.** A requirement holds when it holds in every gating environment; the
  value reported is the worst cell, and every cell is kept (private comparison with the colleague's repository).
- **No sampling.** Every metric here is deterministic: an exact count or ratio, or a modelled time
  from declared constants. Wall-clock timing belongs to the browser assessor and its median-of-N
  rules, not to this catalogue (`research/2026-11-23-external-machine-checkable-ux.md:68,71`).

## 2. Columns

- **dir**: ↓ lower is better, ↑ higher is better, = no direction.
- **res**: result scope: st state, jo journey, pg page, va variant.
- **targets**: the target scopes the metric is valid for (`concept.md` § 3): c component, w widget,
  s section, p page, x composition, d product; "all" = all six.
- **kind**: exact (a count or a ratio) or modelled (a modelled time or index), as
  `uxsearch.metrics.MetricKind` in this repository's `ess/domains/metrics.yaml`.
- **unit**: in the words of `uxsearch.metrics.MetricUnit` (same file), nodes, findings, journeys,
  steps, states, refusals, regions, keys, pairs, stops, values, commands, views, tones, sizes, gaps,
  edges, radii and colours are `count`; s is `ms` x 1000. `bits` (`effort.fitts_id`) and `level`
  (`attention.glance_cost`) have no unit there yet, and "no direction" (=) has no `Direction`.
- **S/D**: S static, from the document alone; D dynamic, needs the layout model.
- **role**: H hard rule (absolute limit, prunes), K KPI candidate (may be declared a KPI, gets a
  protected reference), D diagnostic only (shown, never gated).
- **source**: "delta" is a private comparison with the colleague's loop (not published), "external" is
  `research/2026-11-23-external-machine-checkable-ux.md`; `build-proposal.md` and
  `internal-building-blocks.md` are in `research/`; `ess-ui.md` and `ess-ui-test.md` are
  `ess/website/docs/reference/` in the ESS repository.

## 3. Completion

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | source |
|---|---|---|---|---|---|---|---|---|---|
| `completion.journey_completes` | A walker that picks targets by (role, name) only, among nodes with vis > 0 after scrolling, reaches J's end, and every `expect_command` of J holds. Value: journeys that fail. | journeys | ↓ | jo | all | exact | D | H = 0 | `screen_only_completion` (delta `:57`); preset `screen-only-completes` (`build-proposal.md:23`) |
| `completion.keyboard_parity` | The same walk with keys only (Tab, Shift+Tab, Enter, Space, arrows, Esc). Journeys that fail. | journeys | ↓ | jo | all | exact | D | H = 0 | preset `keyboard-parity` (`build-proposal.md:24`); the colleague's Tab walks (delta `:32`) |
| `completion.miss` | At J's end state c_J: \|{n ∈ required(J) : name(n) empty or vis(n) < 1}\|. Max over journeys. | nodes | ↓ | st | s p x d | exact | D | H = 0 | `completion_miss`, `open_at_readback` (delta `:56,58`) |
| `completion.false_complete` | \|{s ∈ S_J : signal(J) shown (vis > 0, enabled) and some n ∈ required(J) still open}\|. | states | ↓ | jo | s p x d | exact | D | H = 0 | `false_complete` (delta `:60`) |
| `completion.refusal_visible` | For each state reached by a refused command (fixture `refuses:`, `ess/website/docs/reference/ess-ui-test.md:312`): 1 if the error is not fully visible or not in region(target(a)). Sum. | refusals | ↓ | jo | all | exact | D | H = 0 | ESS: a refusal "is shown where the action happened" (`ess/website/docs/reference/ess-ui.md:2074`) [Inference: as a metric] |
| `completion.content_true` | Over states on π_J: \|{n with a declared source value : text(n) ≠ the formatted value, or trunc(n) and n not declared truncatable}\|. | nodes | ↓ | pg | all | exact | D | H = 0 | preset `content-true` (`build-proposal.md:27-28`) |

## 4. Effort

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | source |
|---|---|---|---|---|---|---|---|---|---|
| `effort.actions` | Operations on π_J with pointer input: clicks, key presses other than typed text, and one per scroll of up to one viewport to bring a target into view. | actions | ↓ | jo | all | exact | D | K | `actions_per_fix` (delta `:62`); KLM as a count (external `:14`) |
| `effort.actions_min` | The same on the document's action graph alone, without geometry: a lower bound. `effort.actions` - `effort.actions_min` is the cost of the layout. | actions | ↓ | jo | all | exact | S | D | [Inference] (`research/2026-11-23-internal-building-blocks.md:45-46`) |
| `effort.klm_time` | Σ over π_J of K·0.2 s + P·1.1 s + H·0.4 s + M·1.35 s + R. One M before each step whose target is not fixed by the previous step; H at each switch between pointer and keyboard; R = virtual-clock wait. Constants are inputs of the metric version. | s | ↓ | jo | all | modelled | D | K | KLM (external `:7-9`); replaces `time_to_fix`, `time_to_act` (delta `:63,65`) |
| `effort.fitts_id` | Σ over pointing moves i on π_J of log2(D_i / W_i + 1): D_i the distance between successive target centres, W_i the target's width along the move. Bits, not seconds: the coefficients are per device. | bits | ↓ | jo | all | modelled | D | K | Fitts, Shannon form (external `:15,17`) |
| `effort.keys_keyboard` | Key presses on the keyboard path: focus moves in F(s) between successive targets, plus activations. Typed text is excluded (the same in every variant). | keys | ↓ | jo | all | exact | D | K | [Inference] |
| `effort.wait_virtual` | Virtual time on π_J during which the next target's section is `loading`, from the declared per-view latencies (`ess-ui-test.md:54`). Shows serial `depends_on` chains. | s (virtual) | ↓ | jo | s p x d | exact | D | K | replaces `ready_lag` (delta `:64`) |

## 5. Attention

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | source |
|---|---|---|---|---|---|---|---|---|---|
| `attention.always_visible` | Over all s ∈ S: findings (n, e) with n ∈ always and vis(n) < 1 or cov(n) set. | findings | ↓ | pg | p x d | exact | D | H = 0 | preset `always-visible` (`build-proposal.md:21`) |
| `attention.offscreen_needed` | Steps i on π_J with vis(target(a_i), s_{i-1}) < 1. | steps | ↓ | jo | p x d | exact | D | K | `offscreen_needed` (delta `:67`) |
| `attention.glance_cost` | Per step i: 1 if target(a_i) is in region(target(a_{i-1})) or in the declared primary region; 2 if elsewhere with vis = 1; 3 if vis < 1. Median and p95 (nearest rank), pooled. | level 1-3 | ↓ | jo | p x d | exact | D | K | `glance_cost` (delta `:68`), "Next card" generalised to "the region just acted in" |
| `attention.visible_required` | min over s on π_J of \|{n ∈ required(J) shown in s : vis(n) = 1}\| / \|{n ∈ required(J) shown in s}\|. | ratio | ↑ | jo | p x d | exact | D | K | brief, "visible-required ratio" [Inference: formula] |
| `attention.regions_in_view` | Framed regions with vis > 0 in the viewport; median over π_J. | regions | = | jo | s p x d | exact | D | D | fixation-worthy regions (external `:22`) |

## 6. Layout

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | source |
|---|---|---|---|---|---|---|---|---|---|
| `layout.overflow` | Findings per (s, e): the document if W(s) > vw, and each n whose box leaves its container's box where the container does not scroll. At width 320 px this is WCAG 1.4.10 reflow. | findings | ↓ | pg | all | exact | D | H = 0 | preset `no-overflow` (`build-proposal.md:20`) |
| `layout.clipping` | Findings (n, e) with trunc(n) and n not declared truncatable. | findings | ↓ | pg | all | exact | D | K | `Region::text` today (`state-space.md` § 4) |
| `layout.shift` | Per transition s -> s' with no user action (live event, read finished, clock): Σ over nodes in both states of impact × distance, impact = area(box ∪ box') / (vw·vh), distance = max displacement / max(vw, vh). Sum per journey. | score | ↓ | jo | all | exact | D | K | CLS (external `:54`); `layout_shift` (delta `:69`), per transition instead of per minute |
| `layout.shift_after_input` | The same for transitions caused by a user action, for nodes outside region(target(a)). CLS excludes these (`hadRecentInput`); here they are kept apart, not dropped. | score | ↓ | jo | all | exact | D | D | external `:54-55` |
| `layout.density` | Σ area of content boxes in the viewport / (vw·vh); median over π_J. | ratio | = | jo | p x d | exact | D | D | [Inference] |

### 6.1 Visual ratchet (`ux-browser checks`, `visual`)

Not a metric of the layout model: a comparison of a live page's pixels with a committed baseline,
computed by `ux-browser checks` when its configuration declares `visual`
(`docs/browser-adapter.md`, Visual ratchet). A unit is one page, viewport, mode and shot (the
viewport at scroll 0, or the full page), its mask boxes blanked. A pixel differs when one of its
channels differs by more than the unit's threshold. A unit without a baseline is not run:
incomplete, never a pass.

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | tolerance |
|---|---|---|---|---|---|---|---|---|---|
| `visual.diff_share@1` | max over the page's units of (differing pixels / pixels), in millionths rounded up; 1 for a shot of another size than its baseline. | ratio | ↓ | pg | all | exact | D | K | per page and viewport: a share, and a per-channel threshold 0 to 255 |
| `visual.diff_px@1` | Σ over the page's units of the differing pixels. | px | ↓ | pg | all | exact | D | D | as above |

## 7. Load

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | source |
|---|---|---|---|---|---|---|---|---|---|
| `load.reading_words` | At each decision state s on π_J (the next step chooses among ≥ 2 targets): words of visible text, each node weighted by vis(n), with the viewport at the **fixed point of s**: scroll offset 0, or the smallest offset that shows target(a_{next}) fully if 0 does not. Long-form nodes (`rich_text`) excluded. Median and max, pooled. | words | ↓ | st | p x d | exact | D | K | `reading_load` (delta `:71`), R-01 defect fixed (below) |
| `load.reading_time` | `load.reading_words` / 238 words per minute. | s | ↓ | st | p x d | modelled | D | D | Brysbaert 2019 (external `:21`) |

The R-01 defect: `reading_load` counted "words in the end viewport", which depends on where the walk
stopped scrolling; it moved +2 to +80 in 8 walks with no change of content
(`reviews/2026-11-23-r01-explainer-round.md:22-23`). Here the viewport position is a function of the
state and its next target only, never of the walk's history, so the same state gives the same count.

## 8. Accessibility

Rules follow the ACT Rules Format: each has an id, applicability, expectations and a WCAG mapping;
outcomes are passed, failed, cantTell, inapplicable (external `:26`). The rule set is declared and
small; it runs on the layout model's accessibility tree, not on axe (`build-proposal.md:71`).

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | source |
|---|---|---|---|---|---|---|---|---|---|
| `a11y.act_failed` | Distinct findings (rule, n, e) with outcome failed, over all states. | findings | ↓ | pg | all | exact | D | H = 0 | ACT (external `:26`); the colleague's any-impact rule (delta `:42-43`) |
| `a11y.act_cant_tell` | Distinct findings with outcome cantTell. Each makes its requirement "insufficient evidence", never a pass. | findings | ↓ | pg | all | exact | D | D (insufficient) | external `:29`; `build-proposal.md:25` |
| `a11y.contrast_min` | min over text-on-surface pairs a theme and mode use of (L1 + 0.05) / (L2 + 0.05); limits 4.5 normal text, 3 large text and UI parts (WCAG 1.4.3, 1.4.11). From tokens, alpha composited on declared surfaces. | ratio | ↑ | va | all | exact | S | H | tokens hold the values (`internal-building-blocks.md:27-29`) |
| `a11y.name_missing` | Actions, inputs, icons and images without an accessible name. | nodes | ↓ | va | all | exact | S | H = 0 | `ess-ui/1` requires labels on icon, button, image only; `input` has none (`internal-building-blocks.md:21-23`) |
| `a11y.keyboard_unreachable` | \|{n interactive on some π_J or in always : n ∉ F(s)}\|, per state. | nodes | ↓ | pg | all | exact | D | H = 0 | WCAG 2.1.1; `build-proposal.md:30-31` |
| `a11y.focus_order` | Kendall tau distance between F(s) and the reading order of the same nodes (top, then left, within a region); summed over states on π_J. | pairs | ↓ | jo | all | exact | D | K | WCAG 2.4.3 [Inference: formula] |
| `a11y.focus_obscured` | Focus stops (s, n) where n, focused, has vis(n) = 0 behind a sticky region or an overlay. | stops | ↓ | pg | p x d | exact | D | H = 0 | WCAG 2.4.11; delta `:32` |
| `a11y.focus_trap` | States where no key sequence moves focus out of a region without completing it (Esc closing an overlay counts as out). | states | ↓ | pg | all | exact | D | H = 0 | WCAG 2.1.2; delta `:32` |
| `a11y.target_size` | Interactive n with w(n) < 24 or h(n) < 24 CSS px and no 24 px spacing circle free of other targets. | nodes | ↓ | pg | all | exact | D | H = 0 | WCAG 2.5.8 |

Text scale 200 % (WCAG 1.4.4) is an environment, not a metric: `layout.overflow`, `layout.clipping`
and `completion.content_true` are evaluated in it.

## 9. Consistency

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | source |
|---|---|---|---|---|---|---|---|---|---|
| `consistency.state_vocabulary` | Values of one model enum shown with ≥ 2 different (label, tone) pairs across the document (inline tones, `label_maps`, `tone_maps`). | values | ↓ | va | p x d | exact | S | K | H12 "the state vocabulary" (delta `:122-124`); tone maps exist for this reason (`ess-ui.md:2734`) |
| `consistency.command_label` | Commands (`does`) offered under ≥ 2 different labels. | commands | ↓ | va | p x d | exact | S | K | [Inference] |
| `consistency.primary_actions` | Regions with more than one primary-emphasis action. | regions | ↓ | pg | s p x d | exact | S | H = 0 | brief, Design rule "one primary action per page"; H1 (delta `:122-123`) |
| `consistency.pattern` | ESS views presented with ≥ 2 different composite kinds or styles across pages. | views | ↓ | va | d | exact | S | D | [Inference] |

## 10. Steering (only where the campaign declares a cue node)

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | source |
|---|---|---|---|---|---|---|---|---|---|
| `steering.cue_accuracy` | Share of states on π_J whose cue names a target the journey allows: the next action's target or an earlier one still open. | ratio | ↑ | jo | p x d | exact | D | K | `cue_accuracy` (delta `:72`) |
| `steering.cue_churn` | Cue changes per step (identity = kind + hash of text); flip-flops A -> B -> A within 2 steps or 5 s virtual time. | changes | ↓ | jo | p x d | exact | D | K | `cue_churn` (delta `:70`) |
| `steering.cue_leads` | Share of steps whose target lies in the cue's region or is named by the cue. A modelled stand-in for the share of a person's moves that follow the cue. | ratio | ↑ | jo | p x d | modelled | D | K | replaces `screen_led_share` (delta `:73`) |

## 11. Design proxies

These count properties that design principles name. They are **proxies**: a value says nothing about
whether a page looks good. Diagnostic only; a judgement record may turn one into a soft rule with a
weight (`concept.md` § 4).

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | source |
|---|---|---|---|---|---|---|---|---|---|
| `proxy.emphasis_per_region` | Per region: high-emphasis nodes (primary actions, `heading` text, `warning` and `danger` tones). Regions with a count other than 1. | regions | ↓ | st | s p x d | exact | D | D | principle "one emphasis per region" (delta `:241-244`) |
| `proxy.state_colour_share` | Area filled with tone fills (`*_fill` tokens) / (vw·vh), in states with no open problem. The colleague's bar: ≤ 10 %. | ratio | ↓ | st | p x d | exact | D | D | principle "calm when normal" (delta `:242`) |
| `proxy.urgency_levels` | Distinct urgency tones visible at once. | tones | ↓ | st | p x d | exact | D | D | "≤ 3 urgency levels" (delta `:243`); H11 (delta `:123`) |
| `proxy.type_scale` | Distinct font sizes a theme defines; max deviation of consecutive size ratios from the declared scale ratio. | sizes | ↓ | va | all | exact | S | D | "strict type scale" (delta `:243`) |
| `proxy.spacing_units` | Distinct gaps between sibling boxes that are not a multiple of the smallest space token. | gaps | ↓ | st | all | exact | D | D | "one spacing unit" (delta `:244`) |
| `proxy.alignment_edges` | Distinct left edges of boxes in the viewport. | edges | ↓ | st | s p x d | exact | D | D | [Inference] |
| `proxy.radii` | Distinct radius tokens in use. The colleague's bar: ≤ 4. | radii | ↓ | va | all | exact | S | D | "beauty from consistency" (delta `:244`) |

### 11.1 Live-page computation (`ux-browser checks`, `tokens`)

No engine computes the table's static forms. Version 1 of `proxy.type_scale`, `proxy.spacing_units`
and `proxy.radii`, the new `proxy.colours`, and `a11y.contrast_min` (§ 8) are computed by
`ux-browser checks` on a live page against the tokens its configuration declares (`tokens`,
`docs/browser-adapter.md`, Design tokens): per page, viewport and mode, over every element the page
draws, each count summed per page and the contrast the lowest. An output whose tokens are not
declared, or not defined by the page in a mode, is not run there: incomplete, never a pass.

| id | definition and formula | unit | dir | res | targets | kind | S/D | role | tolerance |
|---|---|---|---|---|---|---|---|---|---|
| `proxy.type_scale@1` | Distinct font sizes of drawn text not on the declared scale (a list of sizes, or base x ratio^k). | sizes | ↓ | pg | all | exact | D | D | 0.5 px |
| `proxy.colours@1` | Distinct colours of text, backgrounds and borders that match none of the declared colours (custom properties read from the root in each mode, and values). | colours | ↓ | pg | all | exact | D | D | 2 of 255 per channel, alpha 0.02 |
| `proxy.spacing_units@1` | Distinct gaps between sibling boxes and paddings that are not a multiple of the declared spacing unit. | gaps | ↓ | pg | all | exact | D | D | 0.5 px |
| `proxy.radii@1` | Distinct corner radii that match none of the declared radii (`full`: at least half the box's shorter side). | radii | ↓ | pg | all | exact | D | D | 0.5 px |
| `a11y.contrast_min@1` | min over drawn text on a known background (ancestors' background colours composited to the first opaque one) of (L1 + 0.05) / (L2 + 0.05), the text's alpha composited first, rounded down to 2 places; text below 4.5 (3 for large text) is named. | ratio | ↑ | pg | all | exact | D | D | none |

## 12. Counts

| group | metrics | H | K | D |
|---|---|---|---|---|
| completion | 6 | 6 | 0 | 0 |
| effort | 6 | 0 | 5 | 1 |
| attention | 5 | 1 | 3 | 1 |
| layout | 5 | 1 | 2 | 2 |
| load | 2 | 0 | 1 | 1 |
| accessibility | 9 | 7 | 1 | 1 |
| consistency | 4 | 1 | 2 | 1 |
| steering | 3 | 0 | 3 | 0 |
| design proxies | 7 | 0 | 0 | 7 |
| **total** | **47** | **16** | **17** | **14** |

Static: 9 (`effort.actions_min`, `a11y.contrast_min`, `a11y.name_missing`, the 4 consistency metrics,
`proxy.type_scale`, `proxy.radii`). Three static hard rules run in stage 1 of every campaign
(`state-space.md` § 6). A design rule may also be a predicate that is no metric (for example "a
command that ends an entity's lifecycle has a confirm", `concept.md` § 5).

## 13. Counterweights

Each lower-is-better metric has a cheap way to game it. It is declared only together with the
metric that catches the cheat (external `:64-65,94`).

| metric | gamed by | caught by |
|---|---|---|
| `effort.actions`, `effort.klm_time` | removing a confirm, hiding a field, an unsafe default | `completion.journey_completes`, `completion.content_true`, the confirm design rule |
| `effort.keys_keyboard` | taking controls out of the tab order | `a11y.keyboard_unreachable`, `completion.keyboard_parity` |
| `effort.fitts_id` | huge targets that push others out of view | `attention.offscreen_needed`, `layout.density` |
| `attention.glance_cost`, `attention.offscreen_needed` | hiding information | `attention.visible_required`, `completion.miss` |
| `load.reading_words` | deleting or truncating text | `completion.content_true`, `layout.clipping`, `attention.visible_required` |
| `layout.shift` | holding all content until every read finished | `effort.wait_virtual` |
| `layout.overflow` | clipping instead of wrapping | `layout.clipping` |
| `a11y.focus_order` | removing focus stops | `a11y.keyboard_unreachable` |
| `steering.cue_churn` | a cue that never changes | `steering.cue_accuracy` |
| `proxy.state_colour_share` | removing colour from problems | `completion.refusal_visible`, `completion.miss` |
| `a11y.target_size` | padding every target | `layout.density`, `effort.fitts_id` |

## 14. The colleague's 18 metrics

Source: a private comparison with the colleague's loop (not published).

| colleague metric | here | why |
|---|---|---|
| `completion_miss` | `completion.miss` | kept |
| `screen_only_completion` | `completion.journey_completes` | kept; truth values become `expect_command` inputs |
| `open_at_readback` | `completion.miss` at the end state | merged: the same count at one fixed state |
| `order_accuracy` | dropped | needs values a person entered in a live session; the walker enters fixture truth, so it would always read 100 % |
| `false_complete` | `completion.false_complete` | kept |
| `silent_wrong` | dropped | a property of the domain engine, not of the UI; belongs to ESS conformance of the domain |
| `actions_per_fix` | `effort.actions` | kept, per journey |
| `time_to_fix` | `effort.klm_time` | wall clock dropped; modelled time instead |
| `ready_lag` | `effort.wait_virtual` | with fixtures it is the declared latency, a constant; the virtual wait on the path still ranks designs |
| `time_to_act` | `effort.klm_time` | wall clock dropped; modelled time instead |
| `handling_time` | dropped | whole-session wall time with a person; no meaning without one |
| `offscreen_needed` | `attention.offscreen_needed` | kept |
| `glance_cost` | `attention.glance_cost` | kept, generalised |
| `layout_shift` | `layout.shift` | kept; per transition, not per minute, since time is virtual |
| `cue_churn` | `steering.cue_churn` | kept where a cue is declared |
| `reading_load` | `load.reading_words` | kept; measured at a fixed point (§ 7) |
| `cue_accuracy` | `steering.cue_accuracy` | kept where a cue is declared |
| `screen_led_share` | `steering.cue_leads` | the person's share cannot be simulated; a modelled stand-in, labelled |

Dropped: 3 (`order_accuracy`, `silent_wrong`, `handling_time`). Replaced by modelled values: 4
(`time_to_fix`, `time_to_act`, `ready_lag`, `screen_led_share`). The questionnaires (SEQ, TLX, SUS,
satisfaction; delta `:182-184`) and the `ux:speed` timings (delta `:177-178`) stay with people and
the browser.
