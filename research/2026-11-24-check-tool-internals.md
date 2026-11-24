# How the deterministic UI check tools work inside, and what owning each takes

Date 2026-11-24. This note goes past `research/2026-11-23-external-machine-checkable-ux.md` and does not
repeat it. That note said what the tools claim. This one reads their source and asks what each check
needs once the UI is a declared model: an `ess-ui/1` document plus a deterministic layout model
(`docs/state-space.md` § 4), and no browser. "[Inference]" marks my own reading, not a source claim.
Line references are to the commits below. Each link opens the file at that commit; a `#L` anchor
opens the line.

| tool | repository | commit read | version |
|---|---|---|---|
| axe-core | https://github.com/dequelabs/axe-core | `32d9bd503128d4b52d818ca89140c1debe5d70d5` | 4.14.1 |
| Lighthouse | https://github.com/GoogleChrome/lighthouse | `f482542ae0c5f05bcd1831906e7e4b4b1857ca4b` | 13.6.0 |
| Playwright | https://github.com/microsoft/playwright | `61775c1ae65409859ebc4e60dec3a4a3a9cc3f92` | main, 2026-11-23 |
| pixelmatch | https://github.com/mapbox/pixelmatch | `ab76f526ef951c0ec6284e314a700e161b1800e6` | 8.0.0 |
| odiff | https://github.com/dmtrKovalenko/odiff | `6dd1e9629810042c03a81fc787f37b245772970a` | 4.5.0 |
| ssim.js | https://github.com/obartra/ssim | `fc0bf9bddbbad71810eb4f04fc427ae25593e809` | master |
| rrweb | https://github.com/rrweb-io/rrweb | `d6926830927706b96d6c018d77d40a945c187342` | 2.2.2 |
| apca-w3 | https://github.com/Myndex/apca-w3 | `6c5c066cf0c853507930a4239b129283b8e05353` | 0.1.9 |

Short names in this note: `AX` = `https://github.com/dequelabs/axe-core/blob/32d9bd503128d4b52d818ca89140c1debe5d70d5/`,
`LH` = `https://github.com/GoogleChrome/lighthouse/blob/f482542ae0c5f05bcd1831906e7e4b4b1857ca4b/`,
`PW` = `https://github.com/microsoft/playwright/blob/61775c1ae65409859ebc4e60dec3a4a3a9cc3f92/`. A path
written `AX lib/x.js#L5` is the URL `AX` + `lib/x.js#L5`.

The geometry classes used below:

- **structure**: the declared document alone: node kinds, roles, names, order, states, tokens.
- **cells**: a layout model on a character grid (ESS's terminal renderer gives one today:
  `Region {path, row, area, text}` per frame, `docs/state-space.md` § 4).
- **px**: a layout model in CSS px with font metrics (`docs/state-space.md` § 4, not built yet).
- **pixels**: a raster of the laid-out UI.
- **browser**: a real web engine, real fonts or a real assistive technology.

## 1. axe-core

### Mechanism

- **What a rule is.** Each rule is a JSON file: `selector`, an optional `matches` function, and three
  check lists `any`, `all`, `none` (for example `AX lib/rules/color-contrast.json`). There are 105 rule
  files in `lib/rules`. 30 are tagged `best-practice` and 6 `experimental`. The 105 includes the
  deprecated rules.
- **Outcome per node.** Each check returns `true`, `false` or `undefined`, and `undefined` means
  "can't tell". `aggregate-checks.js` maps them to priorities. The node result is the worst of three
  parts: the worst `all`, the worst inverted `none`, and the best `any`
  (`AX lib/core/utils/aggregate-checks.js#L44-L56`). Rule outcomes are `passes`, `violations`,
  `incomplete` and `inapplicable`; `incomplete` is the ACT `cantTell`
  (`AX doc/API.md#L671`).
- **What it reads.** Rules skip hidden nodes by default (`excludeHidden`,
  `AX lib/core/base/rule.js#L48`). "Hidden" is computed from computed style. Roles, accessible
  names and ARIA validity come from data tables in `lib/standards` (2,728 lines: `aria-roles.js`,
  `aria-attrs.js`, `html-elms.js`) and from an implementation of the accessible-name algorithm in
  `lib/commons/text` (1,945 lines).
- **How much needs geometry.** I listed each rule's `matches` and check `evaluate` files and searched
  them for layout and style reads (`getBoundingClientRect`, `getClientRects`, `elementsFromPoint`,
  the internal grid, `getComputedStyle`, background and foreground colour, scroll). 86 of 105 rules
  make no such call at that level. They read only the DOM, the ARIA tables and the accessible name.
  The 19 that do:
  - geometry for real: `color-contrast`, `color-contrast-enhanced`, `link-in-text-block`,
    `target-size`, `scrollable-region-focusable`, `object-alt`;
  - computed style only (spacing, display, font size): `avoid-inline-spacing`, `p-as-heading`,
    `hidden-content`, `aria-hidden-focus`;
  - a visible-on-screen guard only: `autocomplete-valid`, `blink`, `button-name`,
    `input-button-name`, `input-image-alt`, `label`, `marquee`, `select-name`, and `skip-link`
    (which reads offscreen).
  [Inference: this is direct calls only; shared helpers that check hidden state read style for every
  rule.]
