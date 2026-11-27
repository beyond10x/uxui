---
format: aep.planning-md/3
id: story:docs-site-phone-and-sidebar
kind: story
status: implemented
title: The documentation site passes its own checks at the phone size and at a page's bottom
relations:
- decomposes: epic:targets
scope:
- confidence: inferred
  path: engine/browser/tests/fixtures/checks/uxui-docs.checks.yaml
- confidence: inferred
  path: website/src/css
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-27T09:57:01Z", actor: "human:operator", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-27T09:57:01Z", actor: "human:operator", revision: 6}
- {from: "active", to: "implemented", at: "2026-11-27T14:02:41Z", actor: "human:operator", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The documentation site passes its own page checks at the phone size and at the bottom of a page.

## Context

`ux-browser` 0.11.0 on the published page `/uxui/docs/reference/metrics` with this repository's
check configuration (2026-11-26): 38 checks, 7 failed. At 390x844 the theme toggle is not found at
the top or bottom of the page in either mode (4 misses), and the keyboard walk does not reach it
(1 miss); it sits in the collapsed menu. At 1440x900, scrolled to the bottom, the sidebar's box
starts at y -222, so its top corners are out of view (2 misses).

## Acceptance

- At 390x844 either the theme toggle is reachable in view and by keyboard, or the configuration
  declares it with a `max_width` and the document says why the phone menu holds it; the run shows
  0 always-visible and 0 keyboard misses there.
- At 1440x900 the sidebar stays wholly in view at the bottom of a long page; 0 misses.
- The site's checks run on the built site in the gate's docs step with these results.
