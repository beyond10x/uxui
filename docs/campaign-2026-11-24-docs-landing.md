# Campaign 2026-11-24: the documentation landing page

Status: run twice, 2026-11-24. The first improvement campaign on real numbers: the whole landing
page `/uxui/` of the documentation site (hero and the four sections below it), through the funnel
of `docs/targets.md` and the spike tools of `docs/scaffold-spike.md`. Every axis changes the page's
sources, so the chosen variant is a diff to `website/product.json` and one CSS file.

Run 1 ranked a page first that hid content (the KPI tiles and three feature cards). Decision D1 = A:
hiding content to win on layout KPIs games the metric. Run 2 adds a content KPI and a hard rule that
keeps every section and card of the current page; its winner is the one applied.

Code: `tools/scaffold/campaign.mjs` (the funnel), `tools/scaffold/campaign-server.mjs` (the studio's
backend, `/compare`, the built site with a variant), `tools/scaffold/lift/docs-landing/` (the lift,
the space, the binding, the in-page renderer and `landing.css`). Spike code: Node with no packages,
Chromium over the DevTools protocol, the cell reader `tools/scaffold/cells/`.

## Space

Seven axes over the sources, 3 x 3 x 2 x 2 x 2 x 2 x 2 = **288 variants**
(`tools/scaffold/lift/docs-landing/space.json`).

| axis | source | values |
|---|---|---|
| actions | `product.json` `/product/actions` (count and order) | all five as today; all five with "What exists today" second; "Install the plugin" and "What exists today" |
| section_order | `product.json` `/sections` | features, flow, status, related (today); status second; status first |
| kpis | `product.json` `/product/kpis` | shown (shipped, decided, planned); hidden (`[]`) |
| features | `product.json` features `items` | six (today); the first three |
| lede | `product.json` `/product/lede` | today's 64 words; a 33-word version written for the campaign |
| density | `website/src/css/landing.css` `--uxui-density` | 1; 0.75 (hero and section spacing) |
| heading_scale | `website/src/css/landing.css` `--uxui-heading-scale` | 1; 0.85 (hero and section headings) |

`landing.css` reproduces the docs-system theme's own values when both custom properties are 1, so
the file is a no-op until a value changes. The current page is the variant with every first value.

Hero actions are navigation, not content: the actions axis may drop four of them. Their targets stay
reachable from the navbar and the sidebar, and the journeys measure that.

## Hard rules

Checked at 1440x900, 1024x768 and 390x844, each in light and dark (6 environments):

| rule | how |
|---|---|
| the primary action is fully in view without scrolling | `runtime.js` `measure`, primary `.b10x-hero__actions > a.b10x-button` |
| no horizontal scroll | `scrollWidth` against the viewport |
| contrast WCAG 2.x 1.4.3: 4.5:1, 3:1 for large text | every visible text against its composited surface |
| every control has an accessible name | every visible interactive element |
| every journey is reachable | a route exists (below) |
| every section and card of the current page stays present (run 2) | the 34 content ids of the current page: 3 KPI tiles, 4 sections, 6 feature cards, 6 flow steps, 14 status items, 1 related-tool card. Stage 1 reads them from the patched sources (`lift.mjs` `contentOf`), stage 2 from the rendered page in each environment (`landing-page.js` `content()`, present and visible) |

Stage 1 checks the lifted document (`ess ui check`, no errors), the primary action in the first
screen of the terminal at both sizes, that every journey has a route and (run 2) the content rule.
A variant that drops content is pruned in stage 1 and checked again in stage 2.

## KPIs and their evidence

Each is the worst value over the six environments (tap targets: 390x844). Evidence grades are those
of `research/2026-11-24-evidence-for-ui-metrics.md` § 12.

