# Campaign 2026-11-26: the studio's frontier page, on a declared campaign

Status: run 2026-11-26 (`story:trial-target`). The first campaign of `ux-search` whose target is
not compiled in: the page, the environments, the axes and the terms come from a declaration file,
and the page is one of a React app that ESS generates from an `ess-ui/1` document and serves on
its fixtures, with its scripts on. Here the app is this repository's own studio
(`ess/ui/studio.ui.yaml`, `themes/studio.css`); the same command takes the page of another
repository. The campaign ends at a frontier for a person to choose from; nothing in the theme
changed.

Code: `engine/search/src/campaign/` (`declaration`, `run`, `record`, `report`), the in-page probe
`engine/search/src/probes/campaign.js`, the script `tools/studio-campaign.sh`. Recorded in
`engine/search/tests/fixtures/campaign/studio/`: the declaration `frontier.campaign.yaml` (format
`ux-search-campaign/1`), `measurements.json` (format `ux-search-campaign-measurements/2`, one line
of 11 numbers per render, where the pointer rested in each environment and how many values of
each axis change the page at all) and the report over it, `report.txt`.

The two tables of "The run" and "Frontier" and the table of "Made with" are written by the script
with every recording; the text around them says what the numbers are, not what they were on one
day.

## The run

<!-- recorded: begin -->
| | |
|---|---|
| page | `/campaigns/frontier/cp-01` |
| variants | 48: spacing 4 x status-ink 3 x status-fill 2 x nav-head 2 |
| environments | 4: 1366x768 light, 1366x768 dark, 1280x720 light, 1280x720 dark |
| renders measured | 192 |
| variants that break a hard rule | 0 |
| frontier | variants 12, sets of terms 1 |
| the page as shipped | `space-stock.ink-stock.fill-stock.head-stock`: rank 36 of 48, 24 variants before it on their terms, 11 equal to it, off the frontier |
| browser | `HeadlessChrome/159.0.8154.6` |
| adapter | `ux-browser 0.11.0` |
| declaration | `sha256:867e6446b66029d69d0a517e074207cd04ebed38f5bd33e9a1e25ef6bb171606` |

| rank | variant | terms | 1366x768 light | 1366x768 dark | 1280x720 dark |
|---|---|---|---|---|---|
| 1 | `space-tight.ink-deep.fill-pale.head-light` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-pale.head-light.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-pale.head-light.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-pale.head-light.1280x720-dark.png) |
| 2 | `space-tight.ink-deep.fill-pale.head-stock` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-pale.head-stock.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-pale.head-stock.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-pale.head-stock.1280x720-dark.png) |
| 3 | `space-tight.ink-deep.fill-stock.head-light` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-stock.head-light.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-stock.head-light.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-stock.head-light.1280x720-dark.png) |
| 4 | `space-tight.ink-deep.fill-stock.head-stock` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-stock.head-stock.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-stock.head-stock.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-deep.fill-stock.head-stock.1280x720-dark.png) |
| 5 | `space-tight.ink-mid.fill-pale.head-light` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-pale.head-light.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-pale.head-light.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-pale.head-light.1280x720-dark.png) |
| 6 | `space-tight.ink-mid.fill-pale.head-stock` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-pale.head-stock.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-pale.head-stock.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-pale.head-stock.1280x720-dark.png) |
| 7 | `space-tight.ink-mid.fill-stock.head-light` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-stock.head-light.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-stock.head-light.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-stock.head-light.1280x720-dark.png) |
| 8 | `space-tight.ink-mid.fill-stock.head-stock` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-stock.head-stock.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-stock.head-stock.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-mid.fill-stock.head-stock.1280x720-dark.png) |
| 9 | `space-tight.ink-stock.fill-pale.head-light` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-pale.head-light.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-pale.head-light.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-pale.head-light.1280x720-dark.png) |
| 10 | `space-tight.ink-stock.fill-pale.head-stock` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-pale.head-stock.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-pale.head-stock.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-pale.head-stock.1280x720-dark.png) |
| 11 | `space-tight.ink-stock.fill-stock.head-light` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-stock.head-light.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-stock.head-light.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-stock.head-light.1280x720-dark.png) |
| 12 | `space-tight.ink-stock.fill-stock.head-stock` | `hard=0 low_contrast=0 overflow=0 always_visible_misses=0 small_targets=0 primary_px=318 text_under_14=0.000 regions=4` | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-stock.head-stock.1366x768-light.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-stock.head-stock.1366x768-dark.png) | [png](campaign-2026-11-26-studio-engine/space-tight.ink-stock.fill-stock.head-stock.1280x720-dark.png) |
<!-- recorded: end -->

