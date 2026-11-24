# Spike: a scaffold over any web page

Status: spike, 2026-11-24. Code: `tools/scaffold/`. Question: can the engine generate and measure
design variants of a page it does not own and has no specification for, "take a random website,
then basically inject a sort of a scaffold, and when the scaffold is moving, the thing is also
moving"? How fast, and how far does it go?

The scaffold is a funnel of four stages:

| stage | what | where | built |
|---|---|---|---|
| 0 lift | a model (here: the agent) names the components of one part of the page and writes an `ess-ui/1` document for it, with the page's real content as fixtures and a binding table node path -> CSS selector | `tools/scaffold/lift/docs-hero/` | yes |
| 1 brute force | every variant of the lifted document (section order, actions, component, label, layout), checked with `ess ui check`, its journeys run with `ess ui test` on the terminal renderer, cheap metrics read from the document and the terminal's cells | `stage1.mjs`, `cells/` | yes |
| 2 project | the best variants of stage 1 projected back onto the real page through the binding table and the runtime, measured in Chromium at three viewports | `stage2.mjs`, `runtime.js`, `sweep.mjs` | yes |
| 3 judge | screenshots, a model's judgement, a person | `sweep.mjs --shots` writes the screenshots | no |

Stage 2's runtime also works alone, without a lift, on generic axes (`mode`, `density`,
`font_scale`, `accent`, `section_order`, `hide`, `max_width`): `probe.mjs`, `sweep.mjs`,
`panel.js` and `serve.mjs`.

No third-party dependency: Node 22 (global `WebSocket` and `fetch`) and Chromium driven over the
DevTools protocol (`cdp.mjs`, 200 lines). The cell reader `cells/` is a Rust binary over the ESS
crates `ess-ui`, `ess-ui-check` and `ess-ui-tui` (path dependencies on an ESS checkout beside this
repository).

## Stage 0: the lift

Target: the hero of the documentation landing page `/uxui/` (kicker, title, lede, five actions,
meta line, three figures).

Effort: about 15 minutes for the first version, of which about 10 were reading the `ess-ui/1` and
`ess-ui-test/1` references; the document passed `ess ui check` on the first run and its three tests
after one fix. Two corrections came later, both found only by stage 2: the hero is one column, not
two (the first lift guessed columns from the content); and the page gained a fifth action while the
spike ran (another session rebuilt the site), which took one minute to add to the document, the
binding and the space. The document drifts from the page the moment the page changes; the binding
table still resolved, so a drift check (every selector matches one element, every element in the
lifted part has a node) is cheap and belongs in stage 0.

What the model had to decide:

1. The grain: six visible parts became the page title and five sections.
2. A component for each part. `ess-ui/1` has no plain paragraph: the kicker, the lede and the meta
   line are each a `record` with one labelled field, and the labels ("About", "What it is", "Kind")
   are invented. The figures are a `collection` table.
3. The actions: an action navigates only to a page of the document, so each link target is a stub
   page (5 invented pages); the secondary links carry `as: link`.
4. The layout (it guessed wrong, above).
5. What is data (fixtures: the texts and the figures) and what is structure.
6. The binding: 16 selectors, each checked to match exactly one element.
7. The design space (`space.json`): which orders, action sets, labels and layouts are worth trying.

What `ess-ui/1` could not express: emphasis inside the title (its second line is `<em>`); a kicker,
a lede or a caption as text without a label; an action to an address outside the document;
decoration (the kicker's dot, the figures' index numbers and status glyphs); and anything the
terminal does not draw: colour, type scale, spacing. The test runner also refuses to read an
action's label inside a record ("the terminal does not draw this node on cells of its own").

## Stage 1: brute force without a browser

Space: 3 section orders x 3 action sets x 2 figure components (table, none) x 2 primary labels x
2 layouts (stack, columns) x 2 densities = 144 variants. Hard rules: the document checks, both
journeys complete ("install the plugin", "what exists today"), the primary action is in view, the
required sections are drawn. KPI: 1e6 x failures + words before the primary action + 2.4 x terminal
rows above it.

Time per variant (144 variants; shared host at load average about 100):

| step | per variant | 144 variants |
|---|---|---|
| materialize the variant document and its test file (JSON, which `ess` reads as YAML) | 0.4-1.2 ms | 0.2 s |
| static reading of the document (words and sections before the action) | 18 µs | under 3 ms |
| `ess ui check` as a process | 40 ms | |
| `ess ui test` (2 journeys) as a process | 120-150 ms | |
| both as processes | about 190 ms | 28-31 s |
| in-process (`cells/`): check 2.6 ms, open the page 1.2 ms, draw 80x24 0.4 ms | 5 ms (the first 100 ms) | 0.9 s, one process |

The number that decides the design: through the CLI, stage 1 runs about 5 variants a second; in
one process with the ESS crates, about 200 a second per core, 40 times faster. Process start and
schema loading are most of the CLI time. A brute-force engine has to call the ESS crates
in-process, not the CLI.

