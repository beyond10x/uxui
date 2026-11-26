# Targets that are not an ess-ui/1 document

Status: design, 2026-11-24. The owner asked how the studio's live climb would improve a UI other than the
studio, for example this repository's documentation site. The specification `uxsearch` v1 cannot: it assumes
every target is an `ess-ui/1` document evaluated on a layout model with no I/O. The documentation site is a
Docusaurus site whose landing page is generated from `website/product.json` (`b10x-product-landing/1`) and
styled by CSS. This is a design gap; this document closes it.

## The change: a target adapter

The engine needs three things from a target, and only these:

| capability | what it does |
|---|---|
| materialize | turn a variant (base + a value per axis) into something renderable: an `ess-ui/1` document, or a patched `product.json` plus a CSS file |
| render | for (artifact, UI state, environment): geometry of the declared nodes and an accessibility tree |
| enumerate | the UI states of each journey: reachable states for `ess-ui/1`, scripted steps (URLs, clicks) for a site |

Two adapters:

- **`ess-ui`**: the specification as it is. Exact, no I/O, layout model v0 (cells), thousands of variants a
  second if the cost spike (roadmap step 3) holds. States enumerated from the model.
- **`browser`**: any page with a URL. Materialize writes the variant's files and builds or serves them; render
  is a headless browser at each viewport and mode (boxes, computed styles, accessibility tree, a screenshot);
  states are the journey's scripted steps. Real pixels, seconds per evaluation, so campaigns use pruned or
  guided strategies and a smaller space. This is what the original loop measured with, without the search.

Everything above the adapter stays as specified: axes, hard and soft rules, metrics, KPIs, the frontier, the
climb events, the studio, promotion and the ratchet evidence. Evidence records name the adapter and its
version, so a value measured in a browser is never compared with one from the cell model.

## The funnel: lift, brute force, project, judge

The owner, 2026-11-24: "take any page, use an LLM to kind of identify the component, and then use some sort of
intermediate ESS UI spec for just that component [...] running the state space exhaustion without rendering
and LLM judgment [...] this brute force technique which needs to be executed in a very fast way. And then, of
course, there will be the more expensive checks at the end, which involves screenshots, LLMs, judgment, maybe
human checks."

| stage | input → output | cost per variant | who |
|---|---|---|---|
| 0 lift | a page → per component an intermediate `ess-ui/1` document, fixtures with the page's real content, and a binding table (node path ↔ CSS selector) | once per component, minutes | a model proposes, `ess ui check` and a render-and-compare against the page verify |
| 1 brute force | the lifted document → every variant in the design space, exhaustive over the UI state space | microseconds to milliseconds (to be measured, roadmap step 3) | the engine, `ess-ui` adapter; no browser, no model |
| 2 project | the top k of stage 1 → applied to the real page through the binding table, measured in a browser | about a second | the engine, `browser` adapter |
| 3 judge | the top few of stage 2 → screenshots side by side, a model's judgement, a person's choice | minutes, money | people and review agents; judgement records, never a gate |

Each stage cuts the candidates by one to two orders of magnitude, so the expensive checks only ever see a
handful. The binding table is what makes "when the scaffold moves, the page moves": a change to the lifted
document is a change to known nodes of the real page.

Two checks keep the funnel honest:

- **Lift fidelity**: the lifted document, rendered, must reproduce the component's structure (nodes, order,
  actions, names) as the page shows it; a lift that does not is refused, not searched.
- **Stage agreement**: stage 1's ranking of the top variants is compared with stage 2's on every campaign. A
  low agreement means the cell model misleads on this target, and the campaign says so in its evidence.

## Worked example: the documentation landing page

- **Target**: page `/uxui/` (scope page), and the hero section (scope section).
- **Axes**: theme (the palette: the documentation site's, the studio's; light and dark are modes of the
  environments), hero layout (text left, centred), number of actions in the hero (1 primary + 3 links, 1 + 1,
  1 + 0), section order (features before status, status first), density (spacing scale 0.85, 1, 1.15),
  heading scale (2 values). 2 x 2 x 3 x 2 x 3 x 2 = 144 variants.