- **The geometry rules rebuild the browser's paint order.** axe builds its own spatial grid of
  200 px cells over every visible box (`AX lib/commons/dom/create-grid.js#L43`, `gridSize: 200` in
  `AX lib/core/constants.js#L31`). It computes stacking contexts itself
  (`AX lib/commons/dom/create-grid.js#L13-L35`) and sorts nodes by CSS 2 paint order
  (`AX lib/commons/dom/visually-sort.js#L11-L37`).
- **`color-contrast`, step by step**
  (`AX lib/checks/color/color-contrast-evaluate.js`):
  1. font size in pt: `Math.ceil(fontSize * 72) / 96`; large text is 18 pt, or 14 pt at weight
     700 or more (`#L49-L51`, options in `AX lib/checks/color/color-contrast.json`);
  2. the background: walk the stack of elements under the text until an opaque background, then
     blend bottom-up over white (`AX lib/commons/color/get-background-color.js#L66-L140`);
  3. pass if the ratio is above the limit: `isValid = contrast > expected` (`#L107`). Ratio =
     (L1 + 0.05) / (L2 + 0.05) (`AX lib/commons/color/get-contrast.js#L24`). Luminance uses the
     sRGB threshold 0.04045 and exponent 2.4 (`AX lib/commons/color/color.js#L220-L228`).
- **When contrast is "can't tell".** The check returns `undefined` for 16 named reasons
  (`AX lib/checks/color/color-contrast.json`, `incomplete` messages). Among them: background
  image, gradient, overlap by another element, partly obscured, outside the viewport, pseudo-element
  content, a complex text shadow, a single character. The stack walk gives up when the element on
  top is not the text node (`AX lib/commons/color/get-background-stack.js#L24-L33`).
- **`target-size`** (`AX lib/checks/mobile/target-size-evaluate.js#L17-L61`): a target of at least
  24 × 24 px passes. A smaller target that is not obscured takes the WCAG 2.5.8 spacing exception
  through `target-offset` (`AX lib/checks/mobile/target-offset-evaluate.js#L10-L60`): find the
  nearby targets on the grid and measure their distance. A target that is not in the tab order can
  only end as "can't tell", never as a failure (`#L54-L61`).

### What it catches and misses

- **Coverage by criterion, from the source.** Collect the WCAG criterion tags of all rules
  (`wcag111`, `wcag143`, ...). They name 29 distinct criteria, 4 of them AAA (1.4.6, 2.2.4, 2.4.9,
  3.2.5). That leaves 25 of the 55 Level A and AA criteria of WCAG 2.2 (31 A plus 24 AA;
  https://www.w3.org/TR/WCAG22/). A rule tagged with a criterion tests one failure of it, not the
  whole criterion. No rule is tagged:
  - 2.1.2 No Keyboard Trap;
  - 2.4.3 Focus Order;
  - 2.4.7 Focus Visible;
  - 2.4.11 Focus Not Obscured;
  - 1.4.10 Reflow;
  - 1.4.13 Content on Hover or Focus;
  - 3.2.1 / 3.2.2 On Focus / On Input.
- **What the vendor claims.** "On average 57% of WCAG issues" (`AX README.md#L19`) and "zero false
  positives (bugs notwithstanding)" (`AX README.md#L71`). The zero is a design rule: when a rule is
  unsure it returns "incomplete" (`AX doc/rule-proposal.md#L5`).
- **An independent benchmark.** Six tools were tested against WCAG 2.0: they covered at most 50 % of
  the criteria (23 to 50 %) and found 14 to 38 % of the real violations. The tools that found the
  most had the lowest correctness, 66 to 71 %; the others reached 93 to 96 %. Source: "Benchmarking
  web accessibility evaluation tools: measuring the harm of sole reliance on automated tests", W4A
  2013, https://ro.ecu.edu.au/ecuworks2013/236/ ; the numbers as summarised in
  https://www.researchgate.net/publication/262352732 . I did not read the full paper.
- **What the common failures are.** The 2026 WebAIM Million found WCAG 2 failures on 95.9 % of the
  top 1,000,000 home pages. The six most common failures make up 96 % of all errors detected:
  - low contrast text, 83.9 %;
  - missing alt text, 53.1 %;
  - missing form labels, 51 %;
  - empty links, 46.3 %;
  - empty buttons, 30.6 %;
  - missing document language, 13.5 %.

  https://webaim.org/projects/million/ . [Inference: four of the six are "a node with no
  accessible name", a structure check, and the fifth is contrast. A declared model checks both
  without layout.]

### Determinism

- The same DOM can give different results on different runs:
  - contrast and target size depend on the viewport, because nodes outside it are "can't tell";
  - they depend on scroll position (axe scrolls the page, and Lighthouse scrolls it back:
    `LH core/gather/gatherers/accessibility.js#L90-L92`);
  - they depend on fonts (text boxes) and on timing (a transition caught halfway).
- The rule logic itself is deterministic. The nondeterminism comes from the rendered page it reads.

### Owning it on a declared model