| KPI | better | measured as | evidence | role here |
|---|---|---|---|---|
| words before the first action | lower | words of visible text above the primary action (navbar included) | moderate: concision +58 % usability [MN97], 4.4 s per 100 words [Wei08] | soft |
| px to the primary action | lower | top of the primary action, page px | moderate: the first screenful gets 57 % of viewing time [Fes18]; the hard rule keeps it in view | soft |
| actions per journey | lower | sum over "install the plugin", "what exists today", "find the metric catalogue" of the cheapest route: one click plus one action per screenful scrolled; a navbar link behind the menu toggle costs one more; a route through a reference page and its sidebar costs one more (two on a narrow viewport) | weak: count-only models RMS 34-49 % [CMN80], clicks do not predict success [Por03] | tie-breaker |
| regions crossed | lower | per journey, the regions (navbar, hero, each section, footer) whose top is at or above the link the route clicks; summed | moderate as Tullis grouping, relative only [Tul84] [Don91] | soft |
| tap targets under 44 px | lower | visible links and buttons of the landing body with a side under 44 px, at 390x844 | consensus; empirical touch thresholds 7.7-9.6 mm [Par06], errors over 40 % under 8 mm [Hen11] | soft |
| content kept (run 2) | higher | content ids of the current page present and visible; the fewest over the six environments | a counterweight against the proxies above (research note § 10), not a usability measure | ranked, and a hard rule at 34 of 34 |
| APCA Lc of body text (under 24 px) | higher | minimum Lc, APCA-W3 0.0.98G | none from human readers | shown, not ranked |

The ranking score is the mean of the ranked KPIs, each divided by the current page's value (content:
the current page's value divided by the variant's), plus 1000 per failed hard rule. Run 1 ranked
five KPIs, run 2 six. The frontier is Pareto over the ranked KPIs among variants that pass every
hard rule. The winner is the best score; within 2 % of it, the variant with the fewest source
changes.

## Run 2: with the content rule

One run, Chromium headless, shared host at load average about 86.

| stage | what | time |
|---|---|---|
| 0 lift | `lift.mjs` lifts the current sources into one `ess-ui/1` document (hero, four sections, 11 stub pages for the link targets), `ess ui check`: 0 findings; 15 binding selectors each match one element of the built page | 52 ms; drift 0 |
| 1 cells | 288 variants: patch the sources, lift, write (487 ms in all); `uxui-cells` in process at 120 and 48 columns, drawn 600 rows high | 8.0 s; 10.5 ms per variant and size (median) |
| 2 browser | the top 30 of stage 1 that pass every hard rule, and the current page, rendered from their patched sources over the built site, 6 environments each | 15.7 s; 386 ms per variant (median), render 1.4 ms, page load 1.4 s |
| 3 shots | the top 5 and the current page: light and dark at 1440 and 390, a whole page at 1440 | 5.4 s |

### Space after pruning

| | variants |
|---|---|
| space | 288 |
| fail the content rule in stage 1 (KPI tiles hidden, or three feature cards) | 216 |
| pass every hard rule of stage 1 | 72 (actions 3 x order 3 x lede 2 x density 2 x headings 2) |
| measured in stage 2 (top 30 and the current page) | 31 |
| fail a hard rule in stage 2 | 0 (content kept 34 of 34 in every environment of every variant) |

### Stage agreement

Over the 31 variants both stages scored:

| | run 2 | run 1 |
|---|---|---|
| Kendall's tau-b | 0.059 | 0.476 |
| tau-a (ties count against) | 0.047 | 0.383 |
| pairs concordant / discordant | 169 / 147 | 248 / 70 |
| pairs tied in stage 1 only / in both | 121 / 26 | 122 / 25 |
| stage-1 top 3 at stage-2 ranks | 23, 16, 19 | |
| stage-2 top 3 at stage-1 ranks | 16, 20, 24 | 16, 12, 10 |

Stage 1 is nearly blind on what run 2 leaves to rank. With the content axes fixed, the remaining
variance is the lede (stage 1 ranks it the wrong way round, below), density and heading scale (stage
1 draws neither) and section order (moves no KPI in either stage). Stage 1 put the 12 long-lede
variants first; the browser put the 12 short-lede variants first.

### Frontier and top six

The frontier has three entries, the same page with the sections in three orders: section order moves
no KPI. KPI columns: words, px to the primary action, actions, regions, small tap targets, content
kept. Every variant measured keeps 34 of 34 content ids, contrast minimum 5.78:1 and APCA Lc minimum
59.7 (no axis touches colour).

