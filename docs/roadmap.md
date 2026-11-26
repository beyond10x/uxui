# Roadmap

## Trajectory (2026-11-24)

Vision:

- One measured UI loop for every repository of the organization: the checks, the ratchet evidence and
  the round live in uxui; a repository keeps only its pages, journeys and baseline.
- Checks are computed by our own code from a declared UI (`ess-ui/1`) or from the real page in a
  browser, deterministic, with no third-party scanner; evidence goes to ratchet.
- Search proposes the smallest measured change; people decide taste and trade-offs.

First consumer: the documentation-site loop of the `llm` repository (`ux/`). Its first gap: uxui ships
nothing it can call. The plugin is text, so `llm` keeps its own harness (`ux/core`, 1,447 lines), one of
eight different copies in seven repositories.

Waves:

1. `story:spec-round-3-target-adapter`: the specification describes browser targets, the lift and the
   binding table, and clears `specification-blocker:funnel-nouns`.
2. `story:browser-adapter`: uxui measures any page with a URL and writes `ratchet-evidence/1` for the
   page checks the loop copies share (overflow, page scroll, layout shift, contrast, target size,
   keyboard reach), checked against `llm`'s own numbers on its site.
3. `story:migrate-smallest-ui-loop-copy` (first `llm`) and `story:point-old-skill-copies-here`: the
   `llm` ratchet reads uxui's evidence for those checks, and its copies of them leave `ux/core`.

Release 0.1.0 holds what is merged on 2026-11-24; 0.2.0 adds waves 1 and 2 and ships `ux-browser` and `ux-static` as Linux archives; 0.3.0 adds the consumer values file and `layout_shift.load` to `ux-browser`; 0.4.0 measures a running server (`--url`) and waits for a ready condition the page sets; 0.5.0 makes the page-scroll and overflow rules configuration and locates always-visible items by CSS or `aria-current`; 0.6.0 fixes the page clock and walks the keyboard to declared controls without a skip link, with a focus mark; 0.7.0 starts Chromium without a GPU process and takes a consumer's own switches; 0.8.1 bounds always-visible items by `max_width`, skips content the page does not draw in the overflow check and pins the Chrome build; 0.9.1 adds `--version` to `ux-browser` and `ux-static` and runs the release gate on every pull request; 0.10.0 tests cover at the corners as well as the centre, measures the states of scripted journeys and prints Chromium's last lines when it does not start; 0.11.0 adds `ux-search`, the engine's first search (exhaustive climbs on ESS's terminal renderer, and on the browser adapter the campaign over the documentation landing page with `ux-search landing promote` for the chosen variant), records the first round of the `ui-iteration-loop` skill on the studio and adds the themes `dense.css` and `slot-jobs.css` (`CHANGELOG.md`).

State on 2026-11-24 and the next steps, in order. Each step names what it unblocks. Specification first, code
second; the first code is a thin vertical slice that measures the assumption the whole design rests on.

## State

- `plugin/`: the `ui-iteration-loop` skill, unchanged from its first version (rounds with people, browser
  runs, a baseline file). One round on its own explainer page: `reviews/2026-11-23-r01-explainer-round.md`.
- `ess/`: specification `uxsearch` v1, valid and compiled (216 declarations); no scenarios yet (156 outcomes
  without one), 13 UNMAPPED markers, 11 gaps in ESS (`ess/GAPS.md`).
- `docs/`: concept, state spaces, 47 metrics, mapping onto ratchet. `presets/`: a draft ratchet preset family.
- ratchet (`beyond10x/ratchet`) is in design; uxui offered to be the first consumer of its "ESS UI behavior"
  integration and sent it four gaps (trade-offs, result scope, exact vs sampled, finding identity).
- Decisions: layout model v0 is ESS's terminal renderer; UX declarations live in a `uxui-declarations/1` file;
  only a person accepts a trade-off.
- Contrast: APCA beside WCAG 2.x, in our own implementation of the published formula (no reference code
  copied). The owner, 2026-11-24: "Just keep using the APCA contrast thing [...] it's just two colors [...] it's
  super nice, it's working well".

## Next

1. **Specification round 2.** Scenarios for the 156 outcomes; resolve the 13 UNMAPPED markers or move each to
   `ess/GAPS.md`; an independent review of the spec by an agent that did not write it (design, scope,
   acceptance). Unblocks: generating anything from the spec.