- **Environments**: 1440x900, 1024x768, 390x844; light and dark mode.
- **Journeys**: "install the plugin" (land, find the install command, copy it), "what exists today" (land,
  reach the status), "find the metric catalogue".
- **Hard rules**: primary action fully in view without scrolling at every viewport; no horizontal scroll;
  contrast at least 4.5:1 for body text in both modes; every interactive element has an accessible name.
- **Metrics and KPIs**: actions per journey, words before the first action, distance in px from the top to the
  primary action, number of regions a journey crosses, tap targets under 44 px. (The original loop's data: visible words per region and regions per task separated designs where 17 of 18 catalogue metrics did not.)
- **Run**: 144 variants x 6 environments x 3 journeys on the browser adapter, about 2,600 renders; at 1 s each,
  about 45 minutes exhaustive, or a beam search in minutes. The studio shows the climb live, the frontier, and
  side-by-side screenshots for the person to judge.
- **Promotion**: the chosen variant becomes a diff to `website/product.json` and the theme CSS, a pull request,
  and ratchet evidence; the site's next build is gated on the promoted values.

## A target outside this repository: the declared campaign

The landing campaign's page, axes and terms are compiled into `ux-search`. A page of another repository is
climbed without a change to the engine: the target is a declaration file, format `ux-search-campaign/1`, and
the server is whatever serves that repository's app on a loopback port.

- **Declaration**: the page's path and the selector that says it is ready; where the pointer rests
  (`page.pointer`); the environments (viewports, and each mode set by media emulation, by an attribute of the
  root element or by a query, with what the page shows in the mode under `expect`); the axes, each value a
  list of custom-property values per CSS selector, with the shipped value named; the primary action's
  selector; the hard rules and the ordered terms, by name from the engine's catalogue. An unknown key, an
  unknown rule or term, an empty axis, two values of one name, a pointer outside a viewport and, at run time,
  a selector that matches nothing are refused with the file and the key. Every key with an example stands at
  the head of `engine/search/src/campaign/declaration.rs`.
- **Run**: `ux-search campaign run --declaration <file> --url <origin> --out <measurements.json> [--shots
  <dir>] [--jobs <n>] [--profile <dir>]` renders every variant in every environment as an injected style
  sheet and writes the measurements with the declaration inside and a digest over both. `--jobs` is the
  number of pages measured at once (default: the CPUs, at most 4). `--profile` is the directory of the
  browser's profile; without it the run makes one of its own under the system's temp directory (`TMPDIR`) and
  removes it when it ends, so the working directory gets nothing but the files the command names.
  `ux-search campaign report --measurements <file>` ranks them with the landing campaign's ranking code and
  prints the frontier, each frontier variant's patch and the shipped variant's rank; it starts no browser.
  With `--notes` it prints, after the same report, the first elements behind the counts `low_contrast`,
  `overflow`, `small_targets` and `always_visible_misses` (at most 3 per count and environment) of the page as
  shipped and of each frontier variant: a count says how many, the note says which, so a person looks at the
  element before taking the number. `--help` after any command prints the usage.
- **Screenshots**: with `--shots`, each frontier variant is shot in every mode at the first viewport and in
  the last environment (the last viewport in the last mode). With two viewports and two modes that is three
  of the four environments. A variant off the frontier gets no screenshot. A mode the page did not take is
  seen in its screenshot.
- **What the run says beside the report** (standard error): a mode that is set by a query and expects
  nothing is named as not read back; per axis, how many of its values change the page at all, and an axis
  none of whose values does is named as one that changes nothing here. A value changes the page when a
  render of it differs from the render of the shipped value in a measured number, in the box of an element
  (rounded to px) or in one of these computed styles of an element or of its `::before` and `::after`: the
  colour, the background colour and image, the font size and weight, the border radius, the border and
  outline colours, the box shadow, `fill` and `stroke`. What is not looked at: `opacity`, `transform`, and
  everything else. An axis that changes nothing is not refused: another page may read its property. The
  number per axis stands in the measurements as well (`axes`).
- **What the other repository adds**: the declaration, and a command that builds and serves its app. Nothing
  of it is compiled into the engine, and the engine writes nothing into it but the files the command names.
- **Limits**: an axis is custom properties, so a campaign cannot reorder or remove anything; the page must
  ask for nothing but its own origin with `GET` and `HEAD`; there is no promotion step, the report's patch is
  what a person applies.

### The state of the browser while it measures

A number is of the page in one state. The run sets that state the same way for every render:

- **Pointer**: before every measurement and every screenshot the pointer is moved, as a mouse moves, to
  `page.pointer` (CSS px from the top-left corner of the viewport), or without that key to the bottom-right
  corner of the viewport. A headless browser's pointer that was never moved rests at 0, 0: a navigation in
  that corner that opens on hover would be measured open, and everything under it lower than a visitor sees
  it. `measurements.json` holds the position per environment (`pointer`). A page whose chrome is driven by
  hover names a pointer position over neutral ground; to measure the hovered state, name a position on the
  element.
- **Focus**: the run focuses nothing and emulates no focus. What the page focuses itself when it loads stays
  focused.
- **Motion**: `prefers-reduced-motion: reduce` is emulated, and after a style sheet is injected every running
  animation and transition is taken to its end before the page is measured.
- **Mode**: `media` emulates `prefers-color-scheme`; a mode without `media` is rendered with the scheme
  `light`. `attribute` is set on the root element before the page's scripts run and again when the page is
  ready, and must still be there at every measurement; a page that sets the attribute itself is refused.
  `query` is the query of the page's URL, and what it does is up to the page. `expect` is the read-back: an
  attribute of the root element with its value, a selector that matches at least one element, or both, read
  in every render of the mode. A miss is refused with the mode, the environment and what was read. A mode by
  `query` without `expect` is measured, and the run says that nothing read it back: a page that does not know
  the query is then measured in its default mode twice.
- **In force**: a property a variant sets is in force when the computed value of that custom property, on
  every element the rule's selector matches, is the declared value. It says the page's cascade did not
  override it. It does not say that any rule of the page reads the property: that is what the run's line per
  axis says.

### A private target

`measurements.json` holds the declaration's whole text (the page's path, its selectors), and the report
prints the page's path in its first line. For a target that is private, both files and the screenshots are
as private as the target: they stay in its repository or outside any repository, and no document of this
repository names its page, its selectors or its product.

