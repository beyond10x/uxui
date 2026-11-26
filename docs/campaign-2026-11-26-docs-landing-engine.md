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
| wall time of `ux-search landing run` | 13.6 s, 4 pages at once, on a build node with 32 CPUs |
| browser | `HeadlessChrome/158.0.8127.18` (the headless shell of Playwright's build 1254) |
| adapter | `ux-browser 0.11.0` |

The wall time covers the 864 renders, the 32 screenshots (each variant measured again before its
screenshot) and the check against a second build of the site; the two site builds are not in it.
`docs/targets.md` reckoned with about 2,600 renders at 1 s each.

The table is the committed recording, the third. The first (adapter `ux-browser 0.10.0`, 14.1 s; a
run before it took 14.8 s and wrote the same measurements, byte for byte) and the second (adapter
`ux-browser 0.11.0`, 13.6 s) measured a page whose eyebrow named the plugin version. The eyebrow is
part of `website/product.json`, so release 0.11.0 changed that source's digest and the campaign was
recorded again for it: the second recording differs from the first in the adapter line and that
digest only.

For the third the version left the eyebrow ("Measured UI improvement"; the status page names the
plugin version). No source of the campaign names a version since, so a release changes neither
digest and records nothing again; the adapter line says which `ux-browser` measured, not which
release the tree is at. Against the second recording `words` is 2 lower in all 864 renders, and
`primary_px` is 14 or 15 px lower in the 288 renders at 390x844, where the eyebrow takes one line
and took two; no other number moved. The frontier is the same 16 variants in the same order and the
page as shipped keeps rank 35. Ranks 49 to 96 changed: `d85.h100` (485, before 500) now stands
before `d115.h85` (495). All 32 screenshots were replaced: at 390x844 the page below the eyebrow
stands 15 px higher.

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
pairs of density and heading scale). `words` is 54 for all of them: no axis changes the text above
the primary action.

| density, heading scale | `primary_px` |
|---|---|
| `d85.h85` | 448 |
| `d100.h85` (shipped) | 472 |
| `d85.h100` | 485 |
| `d115.h85` | 495 |
| `d100.h100` | 501 |
| `d115.h100` | 517 |

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

The page as shipped (`docs-system.left.1+1.features-first.d100.h85`: `hard=0 words=54
primary_px=472 regions=7 small_targets=21 actions=5`) ranks 35th. The `d85.h85` variants with `1+1`
dominate it.

## Frontier

Ranks 1 to 8 have `hard=0 words=54 primary_px=448 regions=6 small_targets=20 actions=6`, ranks 9
to 16 `hard=0 words=54 primary_px=448 regions=7 small_targets=21 actions=5`. Each variant's full
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
At 390x844 the first screen of a `features-first` variant with `1+1` and of its `status-first`
twin is the same picture (4 pairs of files with the same bytes): the sections start below it. With
`1+0` the top edge of the first section is in the last 11 px of the screen, and the 4 pairs differ
there.

## What the numbers do not say

- Three of six axes move no term. The terms measure position, count and reach; they do not see a
  palette that passes contrast, an alignment, or the order of sections no journey's cheapest route
  runs through.
