# The page checks of the UI loop copies (2026-11-24)

Eight copies of the UI loop harness in seven repositories of github.com/beyond10x: `llm/ux`,
`codespec/ux`, `modelspec/ui/ux`, `runbook/ui/ux`, `slot/ui/ux`, `slot/website/ux`, `conductor/ux`,
`substrate/website/ux`. Read from their `baseline.json` and harness code on 2026-11-24.

## Which checks they share

- Horizontal overflow and page scroll: 5 of 8 copies, under 12 names (`layout.overflow`,
  `layout.page_scroll`, `layout.doc_hscroll`, `hscroll_pages`, `contract.scroll`, ...).
- Always-visible elements (navbar, search, theme toggle, sidebar): 5 of 8, 6 names.
- Layout shift: 6 of 8, 5 names and 4 aggregations (sum after ready, per 100 events, per minute, max).
- Keyboard reach (skip link first, skip link moves focus, tabs to targets): 4 of 8.
- axe-core violations by impact: 5 of 8 (a third-party library).
- Reading load: 4 of 8. Screen-only bot `screen_only.actions`: 8 of 8.

## How llm computes them (the first consumer)

Playwright Chromium, headless, reduced motion; gating viewports 1440x900 and 390x844, light and dark
(`prefers-color-scheme`, checked on `data-theme`); external requests aborted; values summed over
viewport x theme, layout shift the max (`llm/ux/core/results.ts:72-103`).

- `layout.page_scroll`: a unit counts 1 when `scrollingElement.scrollWidth -
  documentElement.clientWidth > 0` (`llm/ux/core/layout.ts:37-39`).
- `layout.overflow`: every non-blank text node (Range rect) and every `img, svg, video, canvas, iframe,
  input, select, textarea, table, pre`; overflow when the right edge is past the viewport + 1 px or
  past its nearest non-inline block ancestor + 1 px; skipped under `position:fixed` or `overflow-x`
  auto/scroll; at most one per block container (`layout.ts:65-119`).
- `always_visible.misses`: each item by role (CSS fallback) must be visible, wholly inside the viewport
  +-0.5 px and the `elementFromPoint` of its centre inside it; checked at scroll top and bottom
  (`layout.ts:128-180`).
- `keyboard.misses`: first Tab on a skip link, Enter moves focus to its target, then up to 40 Tabs
  reach search and navbar (`layout.ts:189-234`).
- `layout_shift.after_ready`: PerformanceObserver `layout-shift` without recent input, summed after
  `document.fonts.ready` plus two frames, read after 250 ms (`layout.ts:236-265`).
- axe: `@axe-core/playwright` 4.14.0, WCAG 2.0-2.2 A/AA tags.

Baseline: 27 page scenarios, all 0 on overflow, page scroll and always-visible; `keyboard.misses` 4 on
the search page (waived, finding UXF-005); layout shift 0.0004 to 0.0013 on four pages. Produced by
`task ux-check` (a built site in `website/build`, its own static server, Playwright's Chromium).

## Same name, different numbers

`codespec/ux/core` is byte-identical to llm's except a ready selector in the keyboard walk, but its
specs differ: a fresh context per theme (layout shift measured in both themes), the keyboard walk in
every theme (4 units per page, llm 2), full axe in dark, other settle times and viewports (1366x768,
1280x720). `modelspec/ui/ux` defines stricter hard gates (any scrollable element, 5-point cover test).
A shared implementation needs one definition per check and the viewports, themes and settle time as
declared inputs.