| stage-2 rank | stage-1 rank | digest | actions, order, lede, density, headings (kpis shown, six features in all) | words | px | actions | regions | targets | content | score | changes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| current | 49 | `sha256:295e68b2401c` | all five, today, today, 1, 1 | 86 | 652 | 5 | 7 | 24 | 34 | 1.000 | 0 |
| 1 (frontier) | 16 | `sha256:c3c26fd790e4` | install and status, today, short, 0.75, 0.85 | 53 | 433 | 5 | 7 | 21 | 34 | 0.859 | 4 |
| 2 (frontier) | 20 | `sha256:27dd94ec20ad` | install and status, status second, short, 0.75, 0.85 | 53 | 433 | 5 | 7 | 21 | 34 | 0.859 | 5 |
| 3 (frontier) | 24 | `sha256:a408dc321512` | install and status, status first, short, 0.75, 0.85 | 53 | 433 | 5 | 7 | 21 | 34 | 0.859 | 5 |
| **4 (chosen)** | 14 | `sha256:2fabfc84e2b6` | install and status, today, short, 1, 0.85 | 53 | 472 | 5 | 7 | 21 | 34 | 0.869 | 3 |
| 7 | 15 | `sha256:6976397db7c1` | install and status, today, short, 0.75, 1 | 53 | 475 | 5 | 7 | 21 | 34 | 0.870 | 3 |
| 10 | 13 | `sha256:e2e922fb9fcd` | install and status, today, short, 1, 1 | 53 | 501 | 5 | 7 | 21 | 34 | 0.877 | 2 |

### The chosen variant