- **Structure (about 86 rules' worth).** Names, roles, required children and parents, ARIA attribute
  validity, heading order, landmarks, duplicate ids, list structure, table headers. On ESS most of
  these hold by construction, since ESS emits the roles itself. So the rules worth porting are the
  ones a document author can break:
  - missing names: `button-name`, `link-name`, `image-alt`, `label`, `select-name`;
  - empty headings;
  - heading order;
  - one main landmark;
  - duplicate accessible names on sibling actions;
  - `label-content-name-mismatch` (2.5.3).

  The ARIA tables are data. ESS's node vocabulary is closed, so the role table is one row per ESS
  node kind, not the 2,728 lines of ARIA tables.
- **Tokens, no geometry.** Contrast, once the layout model says which surface each text sits on
  (§ 7).
- **Cells.** Obscured or partly obscured, offscreen, scrollable region focusable.
- **px.** Target size and spacing (WCAG 2.5.8 is defined in CSS px). A cell grid can only give a
  terminal analogue (at least one full cell row, label width plus padding) [Inference].
- **Size.**
  - axe-core is about 36,000 lines of JavaScript (`lib/core` 12,701, `lib/commons` 13,707,
    `lib/checks` 5,817, `lib/rules` 961, `lib/standards` 2,728).
  - [Inference] An ACT-format rule engine with about 15 structural rules, the ESS role table and
    accessible names for ESS nodes: about 1,200 to 1,800 lines of Rust. Most of axe's size goes to
    things a declared model already has: the shadow DOM, iframes, stacking contexts, background
    stacks and the many ways HTML spells one role.

## 2. Lighthouse accessibility and best-practices audits

### Mechanism

- **Accessibility is axe-core under another runner.** Lighthouse runs it with
  `runOnly: tags [wcag2a, wcag2aa]` and a list of rules switched on or off
  (`LH core/gather/gatherers/accessibility.js#L27-L87`). The list switches off
  `scrollable-region-focusable`, `duplicate-id`, `nested-interactive` and 13 others.
- **Scoring is binary per audit.** `score: Number(rule === undefined)`
  (`LH core/audits/accessibility/axe-audit.js#L106`): one failing node and 100 failing nodes score
  the same.
- **Can't tell counts as a pass.** For a scored audit, an `incomplete` result without an error is
  dropped (`#L47-L58`).
- **Not applicable leaves the denominator** (`#L35-L41`). The category score is the weighted mean
  over the applicable audits (`LH core/scoring.js#L18-L41`).
- **Weights follow axe impact:** critical 10, serious 7, moderate 3, minor 1; experimental rules
  weigh 0 (`LH core/config/default-config.js#L483-L500`). 76 audits sit in the category.
- **The manual audits weigh 0.** `logical-tab-order`, `focus-traps`, `focusable-controls`,
  `managed-focus`, `visual-order-follows-dom`, `offscreen-content-hidden`, `use-landmarks` and three
  more (`LH core/audits/accessibility/manual/`; weights at
  `LH core/config/default-config.js#L569-L571`). Lighthouse marks exactly the focus and order checks
  as not automatable.
- **Best practices** (`LH core/config/default-config.js#L595-L618`) is about the platform, not the
  UI: HTTPS, console errors, deprecated APIs, third-party cookies, doctype, charset, image aspect
  ratio, paste-blocking inputs.

### Catches, misses, determinism

- It catches what axe catches minus the rules it switches off. It misses whatever axe marks "can't
  tell", because that counts as a pass.
- [Inference] The score can be gamed: a node that is removed takes its audit out of the denominator,
  and a single violation costs the whole audit weight however many nodes fail.
- Variability: the external note covers it (`research/2026-11-23-external-machine-checkable-ux.md`
  § 5).

### Owning it

- Nothing to own beyond § 1. The parts worth keeping:
  - its weights table, as a default severity mapping;
  - its list of manual audits, as a list of the checks a declared model can turn from manual into
    exact (§ 8).
- The best-practice audits are about network and platform and have no counterpart in a declared
  model: they stay in the browser assessor (`ux-walk@1`).

## 3. Playwright layout assertions: visible, in viewport, overlap, scroll

### Mechanism

- **`isVisible` / `toBeVisible`.** The element is visible when `checkVisibility()` is true, the
  computed `visibility` is `visible` and its bounding box is non-empty
  (`PW packages/injected/src/domUtils.ts#L136-L188`). It does **not** consider:
  - the viewport: an element 5,000 px below the fold is "visible";
  - occlusion by another element;
  - clipping by an `overflow: hidden` ancestor;
  - `opacity: 0`.
- **`toBeInViewport({ratio})`.** It asks an `IntersectionObserver` for the element's
  `intersectionRatio` (`PW packages/injected/src/injectedScript.ts#L639-L650`) and passes when
  `ratio > 0 && ratio > expected - 1e-9` (`#L1631-L1634`). The intersection ratio includes clipping
  by ancestors but not occlusion: a control under a sticky header has ratio 1.
- **Occlusion is checked only before a pointer action.** `expectHitTarget` calls
  `elementsFromPoint` at the action point and requires the target or a descendant to be on top
  (`PW packages/injected/src/injectedScript.ts#L1037-L1100`). There is no assertion for it.
- **Stability.** "Stable" means the bounding box was equal on consecutive animation frames
  (`#L717-L745`; the frame count comes from the browser delegate,
  `PW packages/playwright-core/src/server/dom.ts#L107`).
- **Overlap of two elements and overflow or scroll.** Playwright has no assertion for either. Tests
  compare `boundingBox()` values, or evaluate `scrollWidth > clientWidth`, in test code.

### What a pointer action really does

`_retryPointerAction` scrolls the target into view. On each retry it cycles through four scroll
alignments ("that might not work to scroll from under position:sticky elements … we cycle through
different scroll alignments", `PW packages/playwright-core/src/server/dom.ts#L383-L397`). It waits 0,
20, 100, 100, then 500 ms between retries (`#L330-L331`), until the action timeout. [Inference] So a
Playwright click succeeds on a target that is offscreen, under a sticky bar or still moving. A person
would have had to find it, scroll, or wait. A "screen-only" bot built on `getByRole().click()` gets
help a person does not get, unless the bot checks visibility itself before it acts.

### Catches, misses, determinism

- It catches elements missing from the page, hidden by style, or outside the viewport (only with
  `toBeInViewport`).
- It misses occlusion, partial clipping when no ratio is given, overlap between siblings, and
  horizontal overflow, unless the test code computes them.
- **Flakiness.** An empirical study of 235 flaky UI tests in 62 projects found the largest cause to
  be asynchronous waits, 105 tests or 45 %. Waits were more often the cause in web projects than in
  mobile ones (52.0 % against 32.5 %). Layout differences across platforms caused 5.3 % in web
  projects. Source: "An Empirical Analysis of UI-based Flaky Tests", ICSE 2021,
  https://arxiv.org/abs/2103.02669 .
- [Inference] Playwright's retries hide this kind of flake instead of removing it.

### Owning it on a declared model

All of these predicates are pure functions of boxes. On the cells model, per node path n and state s:

- `vis(n)`: the share of n's cells inside the viewport and not clipped by a non-scrolling ancestor;
- `cov(n)`: the topmost other node on n's cells, from the layer order (overlay above page, sticky
  above flow);
- `overflow(n)`: n's cells leave its container's cells and the container does not scroll;
- document scroll: content wider or taller than the viewport;
- stable: the box is equal in s and its successor without a user action.

The terminal renderer already records per frame which cells each node path occupies, so in-viewport,
overflow and truncation work on cells. Cover needs one more fact, a layer index per region
(`docs/state-space.md` § 4: overlay and page header exist as regions). [Inference] About 400 to 600
lines of Rust over the `Region` list, with exact integer arithmetic and no retries. On cells the
"always visible" gate and the "no scroll" gate are exact.

## 4. Pixel diff: pixelmatch, odiff, Playwright's two comparators, SSIM

### Mechanism

- **pixelmatch 8.0.0.** The threshold is now a distance in OKLab HyAB, default 0.1
  (`https://github.com/mapbox/pixelmatch/blob/ab76f526ef951c0ec6284e314a700e161b1800e6/index.js#L11`).
  It does a per-pixel colour test (`#L83`). A differing pixel is dropped as anti-aliasing when it
  matches the detector, which follows the 2009 paper "Anti-aliased Pixel and Intensity Slope
  Detector" (`#L90`, `#L199-L259`). The detector looks at the 8 neighbours:
  - if more than 2 neighbours are equal to the pixel, it is not anti-aliasing;
  - it needs both a darker and a brighter neighbour;
  - the darkest or the brightest neighbour must have 3 or more equal neighbours in both images.

  The result is a count of differing pixels.
- **Playwright vendors an older pixelmatch.** It uses YIQ distance with `maxDelta = 35215 ×
  threshold²` (`PW packages/utils/third_party/pixelmatch.js#L63-L64`) and a default threshold of
  0.2 (`PW packages/utils/comparators.ts#L87-L90`). Pass if the count of differing pixels is at
  most `maxDiffPixels` or `maxDiffPixelRatio × area`; the default is 0 (`#L95-L106`). [Inference]
  The same option name, `threshold`, now means different things in pixelmatch 8 and in Playwright's
  copy.
- **Playwright's `ssim-cie94` comparator** (`PW packages/utils/image_tools/compare.ts#L36-L122`),
  per pixel:
  1. equal bytes pass;
  2. a ΔE94 below 1.0, the "just noticeable difference", passes;
  3. if either image is flat in the 3 × 3 window around the pixel, it is a real difference;
  4. otherwise compute SSIM over a 31 × 31 window (radius 15); SSIM ≥ 0.99 counts as
     anti-aliasing.
- **Stable screenshots.** `toHaveScreenshot` captures until two consecutive screenshots match
  ("Failed to take two consecutive stable screenshots",
  `PW packages/playwright-core/src/server/page.ts#L740-L803`).
- **odiff 4.5.0** is Zig with SIMD and an assembly path. It uses the same YIQ metric, 35215
  (`https://github.com/dmtrKovalenko/odiff/blob/6dd1e9629810042c03a81fc787f37b245772970a/src/diff.zig#L27`).
  Defaults: threshold 0.1, anti-aliasing detection off, so anti-aliased pixels count as
  differences; ignore regions; images of different sizes fail by default (`#L139-L148`, `#L266`).
  The fast assembly path runs only with default options and images of equal size (`#L272-L276`).
- **SSIM.** The definition is from "Image Quality Assessment: From Error Visibility to Structural
  Similarity", IEEE TIP 2004, https://www.cns.nyu.edu/pub/eero/wang03-reprint.pdf : C1 = (0.01 L)²,
  C2 = (0.03 L)², an 11 × 11 Gaussian window with σ 1.5, and the mean over windows. ssim.js has
  different defaults:
  - Weber's integral-image variant, not the Gaussian window;
  - a downsample of the image to at most 256 px;
  - integer greyscale conversion.

  Source: `https://github.com/obartra/ssim/blob/fc0bf9bddbbad71810eb4f04fc427ae25593e809/src/defaults.ts#L3-L12`.
  [Inference] So "SSIM 0.98" from two libraries is not one number.

### Catches, misses

- **Pixel diffs catch** any visible change: a lost border, a wrong colour, a shifted icon, a font
  change.
- **They miss** what does not change pixels:
  - a missing accessible name;
  - the wrong focus order;
  - a control that is visible but covered by a transparent layer.
- **The verdict is a count with no cause.** The external note's point stands: a loose tolerance
  hides real regressions.
- **SSIM** is weak on small, high-contrast UI changes in large flat areas: it was designed for
  natural images.

### Determinism

- **Sources of difference:**
  - font rasterisation and hinting;
  - subpixel positioning;
  - GPU and Skia backend;
  - colour profile (Playwright forces `--force-color-profile=srgb`,
    `PW packages/playwright-core/src/server/chromium/chromiumSwitches.ts#L86`);
  - the text caret, which Playwright hides
    (`PW packages/playwright-core/src/server/screenshotter.ts#L92-L105`);
  - animations;
  - browser version.
- Playwright's own documentation says rendering "can vary based on the host OS, version, settings,
  hardware, power source" and names snapshots per browser and platform
  (https://playwright.dev/docs/test-snapshots).
- The comparators exist to tolerate this noise, which is what makes them lossy.

### Owning it on a declared model

- **With our own rasteriser** (a pixel model drawn from the px layout with a fixed font table and
  integer coverage), two runs give identical bytes. The comparison becomes byte equality plus a list
  of differing node paths. No anti-aliasing detection, no tolerance, and no image codec: both
  frames are in memory.
- [Inference] The price is that a byte-equal diff then measures only our own renderer, not the
  browser the user sees. Its value as a gate is low: geometry and token checks state the same
  regression with a cause. It stays a review aid (contact sheets) and a fidelity probe against the
  browser.
- **Size.** For tolerant comparison of browser screenshots (fidelity in `ux-walk`): pixelmatch is
  605 lines of JavaScript, the `ssim-cie94` comparator 428 lines of TypeScript, odiff's core 703
  lines of Zig. [Inference] A Rust port of the HyAB, anti-aliasing and SSIM parts is about 500 to
  700 lines. Reading browser PNGs without a dependency adds an inflate decoder of about 800 to
  1,200 lines.

## 5. rrweb recording and replay

### Mechanism

- **A recording is a full snapshot plus events.** A full snapshot is the serialised DOM with node
  ids (`rrweb-snapshot`). It is followed by incremental events of 17 sources: mutation, mouse move,
  mouse interaction, scroll, viewport resize, input, style sheet rule, canvas mutation, font and
  others. Event types: `https://github.com/rrweb-io/rrweb/blob/d6926830927706b96d6c018d77d40a945c187342/packages/types/src/index.ts#L1-L11`
  and `#L145-L163`.
- **Timestamps are wall clock** (`nowTimestamp()`,
  `.../packages/rrweb/src/record/index.ts#L188`).
- **Options** include `checkoutEveryNms` / `checkoutEveryNth` (a new full snapshot),
  `maskAllInputs`, `maskTextSelector`, `recordCanvas` (off by default), `collectFonts` (off by
  default), `inlineImages` (off by default) and `sampling` (`#L71-L98`).
- **The recording holds no geometry.** Layout is computed again by the browser that replays. Only
  blocked elements carry a size (`rr_width`, `rr_height`,
  `.../packages/rrweb-snapshot/src/snapshot.ts#L760-L766`), and scroll offsets are kept.
- **Replay** rebuilds the DOM in an iframe and plays events on a timer driven by
  `requestAnimationFrame` and `performance.now()`, at a speed factor
  (`.../packages/rrweb/src/replay/timer.ts#L42-L62`).
- **Video.** `rrvideo` makes a video by playing the replay in a Playwright Chromium with
  `recordVideo` (`.../packages/rrvideo/src/index.ts#L3`, `#L140`).

### Catches, misses, determinism

- A recording is evidence of what a person saw and did; it is not a check.
- **Fidelity depends on the replaying environment:**
  - fonts are not recorded unless `collectFonts`;
  - canvas and WebGL are not recorded unless `recordCanvas`;
  - external style sheets are inlined only when they are readable;
  - cross-origin iframes need the iframe's cooperation.
- [Inference] A replayed frame is a new rendering, not a copy of what the user saw. Metrics computed
  from it inherit § 4's sources of difference. The timer makes replay timing approximate.

### Owning it on a declared model

The declared model is the source of truth, so a recording does not need the DOM. It needs only a
log of:

- the start state digest;
- `(virtual time, action on a node path, resulting state digest)` per step;
- the environment.

Replay is "open the document with the fixtures, apply the actions": ESS's `App` already does this on
a virtual clock (`docs/state-space.md` § 3). Every replayed state can be laid out again and checked
again with the current checks, and the digest tells whether it is the same state. [Inference] About
200 to 400 lines of Rust plus a format.

- **Recordings of real users** in a browser still need a recorder. But mapping DOM events to node
  paths is simple, because the generated web renderer marks every node with `data-ui-path`
  (`research/2026-11-23-internal-building-blocks.md` § 1). A recorder that logs `(time, event,
  data-ui-path)` replaces rrweb's DOM serialisation; its size is about one event listener per
  action kind [Inference].
- **The masking rule** of `SKILL.md` § 1 (structure-only for real users) then holds by construction:
  a path log carries no typed text unless the log adds it.

## 6. The screen-only bot pattern

### Mechanism (as the skill and its references describe it)

- **What the bot may use.** Deterministic code, no model. It may use only controls that are fully
  in the viewport, not covered at their centre, and found by accessible role and name
  (`plugin/skills/ui-iteration-loop/SKILL.md` § 7).
- **What it counts.** Actions, glances and steps, which become `effort.actions`,
  `attention.glance_cost` and `completion.journey_completes` (`docs/metrics.md` §§ 3-5).
- **How it is built today.** On Playwright, "Playwright accessibility locators"
  (`references/not-available.md`).
- **What role lookup actually checks.** `getByRole` finds by ARIA role and accessible name. The
  filter is "hidden for ARIA", not geometry (`PW packages/injected/src/roleSelectorEngine.ts#L160-L167`,
  `isElementHiddenForAria` at `PW packages/injected/src/roleUtils.ts#L314`). The role and name code
  is 1,396 lines (`roleUtils.ts`).

### What goes wrong

- [Inference, from § 3] A bot that calls `locator.click()` gets help a person does not get:
  - Playwright scrolls the target into view;
  - it climbs out from under sticky bars;
  - it waits for animations.

  The skill's rule ("only controls fully in view, not covered") is therefore enforced only if the
  bot computes `toBeInViewport({ratio: 1})` plus a hit test itself before each action, and counts
  each scroll it needs as an action. Neither is a Playwright default.
- **Other published bot work** is an LLM-driven agent, or randomised keyboard exploration with
  temporal-logic monitors: 31 generated sites, 274 of 309 flagged failures confirmed, 88.7 %
  precision. Source: "A11yLTLNav: Automatic Detection of Accessibility Navigation Failures",
  https://arxiv.org/abs/2609.17959 (abstract read). It supports checking focus and state
  properties over action traces, which is what a walker on a declared model can do exhaustively.

### Owning it on a declared model

- **The walker's view of a state** is the set of nodes with:
  - `vis = 1`;
  - no `cov`;
  - an action, a role and a name.
- **Its moves:** pick a target by (role, name) as the journey asks; if the target is not in view,
  scroll by one viewport, which costs one action; then act through `App` and record the step.
- **What it needs.** Structure for the choice, cells for "in view and not covered". The keyboard
  variant needs the focus model of § 8.
- **What it gives up.** Nothing of the pattern, and it gains exhaustiveness: every journey, every
  gating environment, every state on the path, and no timing.
- **Size.** [Inference] 800 to 1,500 lines of Rust on top of `App` and the visibility predicates of
  § 3. ESS gaps that block it are listed in `docs/state-space.md` § 8:
  - a successor function;
  - a state digest;
  - Tab focus.

## 7. Contrast: WCAG 2.x against APCA

### WCAG 2.x

- **Formula.** Contrast = (L1 + 0.05) / (L2 + 0.05), with L the relative luminance of linearised
  sRGB. axe's luminance code uses the sRGB threshold 0.04045 and exponent 2.4
  (`AX lib/commons/color/color.js#L220-L228`).
- **Limits:**
  - 4.5:1 for normal text and 3:1 for large text (1.4.3);
  - 7:1 and 4.5:1 (1.4.6, AAA);
  - 3:1 for UI components and graphics against adjacent colours (1.4.11).

  Source: https://www.w3.org/TR/WCAG22/ .
- **What it needs:** the foreground, the effective background after alpha compositing, and font size
  and weight (for "large"). It is symmetric in polarity, and it is known to overrate contrast on dark
  backgrounds. [Inference: the latter is the stated motivation for APCA; I did not measure it.]

### APCA

- **The reference code.** `APCAcontrast(txtY, bgY)` in
  `https://github.com/Myndex/apca-w3/blob/6c5c066cf0c853507930a4239b129283b8e05353/src/apca-w3.js#L191-L270`.
  The constants are at `#L148-L172`:
  - Y from sRGB with exponent 2.4 and coefficients 0.2126729, 0.7151522, 0.0721750;
  - a soft clamp of near-black at 0.022 with exponent 1.414;
  - different exponents for the two polarities: text 0.57 and background 0.56 for dark text on
    light, text 0.62 and background 0.65 for light text on dark;
  - a scale of 1.14 and an offset of 0.027;
  - output clipped below 0.1.
- **Output.** A signed lightness contrast Lc, about −108 to 106. Thresholds depend on font size and
  weight through a lookup table (`fontLookupAPCA`, `#L449-L609`).
- **Status.** The WCAG 3 Working Draft of 10 September 2026 does not name a contrast method. It
  says "The contrast algorithm used in WCAG 3 is yet to be determined" (Guideline 2.2.1, Text
  contrast sufficient (minimum), https://www.w3.org/TR/wcag-3.0/). WCAG 2.2 1.4.3 stays the
  normative test.
- **Licence.** `apca-w3` is not open source in the usual sense. Its `LICENSE.md` says "All Rights
  Reserved. Patent(s) pending", licenses the files to the W3C "for use with WCAG accessibility
  guidelines … only", puts other uses under AGPL v3, and limits use of the name "APCA" to compliant
  implementations
  (`https://github.com/Myndex/apca-w3/blob/6c5c066cf0c853507930a4239b129283b8e05353/LICENSE.md`).
  [Inference] Before we ship any implementation, even our own port of the formula, we need a decision
  on the licence and the name. It is not a technical question.

### Catches, misses, determinism

- **The browser problem is the background, not the formula.** axe gives up ("can't tell") on 16
  kinds of background uncertainty (§ 1).
- **A declared model has no such uncertainty:**
  - text colours and surfaces are tokens;
  - alpha is composited over the declared surface stack;
  - font size and weight come from the type tokens.

  Contrast becomes exact and static, per theme and mode (`a11y.contrast_min`, `docs/metrics.md`
  § 8).
- **Images and gradients** under text stay undecidable on a declared model as well. Such a node is
  "insufficient evidence", as in `docs/state-space.md` § 7.

### Owning it

- **WCAG 2.x from tokens.** About 150 lines for luminance, ratio and limits, plus about 150 for
  resolving (text token, surface token stack, size, weight) per node kind and tone. It needs only
  structure plus the layout model's region nesting, to know which surface a text sits on: cells are
  enough.
- **APCA.** About 100 lines of code. Licence decision first. If shipped, then as a diagnostic, never
  as a gate, since no standard requires it.

## 8. Tab order and focus checks

### Mechanism in today's tools

- **No tool computes a tab order.** Playwright tests press Tab and read `document.activeElement`.
- **axe has only structural focus rules:**
  - `tabindex` (a positive tabindex is a best-practice failure);
  - `aria-hidden-focus` (a focusable element inside `aria-hidden`);
  - `nested-interactive`;
  - `scrollable-region-focusable`;
  - `frame-focusable-content`;
  - `focus-order-semantics`, which is experimental.

  None is tagged 2.4.3, 2.4.7, 2.1.2 or 2.4.11 (§ 1).
- **Lighthouse lists the order and trap checks as manual audits with weight 0** (§ 2).
- **What the browser does.** Elements with a positive `tabindex` come first in ascending order, then
  focusable elements with `tabindex` 0 or none in tree order. Inert, disabled and hidden elements
  are skipped. Script can move focus at any time.
  [Inference] That last point is why tools cannot predict focus without running the page.

### Catches, misses, determinism

- **The browser path catches** focus problems only when a test walks Tab through every state.
- **Timing:** focus moved by script after an animation makes walks timing-dependent.
- **Focus visibility (2.4.7) and focus appearance (2.4.13)** need pixels in a browser: the focus
  ring is drawn by the user agent or by CSS.

### Owning it on a declared model

With a declared focus model (ESS today drives the terminal by letter keys, not Tab:
`docs/state-space.md` § 4), per state s:

- **F(s), from structure.** The ordered focus stops: the actionable nodes of the open overlay,
  otherwise the page.
- **Keyboard unreachable, from structure.** An interactive node on a journey that is not in F(s)
  (2.1.1).
- **Focus trap, from structure plus the action graph.** A strongly connected set of focus states
  with no edge out except completion, with Esc counted as an edge out (2.1.2): a graph search.
- **Focus order, from cells.** Kendall tau between F(s) and reading order (top, then left, within a
  region) (2.4.3, as `a11y.focus_order`).
- **Focus not obscured, from cells.** A focus stop with `vis = 0`, or covered by a sticky region or
  an overlay (2.4.11).
- **Focus visible, from px or pixels.** It needs the focus style as a token and a rule that its
  contrast against the surface is at least 3:1 (WCAG 2.4.13's measure, AAA). The token rule needs
  only structure; whether the ring is actually drawn needs pixels.
- **Size.** [Inference] About 400 lines of Rust for F(s), reachability, the trap search and order
  distance, once ESS exposes Tab order and a successor function. Lighthouse calls these checks
  manual; on a declared model they are exact. Of all the checks here, owning this one gains the
  most over the tools.

## 9. Ranked table

| rank | check | needs | value | cost to own (Rust, [Inference]) |
|---|---|---|---|---|
| 1 | accessible name present on every action, input, image, link and icon (the four "empty name" failures of the WebAIM Million) | structure | high: four of the six most common failures on the web | S, about 150 lines on the rule engine |
| 2 | contrast WCAG 2.x per text-on-surface pair, from tokens, per theme and mode | structure plus tokens and region nesting | high: 83.9 % of home pages fail it | S, about 300 lines |
| 3 | always visible: `vis = 1` and not covered, for declared nodes at every state | cells | high: hard gate 2 | M, about 400 to 600 lines shared with 4 and 7 |
| 4 | no overflow, no scroll, no clipping where not declared truncatable | cells | high: hard gate 1, WCAG 1.4.10 at 320 px | S on top of 3 |
| 5 | keyboard reachability, focus trap, focus order against reading order | structure plus a focus model; cells for order | high: no tool automates these | M, about 400 lines; blocked on ESS Tab order |
| 6 | screen-only walker completes every journey; counts actions, scrolls and glances | structure plus cells | highest: hard gate 4 and the effort KPIs | L, 800 to 1,500 lines; blocked on ESS successor and digest |
| 7 | focus not obscured (2.4.11) | cells | medium | S on top of 3 and 5 |
| 8 | structure rules: heading order, one main, label matches visible name (2.5.3), duplicate sibling names | structure | medium | S, about 40 lines each |
| 9 | layout shift per transition | cells | medium | S, about 100 lines |
| 10 | action and state log with replay (rrweb's job) | structure plus the virtual clock | medium: evidence and regression input | S, 200 to 400 lines |
| 11 | target size and spacing (2.5.8) | px | medium | S once a px model exists; meaningless on cells |
| 12 | focus visible and appearance | px plus tokens; pixels to prove | medium | S as a token rule; pixels for proof |
| 13 | own-raster diff (byte equality plus differing node paths) | pixels from our own raster | low as a gate, medium as a review aid | M, needs the rasteriser |
| 14 | tolerant browser-screenshot diff (HyAB, anti-aliasing, SSIM) for renderer fidelity | browser | low to medium: fidelity only | M, 500 to 700 lines plus a PNG decoder |
| 15 | APCA Lc | structure plus tokens | low: not normative | S in code; licence decision first |
| 16 | ARIA validity rules (allowed attributes, required parents) | structure | low on ESS: roles are emitted by ESS itself; test the web emitter once | S |
| 17 | real fonts, real screen reader output, platform best practices | browser or OS | medium | not ownable; stays in `ux-walk@1` |

## 10. Build these five first

1. **The structural ACT rule set**, outcomes passed, failed, cantTell and inapplicable, with
   accessible names first (rank 1, plus rank 8 as the rule engine allows). It runs in stage 1 on the
   document alone and prunes variants before any layout.
2. **WCAG 2.x contrast from theme tokens** (rank 2). Static, exact, per theme and mode. It replaces
   axe's 16 "can't tell" background cases with one resolved surface.
3. **Visibility predicates on cells**: `vis`, `cov`, overflow and clipping per node path and state
   (ranks 3 and 4). They give hard gates 1 and 2 on `layout-model/0` today, and every later check
   builds on them.
4. **The focus model checks**: reachability, trap, order and focus not obscured (ranks 5 and 7). The
   largest gain over today's tools. They need ESS to expose Tab order, so raise that first
   (`docs/state-space.md` § 8 item 3).
5. **The screen-only walker** with honest rules (rank 6): it acts only on `vis = 1` and uncovered
   targets, every scroll is an action, there are no retries and no waits. It depends on 1, 3 and 4
   and on ESS's successor function and state digest.

Deferred: pixel diffs, until our own rasteriser exists; target size, until the px model exists;
APCA, until the licence decision.

## 11. Findings that change the plan

- **Most of axe exists to rebuild facts a declared model already has.** 86 of 105 rules read no
  geometry directly. The geometry rules spend their code on stacking contexts, a spatial grid and
  background stacks, and fall back to "can't tell" when the rendered page is ambiguous.
  [Inference] Owning these checks is cheaper than porting axe, and the results become exact instead
  of "incomplete".
- **Playwright's primitives do not mean what the gates mean.**
  - `toBeVisible` ignores the viewport, occlusion and clipping.
  - `toBeInViewport` ignores occlusion.
  - `click` scrolls, climbs out from under sticky elements and retries for up to the timeout.

  [Inference] A screen-only bot on Playwright defaults overstates what a person can do. The own
  walker must define visibility itself, and so must `ux-walk@1` until then.
- **Focus order and traps are manual in every tool read here** (Lighthouse's manual audits; no axe
  rule for 2.4.3, 2.1.2 or 2.4.11). [Inference] On a declared model with a Tab focus model they are
  graph checks. Of all the checks, owning these gains the most over the tools.
- **APCA cannot be adopted as a plain formula.** It is not in the WCAG 3 draft, and its reference
  code is "All Rights Reserved, patents pending" with restrictions on use and on the name.
- **With our own rasteriser, a pixel diff becomes exact.** But it then tests our own renderer, so it
  moves from gate to review aid. A tolerant comparison of real browser screenshots stays only as a
  fidelity probe of the layout model against the browser.