The first table is the run, the second its frontier: each variant with its terms and its
screenshots, the first screen in each mode at the first viewport and in the last environment.
Each frontier variant's patch stands under its line in `report.txt`, and the full ranking follows
there.

## The page

`/campaigns/frontier/cp-01`: the frontier of one campaign, with the campaign's summary above a
table of variants and four actions per row. It is the page a person chooses a variant on, and the
loop round on the studio (`ess/ui/rounds/R-01/`) found on it what the terms below count: text
under 14 px, two status texts under 4.5:1, four buttons of equal weight per row. It shows no clock
and no date, and it does not scroll at either viewport, so the hard rules can hold.

The page is ready when no section shows its loading status any more
(`main.ui-page:not(:has([role=status][aria-label=Loading]))`), and is measured 750 ms later, when
the fixture channel has gone from connecting to live.

## Space

Four axes, 4 x 3 x 2 x 2 = **48 variants** (`frontier.campaign.yaml`, `axes`: the one place the
values stand). A variant's id is its values joined by `.`, in this order. Every property is one
`themes/studio.css` declares in `:root` (a test holds the declaration to that), and the shipped
value of every axis sets nothing: the page as shipped is the page the theme gives.

| axis | values, the shipped one in bold | sets |
|---|---|---|
| spacing | `space-tight`, `space-snug`, **`space-stock`**, `space-roomy` | `--ui-space-md` of `:root`: 0.8rem, 0.95rem, the theme's, 1.3rem. The gap of the page's stack and the padding of a section |
| status-ink | **`ink-stock`**, `ink-mid`, `ink-deep` | `--ui-color-success` and `--ui-color-info` of `:root`, in light mode: the ink of the live indicator and of the badge of a running campaign. In dark mode every value keeps the theme's inks |
| status-fill | **`fill-stock`**, `fill-pale` | `--ui-color-success_fill` and `--ui-color-info_fill`: the fill behind those inks, nearer to the surface. In light mode of `:root`, in dark mode of `:root:not([data-uxui-mode="light"])`, the selector the theme sets its dark values under |
| nav-head | **`head-stock`**, `head-light` | `--uxui-nav-head`, in dark mode: the ink of the group headings of the navigation. In light mode every value keeps the theme's ink |

A variant is a style sheet of custom properties, by selector, injected after the page's own
sheets. A rule may be for one mode: the theme's dark values stand in a rule that outweighs
`:root`, so a dark value has to be set under that rule's selector. The declaration refuses a
property of a selector that two rules of one variant would set in the same mode.

## Hard rules and terms

The declaration chooses its hard rules and its terms by name from the catalogue the probe measures
(`engine/search/src/campaign/declaration.rs`, `HARD_RULES` and `TERMS`).