- The numbers are of one browser build on one host. With the eyebrow of the first two recordings, a
  workstation with the full Chromium of the same version (`Chrome/158.0.8127.18`) measured 304 of
  the 864 renders differently: all 288 at 390x844 in `primary_px` (14 or 15 px lower there: the
  hero's eyebrow took one line, two on the build node), and 16 at 1024x768 in the regions of the
  metrics journey (one more there). The frontier and the rank of the page as shipped came out the
  same. That comparison was not run again for the committed recording, whose eyebrow takes one line
  on the build node too. The committed measurements are the build node's.
- A render is the built page changed in place, not a build of the variant. One variant is checked
  against a real build in every run (above); the other 142 rest on the same mapping.
- A screenshot is the first screen, not the whole page.
- Text over a background image is not contrast-checked and not counted as a finding; the landing
  body has none.

## Run it again

One command, from the repository root, with Node 22, npm, cargo and a Chromium (`UXUI_CHROME` names
the binary; else the headless shell of Playwright's build 1254 in the Playwright cache, the build
this campaign is recorded with):

```console
sh tools/docs-landing-campaign.sh
```

On a host with a build pool the same is one job of the pool, started by
`sh tools/docs-landing-campaign.sh --pool`: the job runs on a build node and the files it wrote are
copied back. No other job of the same checkout runs beside it.

The script builds `ux-search` and the site, builds the site once more from the patched sources of
one variant in a copy of `website/` (it writes neither source of the site), runs
`ux-search landing run`, and prints the report. It records in place: the measurements and the
report in `engine/search/tests/fixtures/docs-landing/`, the golden evidence of the tests of
`landing promote` beside them (through the tests' own switch, `UX_SEARCH_BLESS`), and the
screenshots in `docs/campaign-2026-11-26-docs-landing-engine/`, in place of the ones that were
there. No file is moved by hand and no golden file is blessed by hand; the numbers of this document
are written from the report. The run's own files stay in `engine/search/target/landing/run`.

A release does not record the campaign again. Neither source names a version
(`engine/search/tests/landing.rs` holds that), so a release changes neither digest, and the
measurements keep the adapter line they were recorded with: it says which `ux-browser` measured
them, not which release the tree is at.

The report of a measurements file alone, with no browser:

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

## Promotion

`ux-search landing promote` makes one variant of the frontier the page as shipped
(`engine/search/src/landing/promote.rs`). It starts no browser and builds no site. Run it in the
repository's root: the evidence names each file by its path from the working directory.

```console
engine/search/target/debug/ux-search landing promote \
  --measurements engine/search/tests/fixtures/docs-landing/measurements.json \
  --variant <variant id> \
  --product website/product.json --css website/src/css/landing.css \
  --evidence docs/promotions/<date>-docs-landing.evidence.json \
  --chosen-by "<who chose>" \
  --shipped-file engine/search/src/landing/shipped.txt
```

It refuses, with the reason on standard error and exit code 1, and writes nothing:

- when the id is no variant of the space, is the page as shipped, is not in the measurements,
  breaks a hard rule or is not on the frontier of the measurements (the message names a variant
  that dominates it);
- when one of the two files is not the source the measurements were recorded on (the measurements
  hold each file's SHA-256), or when the patch does not apply;
- when a place it is to write is a directory or has no file name, and when two of its six files
  (the measurements, the two sources, the evidence, the copy of the measurements, the shipped file)
  are one file, by whatever spelling: `..`, an absolute path, a link.

A wrong invocation exits with 2. There is no flag that promotes a variant off the frontier.

Otherwise it writes five files, all of them or none:

| file | what |
|---|---|
| `website/product.json`, `website/src/css/landing.css` | the two sources with the variant's patch applied |
| the evidence file | one `ratchet-evidence/1` document, below |
| `<date>-docs-landing.measurements.json` beside the evidence file | a copy of the measurements the choice was made on, byte for byte: the name of the evidence file with `.measurements.json` for `.evidence.json`. The evidence names this copy, because the campaign's own `measurements.json` is replaced when the campaign is recorded again |
| `engine/search/src/landing/shipped.txt` | the variant's id: the one place that says which variant is shipped. `ux-search` reads it when it is built (the report's `shipped` line, the tests), so the next step is a build. Without `--shipped-file` the file stays, and the command says so in one line that names it |

Each text is staged beside its place, and the files take their places only when all are staged.
When a move fails after others were made, the files already in place get back the bytes they had
(one that was not there is removed), and the staged files and the directories made for the
promotion are removed; the message says that nothing is changed, or names what could not be put
back. Two things stay as they are: a place that is a link is replaced by a file, and a rewritten
file has the mode of a new file.

The evidence document has no `run` member and no timestamp: the same measurements, variant,
sources and paths give the same bytes. Promoted evidence files live in `docs/promotions/`, named
`<date>-docs-landing.evidence.json`. The date of the choice is the commit's; `--chosen-by` says who
chose.

| member of `evidence` | holds |
|---|---|
| `candidate`, `target.identifiers.digest` | SHA-256 over the `sha256sum` lines of the two sources after the promotion |
| `inputs` | `measurements`, `product`, `css`: each file's path (`value`) and SHA-256, the measurements as the copy beside the evidence, the sources as they were before the promotion; `variant`: the id; `chosen_by`: who chose it (`--chosen-by`, left out without the flag) |
| `conditions` | `ux-search`, `ux-browser` and Chromium as the measurements name them, the measurements format, the page, the 6 environments |
| `coverage` | the 144 variants, those with `hard` = 0 as `passed` |
| `results` | for the chosen variant and for the page as shipped before: its rank (`landing.rank@1`, with the number of variants and whether it is on the frontier) and its six terms (`landing.<term>@1`); the size of the frontier with its ids in rank order (`landing.frontier_size@1`); per source file its SHA-256 after, with the SHA-256 before and the patch as the report prints it (`landing.source@1`) |

A path is the file's path from the working directory, with `/`, however it was written (`./`, `..`
or an absolute path to a file under it give the same text). A file outside the working directory
has no `value`: its SHA-256 identifies it, as in `ux-static` (`docs/ux-static.md`, Evidence). So
the document holds no absolute path, and run in the repository's root it names the files as the
tree does.

`engine/search/ux-search.assessor.yaml` is the declaration `ratchet-assessor/1` of `ux-search@1`:
the arguments of the command as inputs, the outputs with their scopes (`variant`, `source`; the
frontier's size has the scope `run`) and units, the coverage unit `variant` and the three
conditions. `engine/search/tests/assessor.rs` holds the command line and the evidence documents to
it. It names no `target_input`: the target is the two sources after the patch under one digest,
not one input. The command takes `--evidence` where ratchet passes `--out`, and none of ratchet's
other reserved arguments; it is run by hand.

After a promotion, in this order. What is regenerated: the fixture's two sources, its patched pair
when step 2 says so, the measurements, the report, the screenshots and the golden evidence of the
tests. The tests name no variant a promotion can make the shipped one: the variant they promote is
the first of the recorded frontier that is not the shipped one.

1. `shipped.txt` is compiled in: the next build of `ux-search` reads it (the script of step 3
   builds).
2. Copy the two sources into the fixture (`engine/search/tests/fixtures/docs-landing/product.json`
   and `landing.css`). The rebuilt variant of the campaign script has no axis at its shipped value:
   when the promoted variant shares one with it, name another in `tools/docs-landing-campaign.sh`
   (`variant=`) and in `engine/search/tests/landing.rs` (`FAR`), and write the fixture's
   `variant.product.json` and `variant.landing.css` from its patch. A test names what is missing.
3. Record the campaign again on the new sources with `sh tools/docs-landing-campaign.sh` ("Run it
   again"): the fixture's `measurements.json`, `report.txt`, the golden evidence of the tests
   (`promote.evidence.json`) and the screenshots are replaced together, by the script. The
   evidence file of the promotion and the copy of the measurements beside it stay as written: they
   are the record of the choice.
4. Commit the diff of the sources, the fixture, `shipped.txt` and the screenshots together with the
   evidence file and the copy of the measurements.

One limit. The operations for the actions, the section order and the studio palette are written
against a shipped page with `1+1`, `features-first` and `docs-system`
(`engine/search/src/landing/space.rs`, `ANCHOR`); density, heading scale and hero alignment are
absolute values and carry no such anchor. `promote` applies a variant off the anchored values and
prints a note: before the next campaign the operations of `space.rs` are then rewritten against the
new shipped page, and for a shipped studio palette the probe too, which appends a variant's style
sheet after the built ones and cannot take a shipped palette away.

## Next

The owner picks a variant from the frontier; `ux-search landing promote` applies it as above, and
the diff goes into a pull request with the evidence file (`docs/targets.md`, Promotion).
