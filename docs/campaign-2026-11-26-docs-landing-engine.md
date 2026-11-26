# Campaign 2026-11-26: the documentation landing page, on the engine

Status: run 2026-11-26 (`story:docs-site-first-climb`, phase A). The first campaign of `ux-search`
on the browser adapter: the worked example of `docs/targets.md`, the page `/uxui/` built from
`website/`, 144 variants in 6 environments. It ends at a frontier for a person to choose from;
nothing on the site changed. The spike campaign over the same page is
`docs/campaign-2026-11-24-docs-landing.md`; its probe and its KPI definitions are reused here.

Code: `engine/search/src/landing/` (`space`, `patch`, `record`, `report`, `run`), the in-page probe
`engine/search/src/probes/landing.js`, the script `tools/docs-landing-campaign.sh`. Recorded:
`engine/search/tests/fixtures/docs-landing/measurements.json` (format `ux-search-landing/1`, one
line of 14 numbers per render) and the report over it, `report.txt` beside it.

## The run

| | |
|---|---|
| variants | 144 |
| environments | 6: 1440x900, 1024x768 and 390x844, each light and dark |
| renders measured | 864, in 6 page loads |
| variants that break a hard rule | 0 |
| frontier | 16 variants, 2 different sets of terms |
| the page as shipped | rank 35 of 144: 32 variants before it, 7 equal to it, off the frontier |
| wall time of `ux-search landing run` | 14.1 s, 4 pages at once, on a build node with 32 CPUs; a first run took 14.8 s and wrote the same measurements, byte for byte |
| browser | `HeadlessChrome/158.0.8127.18` (the headless shell of Playwright's build 1254) |
| adapter | `ux-browser 0.10.0` |

The wall time covers the 864 renders, the 32 screenshots (each variant measured again before its
screenshot) and the check against a second build of the site; the two site builds are not in it.
`docs/targets.md` reckoned with about 2,600 renders at 1 s each.

## Space

Six axes, 2 x 2 x 3 x 2 x 3 x 2 = **144 variants** (`engine/search/src/landing/space.rs`, `AXES`:
the one place the axis values stand). A variant's id is its values joined by `.`, in this order.

| axis | values, the shipped one in bold | source |
|---|---|---|
| theme | **`docs-system`**, `studio` | `landing.css`: the colours of `themes/studio.css`, light and dark, on the docs-system colour tokens `--b10x-color-*`, in two rules the patch appends |
| hero | **`left`**, `centred` | `landing.css` `--uxui-hero-align` (`start`, `center`), `--uxui-hero-lede-inline` (`0`, `auto`) |
| actions | `1+3`, **`1+1`**, `1+0` | `product.json` `/product/actions`: the primary action and three, one or no links. `1+3` adds "Run one round" and "The search engine design" before "What exists today" |
| section order | **`features-first`**, `status-first` | `product.json` `/sections`: the status section moved to the front |
| density | `d85`, **`d100`**, `d115` | `landing.css` `--uxui-density` 0.85, 1, 1.15 |
| heading scale | **`h85`**, `h100` | `landing.css` `--uxui-heading-scale` 0.85, 1 |

The owner chose the palette reading of the theme axis on 2026-11-26: the docs-site palette or the
studio palette. `docs/targets.md` listed the axis as light and dark until then and names the two
palettes since; light and dark are the two modes of the environments, there and here.

A variant is a patch to the two sources: RFC 6902 operations on `website/product.json`, each group
opened by a `test` of what it changes, and values of custom properties of
`website/src/css/landing.css`, by selector. The patch of the page as shipped gives both files back
byte for byte. A patch that does not hold is refused whole: an operation that fails leaves the
document as it was, and a custom property the style sheet declares more than once for the selector
(in a second rule, or in one inside `@media`) is refused, not set in one of the places.

| axis value | patch |
|---|---|
| `1+3` | `test /product/actions/1/href "/docs/status"`, `add /product/actions/1`, `add /product/actions/2` |
| `1+0` | `test /product/actions/1/href "/docs/status"`, `remove /product/actions/1` |
| `status-first` | `test /sections/2/kind "status"`, `move /sections/2` to `/sections/0` |
| hero, density, heading scale | the four `--uxui-*` properties of `:root` |
| `studio` | 22 `--b10x-color-*` properties in `html:root:has(.b10x-product)` and in `html[data-theme='dark']:has(.b10x-product)` |

## Hard rules and terms

Hard rules, each checked in every environment: the primary action is fully in view without
scrolling; no horizontal scroll; contrast WCAG 2.x 1.4.3 (4.5:1, 3:1 for large text) of every
visible text against the surface it lies on; an accessible name on every visible interactive
element; every section and card of the built page is shown (its KPI tiles, sections, feature cards,
flow steps, status items and related-tool cards); each of the three
journeys ("install the plugin", "what exists today", "find the metric catalogue") has a route.

Terms, in ranking order, lower is better:

| term | measured as |
|---|---|
| `hard` | hard rules broken, one per rule and environment, summed over the six environments |
| `words` | words of visible text above the primary action |
| `primary_px` | top of the primary action, page px |
| `regions` | regions (navbar, hero, each section, footer) the three journeys cross, summed |
| `small_targets` | visible links and buttons of the landing body with a side under 44 px, at 390x844 |
| `actions` | actions of the three journeys, summed: one per click, one per screenful scrolled, one for the menu toggle, one for a route through a reference page's sidebar (two at a narrow viewport) |

`hard` is summed over the environments: a rule broken in two of them counts 2. Every other term is
the worst value over the environments it is measured in. The ranking is lexicographic over the terms,
ties by variant id. The frontier is the Pareto frontier among the variants with `hard` = 0;
variants with equal terms stand on it together. The same measurements give the same bytes:
`engine/search/tests/landing.rs` holds the report over the committed measurements to `report.txt`.

## How a variant is rendered

One browser, one page per environment. The page is the built landing page with its own scripts off
(`Emulation.setScriptExecutionDisabled`): the static HTML of a first paint. In it the probe renders
each variant from the built page: the hero actions and the section order as the variant's
`product.json` has them, and the variant's `landing.css` after the built style sheets. So the 864
renders take 6 page loads and no build per variant. The page reaches only the server of the built
site, with `GET` and `HEAD`.

The run checks that shortcut, and fails when a check does:

| check | result |
|---|---|
| the probe's render of the variant the sources are equals the built page: same markup of the landing body, same numbers | held in 6 environments |
| the first variant, measured again after the last, gives the same numbers | held in 6 environments |
| a variant's `product.json` differs from the sources only in `/product/actions` and the order of `/sections` (all the probe renders) | held for 144 variants |
| a frontier variant measures, on the freshly loaded page its screenshot is taken of, what it measured in the campaign | held for 16 variants in 2 environments |
| a site built from the patched sources of `studio.centred.1+3.status-first.d85.h100` (no axis at its shipped value) shows the markup and the numbers the probe rendered for that variant | held in 6 environments |

With its scripts off the page runs no callback (no animation frame, no promise of the page), so
the probe lays each render out itself, takes transitions to their end, and a render measured while
a font was loading is measured again.

## Result

No variant breaks a hard rule. Theme, hero alignment and section order move no term, so every set
of terms is shared by 8 variants; the 144 variants have 18 different sets (3 action counts x 6
pairs of density and heading scale). `words` is 56 for all of them: no axis changes the text above
the primary action.

| density, heading scale | `primary_px` |
|---|---|
| `d85.h85` | 448 |
| `d100.h85` (shipped) | 472 |
| `d115.h85` | 495 |
| `d85.h100` | 500 |
| `d100.h100` | 516 |
| `d115.h100` | 532 |

| actions | `regions` | `small_targets` | `actions` |
|---|---|---|---|
| `1+0` | 6 | 20 | 6 |
| `1+1` (shipped) | 7 | 21 | 5 |
| `1+3` | 7 | 23 | 5 |

The frontier is `d85.h85` with `1+0` or `1+1`. Against the page as shipped, `d85` brings the
primary action 24 px up (472 to 448). `1+0` drops the link "What exists today" from the hero: one
tap target under 44 px fewer and one region fewer for that journey, and at 390x844 one action more,
since the route is then the menu toggle and the navbar's "Status". `1+3` is dominated by `1+1`: two
more small tap targets and nothing gained.

The page as shipped (`docs-system.left.1+1.features-first.d100.h85`: `hard=0 words=56
primary_px=472 regions=7 small_targets=21 actions=5`) ranks 35th. The `d85.h85` variants with `1+1`
dominate it.

## Frontier

Ranks 1 to 8 have `hard=0 words=56 primary_px=448 regions=6 small_targets=20 actions=6`, ranks 9
to 16 `hard=0 words=56 primary_px=448 regions=7 small_targets=21 actions=5`. Each variant's full
patch stands under its line in `engine/search/tests/fixtures/docs-landing/report.txt`. The
screenshots are the first screen, 1440x900 light and 390x844 dark, in
`docs/campaign-2026-11-26-docs-landing-engine/`.

| rank | variant | 1440x900 light | 390x844 dark |
|---|---|---|---|
| 1 | `docs-system.centred.1+0.features-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/docs-system.centred.1+0.features-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/docs-system.centred.1+0.features-first.d85.h85.390x844-dark.png) |
| 2 | `docs-system.centred.1+0.status-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/docs-system.centred.1+0.status-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/docs-system.centred.1+0.status-first.d85.h85.390x844-dark.png) |
| 3 | `docs-system.left.1+0.features-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/docs-system.left.1+0.features-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/docs-system.left.1+0.features-first.d85.h85.390x844-dark.png) |
| 4 | `docs-system.left.1+0.status-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/docs-system.left.1+0.status-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/docs-system.left.1+0.status-first.d85.h85.390x844-dark.png) |
| 5 | `studio.centred.1+0.features-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/studio.centred.1+0.features-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/studio.centred.1+0.features-first.d85.h85.390x844-dark.png) |
| 6 | `studio.centred.1+0.status-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/studio.centred.1+0.status-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/studio.centred.1+0.status-first.d85.h85.390x844-dark.png) |
| 7 | `studio.left.1+0.features-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/studio.left.1+0.features-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/studio.left.1+0.features-first.d85.h85.390x844-dark.png) |
| 8 | `studio.left.1+0.status-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/studio.left.1+0.status-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/studio.left.1+0.status-first.d85.h85.390x844-dark.png) |
| 9 | `docs-system.centred.1+1.features-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/docs-system.centred.1+1.features-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/docs-system.centred.1+1.features-first.d85.h85.390x844-dark.png) |
| 10 | `docs-system.centred.1+1.status-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/docs-system.centred.1+1.status-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/docs-system.centred.1+1.status-first.d85.h85.390x844-dark.png) |
| 11 | `docs-system.left.1+1.features-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/docs-system.left.1+1.features-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/docs-system.left.1+1.features-first.d85.h85.390x844-dark.png) |
| 12 | `docs-system.left.1+1.status-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/docs-system.left.1+1.status-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/docs-system.left.1+1.status-first.d85.h85.390x844-dark.png) |
| 13 | `studio.centred.1+1.features-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/studio.centred.1+1.features-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/studio.centred.1+1.features-first.d85.h85.390x844-dark.png) |
| 14 | `studio.centred.1+1.status-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/studio.centred.1+1.status-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/studio.centred.1+1.status-first.d85.h85.390x844-dark.png) |
| 15 | `studio.left.1+1.features-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/studio.left.1+1.features-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/studio.left.1+1.features-first.d85.h85.390x844-dark.png) |
| 16 | `studio.left.1+1.status-first.d85.h85` | [png](campaign-2026-11-26-docs-landing-engine/studio.left.1+1.status-first.d85.h85.1440x900-light.png) | [png](campaign-2026-11-26-docs-landing-engine/studio.left.1+1.status-first.d85.h85.390x844-dark.png) |

Within a rank group the terms are equal: the order is the variant id's, not a preference. The
choice between palette, hero alignment and section order is the person's, from the screenshots.
At 390x844 the first screen of a `features-first` variant and of its `status-first` twin is the
same picture (8 pairs of files with the same bytes): the sections start below it.

## What the numbers do not say

- Three of six axes move no term. The terms measure position, count and reach; they do not see a
  palette that passes contrast, an alignment, or the order of sections no journey's cheapest route
  runs through.
- The numbers are of one browser build on one host. On a workstation with the full Chromium of the
  same version (`Chrome/158.0.8127.18`) 304 of the 864 renders differ: all 288 at 390x844 in
  `primary_px` (14 or 15 px lower there: the hero's eyebrow takes one line, two on the build node),
  and 16 at 1024x768 in the regions of the metrics journey (one more there). The frontier and the
  rank of the page as shipped come out the same. The committed measurements are the build node's.
- A render is the built page changed in place, not a build of the variant. One variant is checked
  against a real build in every run (above); the other 142 rest on the same mapping.
- A screenshot is the first screen, not the whole page.
- Text over a background image is not contrast-checked and not counted as a finding; the landing
  body has none.

## Run it again

From the repository root, with Node 22, npm and a Chromium (`UXUI_CHROME` names the binary; else
the newest `chromium-<n>` of the Playwright cache):

```console
cargo build --manifest-path engine/search/Cargo.toml --locked
sh tools/docs-landing-campaign.sh docs/campaign-2026-11-26-docs-landing-engine > report.txt
```

The script builds the site, builds it once more from the patched sources of one variant, runs
`ux-search landing run`, and prints the report. It writes `measurements.json` and the screenshots
to the directory named; the measurements kept in the repository are that file, moved to
`engine/search/tests/fixtures/docs-landing/`. The report of a measurements file alone, with no
browser:

```console
engine/search/target/debug/ux-search landing report \
  --measurements engine/search/tests/fixtures/docs-landing/measurements.json
```

The fixture keeps the two sources as this campaign measured them, and the measurements name their
digests. After a change of `website/product.json` or `website/src/css/landing.css` the script
refuses to run until the fixture holds the new sources (`product.json`, `landing.css`, and the
patched pair `variant.product.json`, `variant.landing.css`, which a test compares with what the
patch writes); a new campaign then replaces the measurements, the report and the screenshots
together.

## Next

The owner picks a variant from the frontier. Phase B applies its patch to `website/product.json`
and `website/src/css/landing.css` as a pull request with ratchet evidence (`docs/targets.md`,
Promotion).