Hard rules of this campaign, each checked in every environment: the primary action (the first
row's "Propose promotion") is fully in view without scrolling, in the viewport and in every box
around it that clips or scrolls; no horizontal scroll of the page; an accessible name on every
visible interactive element. `contrast` is in the catalogue and is not a hard rule here: as a
hard rule it says broken or not, per environment. As the first term, `low_contrast`, it orders
the variants by how many texts each leaves under the threshold.

Terms, in this campaign's ranking order, lower is better:

| term | measured as |
|---|---|
| `hard` | hard rules broken, one per rule and environment, summed over the four environments |
| `low_contrast` | visible text runs under WCAG 2.x 1.4.3 (4.5:1; 3:1 from 24 px, or 18.66 px bold) against the surface they are composited on. A run is a text node; a run in a disabled control or over a background image is not judged |
| `overflow` | visible text runs and replaced elements (images, form controls, buttons) that end to the right of the viewport or of a box around them that clips or scrolls |
| `always_visible_misses` | declared items (here the navigation and the view switch) that are not fully in view at the top of the page, or not at its end |
| `small_targets` | visible links and buttons with a side under `small_target_px`, here 24 px (WCAG 2.2 2.5.8) |
| `primary_px` | top of the primary action, page px |
| `text_under_14` | the share of the visible text runs under 14 px, in thousandths rounded down, written `0.564` |
| `regions` | declared regions (here the navigation, the page header and each section) that start at or above the primary action |

`hard` is summed over the environments: a rule broken in two of them counts 2. Every other term is
the worst value over the environments. The ranking is lexicographic over the terms, ties by variant
id; the frontier is the Pareto frontier among the variants with `hard` = 0, and variants with equal
terms stand on it together. It is the ranking code of the landing campaign
(`engine/search/src/rank.rs`). The same measurements give the same bytes:
`engine/search/tests/campaign.rs` holds the report over the committed measurements to `report.txt`.

## How a variant is rendered

One browser, one page per environment. The page is loaded from the server the script started, with
its own scripts on, in a browser that emulates the environment's viewport and colour scheme and
`prefers-reduced-motion`. Once it is ready the probe injects each variant's style sheet in turn and
measures: the renders of one environment take one page load and no build per variant.

The run checks that shortcut, and fails when a check does:

| check | what it catches |
|---|---|
| the page is measured with no sheet injected, and the variant of the shipped values, measured with its sheet, gives the same numbers | a probe that changes the page by measuring it |
| the first variant, measured again after the last, gives the same numbers | a render that leaves a trace in the next; a page that changes while it is measured (a timer, an event of the fixture channel) |
| every property a sheet sets is read back from every element its selector matches | a selector that matches nothing; a value another rule of the page outweighs. The refusal names the declaration file and the rule's key, as `axes[2].values[1].rules[1]` |
| each frontier variant is measured once more on a page loaded anew, in the environments of its screenshots, and its screenshot is of that page | numbers that depend on the variants measured before |
| the page asks for nothing but its own origin, with `GET` and `HEAD` | a page that was denied a request: the run fails and names the URL |
| the pointer is moved to the bottom-right corner of the viewport before every measurement and every screenshot (this declaration names no `page.pointer`) | a page measured with whatever opens under a headless pointer at 0, 0 |

A change of the sheet may start transitions; the probe takes each to its end before it measures,
waits for the fonts and for two frames, and scrolls to the top. The generated app compiles its
fixtures into its bundle (the default data adapter reads them in memory), so it asks the server
for its files and nothing else. An app that fetches fixtures from another origin, or writes with
`POST`, is not measured by this command as it is: it is served so that it does not.

The script runs the campaign twice and fails unless the second run writes the bytes of the first.

## Made with

<!-- versions: begin -->
| | |
|---|---|
| `ess/ui/studio.ui.yaml` | `sha256:e7fffe33804e6266835294a7150122f04fe809b3004f8b4c6e9cb42ee4db1404` |
| `themes/studio.css` | `sha256:9e7591d6b161b463c57aedac96598b49cad0bc3d694b3c011c3cdc6a7baaa80e` |
| the served app, `www/` | `sha256:e3ef71ca68535e01fb5b63e1bd37d64843e532315acb2c67d042e7cdd557a4d5` |
| generator | `ess 0.99.2` |
| node, npm | `v22.23.3`, `10.9.9` |
| react, react-dom, esbuild, typescript | `19.3.1`, `19.3.1`, `0.28.2`, `5.9.3` |
| Chromium | `Google Chrome for Testing 159.0.8154.6` |
| engine | `ux-search 0.11.0` |
| first run | 192 renders in HeadlessChrome/159.0.8154.6, 25.1 s with 4 pages at once |
| second run | 192 renders in HeadlessChrome/159.0.8154.6, 25.2 s with 4 pages at once; the same bytes of `measurements.json` as the first run, 36 of 36 screenshots the same bytes |
<!-- versions: end -->

## Reading the frontier

Within a group of equal terms the order is the variant id's, not a preference: the choice between
them is the person's, from the screenshots. An axis whose values all stand on the frontier with
the same terms moves no term in this campaign; the report's line `ranking <n>, sets of terms <m>`
says how many different sets the space has.

What each axis can move on this page: spacing moves `primary_px`, and `overflow` when the table no
longer fits its frame; the two status axes and the heading's ink move `low_contrast`, each in the
mode its rules are for, on a theme that leaves a text under the threshold. `low_contrast` is the
worst of the environments, so an ink that mends the light mode shows in the term only together with
one that mends the dark mode, and a fill and an ink that each fall short alone can pass together:
the axes are not independent, which is what a campaign over their product is for.

What each axis does on the theme as recorded, by the run's line per axis (`axes` in the
measurements): all 3 values of the spacing beside the shipped one, both of the status ink and the
one of the status fill change a box or a computed style of the page, and only the spacing moves a
term. The axis `nav-head` changes nothing: its one value sets the colour the theme has had for the
heading since loop round R-02 (`--uxui-nav-head`), so its two values are one page.

Nothing a variant of this declaration sets moves
`text_under_14`, `small_targets` or `regions`: no axis sets the size of the small text. Since loop
round R-02 the theme declares that size in `:root` (`--uxui-text-small`), so a later declaration
can make it an axis.

Which text runs a variant leaves under the threshold is in the measurements for the page as
shipped and for the frontier: the first three per environment, each with its element, the two
colours and the ratio, and the same for `overflow`, `small_targets` and `always_visible_misses`.
`ux-search campaign report --measurements <measurements.json> --notes` prints them after the
report. For every other variant the measurements hold the count only.