Result: 48 of 144 variants pass. 48 fail the journey "what exists today" (its link removed), 48
lose the figures (a required section). The cell metrics separate only the section order: the
primary action sits at terminal row 8 with 28 words above it in the original order, at row 5 with
14 words when the actions come before the lede. Layout, density and the label change nothing on
the terminal: it falls back to a stack for `columns` at 80 cells, draws no tokens, and the label
fits. It also cuts a record field to one line, so the 60-word lede counts as 10 words.

## Stage 2: projection onto the real page

`stage2.mjs` takes the top 3 of stage 1 and a spread over the rest of its ranking, turns each into
DOM operations through the binding table (`order` of the bound sections among their own slots,
`hide` for removed actions and sections, `text` for the label, `style` for the column layout,
`density` scoped to the hero), and measures them with `sweep.mjs` at 1440x900, 1024x768 and
390x844, with the same hard rules (journey targets and required sections present and rendered).

Time: 450-560 ms per variant for 3 viewports (a viewport switch waits two animation frames;
`apply` plus `measure` in the page is 30-150 ms of it); one page load of 2-5 s per run. 24
projected variants plus the baseline: 15-21 s.

Agreement on 24 projected variants (the stage-1 top 3 and 21 spread over its ranking):

| | value |
|---|---|
| Kendall's tau (ties count against) | 0.815 |
| pairs concordant / discordant / tied in stage 1 | 225 / 0 / 51 |
| stage-1 top 3 in the stage-2 top 3 | 2 |
| stage-1 rank 1 is stage-2 rank 1 | yes: actions before the lede, all actions, figures shown, compact |

Stage 1 never ranked two variants the wrong way round; it tied 51 pairs that the browser
separates, all of them differing in an axis the terminal does not draw. Density 0.8 moves the
primary action from 376 to 346 px (worst viewport); the two-column layout moves it to 423-443 px
and, with the original order, out of view at 390x844 (1013 px), which stage 1 passed. Stage 1's
third place (the same variant with columns) is the browser's sixth. In the first stage-2 run,
which checked the journeys but not the required sections, tau was 0.53: a hard rule missing in one
stage costs more agreement than the renderer does.

On the page the moves work as bound: the actions move above the lede inside the hero's copy
column, removed links and figures disappear, the label changes. One artifact: the meta line lost
its top margin after the move, because the site's CSS spaces it by its neighbour.

## Generic axes without a lift

`sweep.mjs <url> <axes.json>` on the three targets, 32 + 6 variants each, 3 viewports. Times were
taken on a shared host at load average about 100; `apply` plus `measure` in the page is the part
that depends least on it.

| page | scaffold | median per variant (3 viewports) | apply / measure in the page |
|---|---|---|---|
| documentation site `/uxui/` (Docusaurus, server-rendered) | 30.8 KB: 331 elements, 9 regions, 5 sections, 11 headings, 51 interactive | 640 ms | 72 / 50 ms |
| `https://example.com/` (built by a script; no landmarks, no headings) | 3.2 KB: 13 elements, 6 sections (one paragraph per language) | 170 ms | 4 / 6 ms |
| `https://www.rust-lang.org/` (redirects to `rust-lang.org`) | 17.8 KB: 196 elements, 3 regions, 6 sections, 22 headings, 37 interactive | 660 ms | 35 / 32 ms |

Per axis:

| axis | documentation site | example.com | rust-lang.org |
|---|---|---|---|
| mode dark | works; contrast 4.7 or more after the fix below | works, contrast 7.6 | works; coloured section fills kept |
| density | 0.8 moves the primary action 652 -> 627 px | 0.8 brings the link into view at 390x844 (864 -> 788 px) | 1.25 causes horizontal scroll at 1024x768 |
| font_scale 1.2 | primary action out of view at 390x844 | out of view at 1024 and 390 | moves the action 444 -> 525 px |
| accent | recolours the green; with dark mode it first broke the button (contrast 1.83), fixed | recolours the link | recolours the yellow |
| section_order reverse | hero last, primary out of view everywhere | works (paragraphs) | hero last, out of view |
| hide | the largest section | one paragraph; action 864 -> 716 px | works |
| max_width | no effect on these metrics | none | 720 causes horizontal scroll at 1024x768 |

Found on the pages as they are: example.com's only link is out of view at 390x844; rust-lang.org
fails contrast at every viewport on one element, the label "Language" drawn in its own background
colour (ratio 1).