### Checklist: a target outside this repository

1. A build of the app with fixture data and no backend: the page asks for nothing but its own origin, with
   `GET` and `HEAD`. A request the run's guard fails ends the run.
2. A static server with a single-page fallback, so a deep link answers 200 with the app's `index.html`. A
   server without it answers 404 on the page's path, and the run stops there.
3. A free loopback port: start the server on port 0, read the port it took from its log, and end the server
   by its PID when the run is over.
4. Look at the page and its data in a browser before the run: the fixture data shows what a visitor sees
   (rows in the list, no empty or error state), the primary action's selector matches the action meant, and
   the ready selector matches only when the data is drawn.
5. Hover-driven chrome: when a navigation or a toolbar opens under the pointer, name `page.pointer` over a
   place that opens nothing, and check the first screenshot.
6. A mode by query or by attribute gets an `expect`, and the screenshots of both modes are looked at.
7. Read the run's lines per axis: an axis that changes nothing on this page is dropped from the declaration
   or pointed at a property the page reads.
8. Run `campaign report --notes` and look at the named elements before a count is taken as a finding.

The proof on a page of this repository is the studio's frontier page, generated by ESS and served on its
fixtures: `docs/campaign-2026-11-26-studio-engine.md`, recorded by `tools/studio-campaign.sh`.

## What changes in the specification

- `design`: a `Target` names its adapter (`ess-ui` or `browser`) and, for `browser`, its base URL and build
  command; an axis value is a patch to a named source file (JSON pointer or CSS variable), not only an
  `ess-ui/1` mutation.
- `journeys`: a journey has either a model (enumerated states) or scripted steps.
- `evaluation`: the evaluation key includes the adapter and its version; the layout model is one adapter's
  renderer.
- `promotion`: the diff is per source file.

These land in the specification after the current hardening pass, as specification round 3.

## Order

The documentation site becomes the first non-studio target: it is ours, public, small, and has a declared
source (`product.json`). The browser adapter is built before the cell model is extended, because it reuses
the original loop's proven measurements and gives the comparison bench (`docs/comparison.md`) a referee.
