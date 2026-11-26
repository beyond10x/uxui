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

- **Declaration**: the page's path and the selector that says it is ready; the environments (viewports, and
  each mode set by media emulation, by an attribute of the root element or by a query); the axes, each value a
  list of custom-property values per CSS selector, with the shipped value named; the primary action's
  selector; the hard rules and the ordered terms, by name from the engine's catalogue. An unknown key, an
  unknown rule or term, an empty axis, two values of one name and, at run time, a selector that matches
  nothing are refused with the file and the key.
- **Run**: `ux-search campaign run --declaration <file> --url <origin> --out <measurements.json> [--shots
  <dir>]` renders every variant in every environment as an injected style sheet and writes the measurements
  with the declaration inside and a digest over both. `ux-search campaign report --measurements <file>` ranks
  them with the landing campaign's ranking code and prints the frontier, each frontier variant's patch and
  the shipped variant's rank; it starts no browser.
- **What the other repository adds**: the declaration, and a command that builds and serves its app. Nothing
  of it is compiled into the engine, and the engine writes nothing into it but the files the command names.
- **Limits**: an axis is custom properties, so a campaign cannot reorder or remove anything; the page must
  ask for nothing but its own origin with `GET` and `HEAD`; there is no promotion step, the report's patch is
  what a person applies.

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