2. **ESS gaps upstream, ordered by what blocks the engine first.** As stories to the ESS maintainers:
   (a) the actions available in a UI state and a cloneable or replay-free UI state, (b) a canonical digest of a
   UI state and of an `ess-ui/1` document, (c) a per-step trace in the `ess-ui-test` report, (d) rendering one
   component alone (a harness), (e) UX declarations as constructs (proposal from `uxui-declarations/1`).
   (a) and (b) decide whether exhaustive search is possible at all.
3. **Spike: measure the cost of one evaluation.** `docs/state-space.md` assumes 50 µs per layout call; nothing
   has measured it. Drive ESS's terminal renderer in process on one example page, time open + N steps +
   geometry read, report per-state cost and memory. Go/no-go for "exhaustive" per scope. Code in Rust, no
   third-party dependency beyond ESS. The tool exists: `ux-search cost` (`engine/search`), with the method,
   the rule per scope and a table still to be measured in `docs/cost-spike.md`.
4. **Thin vertical slice `ux-static@1`.** Read an `ess-ui/1` document, its `ess-ui-test/1` journeys and a
   `uxui-declarations/1` file; compute the static metrics (required nodes present, actions per journey,
   accessible names); write ratchet evidence records in ratchet's draft shape. No browser, no layout model.
   Proves the evidence contract with ratchet.
5. **First climb on a toy.** Two axes (theme x section order) on an ESS example page, exhaustive, layout model
   v0, a ranked frontier printed as text. Then the studio, generated from `ess/ui/studio.ui.yaml`.
   Exists: `ux-search climb` (`engine/search`), a first search on layout model v0 over a toy page: the
   ticket page of ESS's partner-portal example, 3 themes x 6 section orders, each variant checked, opened
   and drawn at two sizes, ranked on check errors, regions below the fold, regions cut by the frame and rows
   above the primary action, the frontier and the full ranking printed as text and held by a golden file.
   Themes tie: the terminal renderer reads no theme. The studio: `campaigns.climb`, section order inside each
   layout column, 120 variants, a frontier of 42; the page as written ranks 72nd (`docs/studio-climb.md`).
   Not yet: exploring UI states (each variant is its page as opened) and the browser stage.
   **Then the first real target, this repository's documentation site,** on a browser adapter: the engine
   with a headless browser as renderer, for any page with a URL (`docs/targets.md`).
   Exists: `ux-search landing` (`engine/search`), the engine's first campaign on the browser adapter, over
   the documentation landing page: 144 variants, 864 renders, a frontier of 16; the page as shipped ranks
   35th (`docs/campaign-2026-11-26-docs-landing-engine.md`). Not yet: a person's pick promoted as a pull
   request.
6. **Trial target.** One page or component of an admin console generated from an `ess-ui/1` document (a
   private repository; its fixtures make it I/O-free). Results stay in that repository; only generic findings
   come back here.
7. **Ratchet integration.** When ratchet's format settles: presets from `presets/web-ux.ratchet.yaml`, gates on
   merge and promote, and the migration of one of the organization's seven UI-loop copies (each with its own
   baseline file) onto ratchet + uxui, the smallest first.
8. **The skill.** Thin `ui-iteration-loop` to the round on top of the engine: findings become ratchet
   requirements, design decisions and taste stay with people. Take the generic process improvements of the
   loop's original author (decision template, reviewer sections, A/B decision order, survey rule) without their
   project details.

## Risks

- No study shows a computed metric orders close variants of one design as people do; model judges pick the
  better of a close pair 51.58 % of the time, and KLM-style models get the direction wrong in 2 of 6 measured
  differences (`research/2026-11-24-evidence-for-ui-metrics.md`, in worktree uxui-research). Picking the best
  of thousands is best-of-n optimisation. Counter: tie bands, the Pareto frontier, prefer the smallest change,
  and a small human calibration set per campaign before a ranking is trusted.
- Layout model v0 has cells, not pixels: overflow, clipping and density are coarse until a pixel model exists.
- Exhaustive search depends on step 2 (a) and on the cost measured in step 3.
- The metrics rank what is declared; aesthetics and focus stay judgement. The original loop's own data shows
  hermetic metrics did not separate 28 design variants.
- Two older copies of the skill exist outside this repository; until they point here, they drift.