`sha256:2fabfc84e2b6`, applied to the sources: two hero actions ("Install the plugin", "What exists
today"), the 33-word lede, heading scale 0.85. KPI tiles, all six feature cards, the section order
and the density stay as they are.

It is the campaign's own pick: rank 4 scores 0.869, within 2 % of rank 1's 0.859, with three source
changes against four (rank 7 also has three and scores lower; rank 10 has two and is 2.03 % off).
Ranks 1-3 add density 0.75, which brings the primary action 39 px higher at the worst viewport and
changes nothing else; it is one line of `landing.css` if wanted.

Against the current page, the same six KPIs in the same six environments:

| KPI | current | chosen | delta |
|---|---|---|---|
| words before the first action | 86 | 53 | -33 (-38 %) |
| px to the primary action (worst of the six: current at 390x844, chosen at 1440x900) | 652 | 472 | -180 (-28 %) |
| actions over the three journeys | 5 | 5 | 0 |
| regions crossed | 7 | 7 | 0 |
| tap targets under 44 px (390x844) | 24 | 21 | -3 |
| content kept | 34 | 34 | 0 |
| contrast minimum (light / dark) | 5.78 / 7.81 | 5.78 / 7.81 | 0 |

Better on three KPIs, worse on none, so it is kept. These are the values of the site rebuilt with
the diff (`npm run build`, scripts on, hydrated) and measured in the six environments; stage 2 gave
the same values from the patched sources.

Screenshots of the first screen, current and chosen, in `docs/campaign-2026-11-24/`:
`current-light-1440.png`, `chosen-light-1440.png`, `current-dark-390.png`, `chosen-dark-390.png`.

The diff: `website/product.json` (actions and lede), `website/src/css/landing.css` (new, heading
scale 0.85), `website/docusaurus.config.ts` (loads it as `customCss`), `website/README.md` (the new
file's row).

## Run 1: without the content rule

The same space, five ranked KPIs and no content rule; 0 of 288 variants failed a hard rule. Stage 1
31.2 s (check 12 ms, draw 13 ms per variant and size), stage 2
32.9 s for 31 variants, stage 3 8.8 s.

| stage-2 rank | digest | actions, order, kpis, features, lede, density, headings | words | px | actions | regions | targets | content (not measured then) |
|---|---|---|---|---|---|---|---|---|
| 1 | `sha256:b027c9396cc6` | install and status, status first, hidden, three, short, 0.75, 0.85 | 53 | 433 | 4 | 7 | 18 | 28 of 34 |
| 2 | `sha256:2060766ab866` | install and status, status first, shown, three, short, 0.75, 0.85 | 53 | 433 | 5 | 7 | 18 | 31 of 34 |

Rank 1 was applied at first and is replaced. It hid the KPI tiles and the three "decided" feature
cards (variants from the specification, evaluation without I/O, the studio), and the features
section kept its title "A loop you can run today. An engine on paper." with only loop cards. Its
extra gains over run 2's choice (4 actions instead of 5, 18 small targets instead of 21) came from
the removed cards: fewer links in the body, and the status section one screenful closer at 390x844.

## What the funnel got wrong

1. **Stage 1 ranks the short lede worse.** The terminal renderer cuts a text to one line, so stage
   1 reads 20 words of either lede, and the short lede's first line has one word more. In both runs
   every short-lede variant ranked below its long-lede twin in stage 1 and above it in the browser.
   With the content axes fixed this is most of what is left to rank, so run 2's tau-b fell to 0.059.
   The top-30 cut kept the short-lede variants only because 30 of 72 is a wide cut.
2. **Stage 1 is blind to two axes and mostly to a third.** Density and heading scale draw nothing
   on the terminal; section order moves no KPI in either stage.
3. **The layout hard rules never bound.** The hero is first in every variant and the theme already
   passes contrast in both modes. A space that tests them needs axes that move the primary action (an
   order inside the hero, a hero art) or colour. The content rule bound: it pruned 216 of 288.
4. **Regions crossed did not move** (7 everywhere): at 1440 and 1024 the navbar reaches every
   journey in one or two actions, so the landing body only matters at 390x844, where the routes
   change cost but not the regions they cross.
5. **Run 1's score rewarded deleting content**: best-of-n against proxies with no content
   counterweight, the Goodhart case of the research note § 10. Run 2's content rule closes the case
   for this page; it counts what the current page has, so it does not see content a variant would
   add.
6. **A harness defect, fixed during run 1.** The theme fades colours when the mode switches; a
   measurement inside the fade read a colour between the two modes and reported a false contrast
   failure. Transitions are turned off while measuring.
7. **A harness defect, fixed before run 2.** The host's disk keeper deletes idle Cargo target
   directories, and with it the `uxui-cells` binary; the driver then fell back to a static stage 1
   without saying so. It now exits 2 when `--cells` names a missing file.

Checks of the projection:

- Stage 2 renders a variant by patching the built page in place with the site's scripts removed
  (`landing-page.js` follows the docs-system renderer: actions by `href`, sections by kind, feature
  cards by title, section numbers, an empty `kpis` drops the tiles, feature columns by count). The
  current page measured with its scripts on (hydrated, served from the build) gives the same values
  as stage 2's scripts-off reading: 86, 652, 5, 7, 24, 34.
- Each applied winner rebuilt from the patched sources and measured with its scripts on gives stage
  2's values exactly (run 1: 53, 433, 4, 7, 18; run 2: 53, 472, 5, 7, 21, 34). For these axes the
  projection is the build.

## The studio

`campaign-server.mjs` answers the studio's views (Campaigns, CampaignById, Climb, Frontier,
Variants, VariantById, EvaluationsOfVariant, VerdictsOfVariant, AxisValues, Promotions and the rest)
from the campaign's state, and pushes `CampaignProgressed` and `FrontierChanged` on the climb
channel as each batch of stage 1 and each variant of stage 2 finishes, then `CampaignEnded`. The
generated studio runs against it with `setDataAdapter(httpAdapter(...))` and
`useNetworkChannels(...)`; `StartCampaign` (or `POST /api/run`) runs the campaign again while the
climb page is open. Run 2 streamed into an open climb page. Each variant's
verdicts include `every-base-section-and-card-present` (stage 1 on the sources, stage 2 in the
browser) and its evaluations `content.items_kept`.

Two defects of the generated studio showed on real data: the line chart draws in the middle third of
its box (its SVG has a fixed viewBox and no `preserveAspectRatio="none"`), and it plots rows in the
order they arrived, so a re-run with the page open draws a V; the climb channel has no event for a
campaign that starts, so the summary says "Ended" during a re-run until it is reloaded.

## Run it

```sh
node tools/scaffold/campaign-server.mjs --port 5200 --dir target/campaign \
  --studio <generated studio>/www --site website/build \
  --cells <uxui-cells binary> --product <product.json of the page to improve> --run
```

The live climb is `/campaigns/climb/cp-docs-landing-<date>`, the side-by-side page `/compare`, a
variant rendered from its sources `/uxui/?variant=<digest>&mode=light`. Scripts that serve and
measure run without `slot`; `uxui-cells` builds through it.

## Lessons for the next campaign

- Content needs a hard rule from the first run. Without it the search removed the KPI tiles and
  three feature cards; with it, 216 of 288 variants were never worth measuring, and the axes that
  only remove content should leave the space or carry a declared reason.
- The cell renderer cuts text to one line, which inverts the ranking of the short lede in stage 1
  (tau-b 0.476 in run 1, 0.059 in run 2). Wrapping at the target width is the first fix to the layout
  model; drawing spacing and heading size is the second.
- Ties: the frontier was three copies of one page in three section orders, and the tie band chose a
  variant one step off the frontier with one change fewer. Both are by the rule; the 2 % band is wide
  next to a 39 px difference that only density makes.