Defects the sweeps found in the runtime, fixed: dark mode left navigation text at 4.44 (the remap
now keeps each text's contrast on its remapped surface, up to 4.5); dark mode with a new accent
inverted the accent button (fills that carry colour keep their lightness); the primary action was
the skip link on the documentation site and a lower button on rust-lang.org (skip links and
off-page controls are excluded, and a `header` inside `main` counts as content).

## The live panel

`serve.mjs <url> --port 5301` serves the page through a read-only proxy (GET and HEAD) with
`runtime.js` and `panel.js` injected; `/__uxui/` offers two bookmarklets that inject the same panel
into any open tab. Each control applies the variant at once (3-30 ms on the documentation site)
and measures the hard rules and metrics again at the current viewport. With
`--variants <list.json>` the panel lists prepared variants, for example the projected variants of
the lift.

## Limits and what would fix them

1. **Stage 1 is blind to most of what moves the real page.** The terminal renderer ignores tokens
   (density), falls back from columns to a stack, cuts fields to one line, and has one width. It
   agreed with the browser on order and presence and tied everything else. Fix: a cell layout model
   that wraps text and lays out at several widths (the roadmap's layout model v0), or the terminal
   renderer drawn at two or three widths with wrapped fields, and spacing tokens mapped to blank
   rows; until then stage 1 prunes (hard rules, order) and stage 2 ranks.
2. **The lift is manual, lossy and drifts.** 15 minutes for one hero; invented labels and stub pages
   because `ess-ui/1` has no plain text block and no action to an outside address; the layout was
   guessed wrong; the page changed during the spike. Fix: feed `probe.mjs`'s scaffold (sections,
   boxes, headings, interactive elements, primary action) to the model as the lift's input instead
   of the HTML; check drift in stage 0 (every selector matches one element, no visible text in the
   lifted part without a node); propose a text composite and an `href` action for `ess-ui/1`.
3. **Projection is surgery on someone else's CSS.** DOM moves break spacing written for neighbours
   (`+` selectors), the column projection is not responsive (it fails at 390x844), the generic axes
   set inline styles per element at one viewport (captured again per viewport; pseudo-elements and
   images are not remapped), and a framework that re-renders can undo them. Fix: where the site is
   ours, project into its sources (`product.json`, CSS custom properties, as `docs/targets.md`
   plans); for foreign pages, emit one injected stylesheet with media queries per variant instead
   of inline styles, and add a visual diff of the screenshots to stage 3.

Other limits: the primary action and the sections are heuristics (`--primary <selector>` overrides
the first); contrast is computed from ancestors' backgrounds, so text over an image or a positioned
sibling is "can't tell" or wrong; the accessible name approximates the browser's; the tools are
JavaScript for a zero-dependency spike, while this repository's rule for its tools is POSIX sh or
Rust.

## Files

| file | what |
|---|---|
| `tools/scaffold/cdp.mjs` | Chromium over CDP: launch, page, evaluate, viewport, screenshot |
| `tools/scaffold/runtime.js` | injected: `__uxui.scaffold()`, `apply(variant)`, `reset()`, `measure()` |
| `tools/scaffold/panel.js` | the floating control panel |
| `tools/scaffold/probe.mjs` | `probe.mjs <url>` writes the `uxui-scaffold/1` JSON |
| `tools/scaffold/sweep.mjs` | exhaustive sweep over axes or a list of variants: climb lines, frontier, screenshots |
| `tools/scaffold/serve.mjs` | the live demo: proxy or snapshot with the panel, bookmarklets |
| `tools/scaffold/stage1.mjs` | stage 1 over a lifted document |
| `tools/scaffold/stage2.mjs` | stage 2: projection, measurement, rank agreement |
| `tools/scaffold/cells/` | Rust: `ess ui check` and the terminal renderer in-process, cell geometry |
| `tools/scaffold/lift/docs-hero/` | the lifted hero: `hero.ui.json`, `fixtures/hero.json`, `tests/hero.yaml`, `binding.json`, `space.json` |

Run (scripts that measure run without `slot`; `cells/` builds through it):

```sh
node tools/scaffold/probe.mjs http://127.0.0.1:3299/uxui/
node tools/scaffold/sweep.mjs http://127.0.0.1:3299/uxui/ auto --limit 48
node tools/scaffold/stage1.mjs tools/scaffold/lift/docs-hero --out target/stage1 --cells <uxui-cells>
node tools/scaffold/stage2.mjs tools/scaffold/lift/docs-hero http://127.0.0.1:3299/uxui/ --stage1 target/stage1/stage1.json --spread 21
node tools/scaffold/serve.mjs http://127.0.0.1:3299/uxui/ --port 5301
```

## Language of the spike code

`runtime.js` and `panel.js` run inside the page and are JavaScript by necessity. The drivers (`cdp.mjs`, `probe.mjs`,
`sweep.mjs`, `serve.mjs`, `stage1.mjs`, `stage2.mjs`) are spike code in Node with no packages; when a driver is
kept beyond the spike it is ported to Rust (`AGENTS.md`: tools are POSIX sh or Rust). Stage 1 must call the ESS
crates in process (`cells/`: about 5 ms per variant against about 190 ms through the CLI).