## Three recordings, three themes

The campaign was recorded three times on 2026-11-26, on three states of `themes/studio.css`, with
one declaration (the digest in "The run").

| | the theme before loop round R-02 | after R-02 | after R-03 (the tables above) |
|---|---|---|---|
| `low_contrast`, the page as shipped | 3 | 0 | 0 |
| `text_under_14`, the page as shipped | 0.400 | 0.000 | 0.000 |
| `primary_px`, the page as shipped | 321 | 338 | 340 |
| `primary_px`, the four spacings | not kept | 315, 327, 338, 353 | 318, 329, 340, 356 |
| sets of terms among the 48 variants | 12 | 4 | 4 |
| frontier | 3 variants | 12 variants | 12 variants |
| the page as shipped | rank 42 of 48 | rank 36 of 48 | rank 36 of 48 |

Round R-02 (`ess/ui/rounds/R-02/`, finding UXF-017) set all small text to 14 px and gave each toned
badge a text colour per mode. On that theme no variant leaves a text under the threshold, so the
three ink and fill axes move no term: the 12 variants of the frontier are the 12 with the tight
spacing, and only `primary_px` separates the space. The first recording is in the commit that added
this document; its frontier was three variants that mended the contrast the round then mended in
the theme.

Round R-03 (`ess/ui/rounds/R-03/`) keeps the header of a page in view and takes the outer padding
from the first and last cell of a table row; it gave the page header a least height. The primary
action stands 2 to 3 px lower for it, and the ranking is the one of the second recording. The third
recording is the first with the engine that parks the pointer, reads back what a mode expects and
says what each axis changes; between the second and the third engine on one theme no number moved.

A next campaign on this page needs axes the theme has not settled: on this theme three of these
four move no term, and one of them changes nothing at all.

## What the numbers do not say

- The axes are custom properties only. A variant cannot reorder, remove or relabel anything: the
  four equal buttons per row stay four equal buttons, whatever the frontier is.
- The terms count and measure position. They do not see whether a pale fill still reads as a
  status colour, or whether a tighter page is easier to scan; that is what the screenshots are
  for.
- A term is the worst value of the four environments. A variant that is better in one mode and
  equal in the other has the terms of a variant that is equal in both; the lines of
  `measurements.json` keep the environments apart.
- `low_contrast` judges a text against the colour under it. A text over a background image is not
  judged and not counted: the column heads of the table lie over the gradients of the table's
  scroll frame.
- `overflow` looks to the right only, and takes the boxes around an element as what clips it; an
  element positioned out of its box is judged against that box all the same.
- `always_visible_misses` scrolls the document, not a scrolling box inside it.
- The numbers are of one browser build on one host, with the fonts that host has; the theme names
  Inter first and falls back to the system's font. Another host may measure other px.
- A render is the page with a sheet injected, not a build of the theme with the values written
  in. The check that reads every property back is what stands for that.
- A screenshot is the first screen, not the whole page. The screenshots are not held to bytes:
  the script counts how many of them its two runs wrote the same ("Made with") and names the
  others. The one such pair that was compared pixel by pixel differed in six pixels at the
  rounded corners of the view switch, by one level of a channel.
- One page of the app. A value that helps here may hurt elsewhere: promoting a value into the theme is
  a change of every page, and a round of the loop measures them all (`ess/ui/rounds/round.sh`).

## Run it again

From the repository root, with `ess`, Node 22, npm with a registry, cargo and a Chromium
(`UXUI_CHROME` names the binary; else the newest headless shell of the Playwright cache):

```console
sh tools/studio-campaign.sh
```

It is the one step after a change of `ess/ui/studio.ui.yaml`, its fixtures, `themes/studio.css`,
the declaration or the engine: it replaces `measurements.json`, `report.txt`, the screenshots in
`docs/campaign-2026-11-26-studio-engine/` and the three tables of this document together, and the
tests of `engine/search/tests/campaign.rs` hold the four to each other. On a host with a build
pool the one step is `sh tools/studio-campaign.sh --pool`: it runs the script as one job of the
pool, which copies the written files back, and removes the screenshots of variants that left the
frontier.

A campaign on another page needs a declaration and a server:

```console
engine/search/target/debug/ux-search campaign run --declaration <campaign.yaml> \
  --url http://127.0.0.1:<port> --out <measurements.json> --shots <dir>
engine/search/target/debug/ux-search campaign report --measurements <measurements.json>
```

The declaration's keys are in the head of `engine/search/src/campaign/declaration.rs`;
`engine/search/tests/fixtures/campaign/small.campaign.yaml` uses every one of them. `campaign
report` starts no browser; it refuses a measurements file whose content is not what its digest
names.
