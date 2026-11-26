---
format: aep.planning-md/3
id: story:campaign-terms-for-list-pages
kind: story
status: draft
title: Campaign terms that separate variants of a list page
summary: rows in view, a primary-action term that does not punish a smaller control, always-visible inside a scroll container
relations:
- decomposes: epic:targets
- depends_on: story:trial-target
revision: 2
---
# Story: Campaign terms that separate variants of a list page

## Outcome

A declared campaign on a list page ranks its variants by what a list page is for: rows in view, and
the place of its primary action, measured so that a smaller control is not ranked worse.

## Context

The first run of `ux-search campaign` on a page outside this repository (2026-11-26, a list page with
one create action in its header; `story:trial-target`) ended with a ranked frontier, and showed what
the catalogue of terms cannot say for such a page:

- `primary_px` is the top edge of the primary action. The action is centred in the page header, so a
  tighter control height moved its top edge from 24 to 26 px and the term ranked the tighter variant
  worse, although nothing moved down the page.
- `regions` and `primary_px` are trivial when the action sits in the page header: they are the same
  in almost every variant.
- Rows in view, the natural term for a list page, does not exist; a spacing axis shows up through
  `primary_px` only.
- `always_visible` cannot say "fully visible, or clipped on purpose inside a scroll box": a list in a
  navigation that is taller than the viewport counts as a miss.
- The ink of a disabled control is never judged.

Not in this story: the pointer position, the read-back of a mode set by query, `--help`, the default
profile directory, notes per term (built as a gap closure in the third wave of 2026-11-26).

## Acceptance

On a fixture list page in `engine/search/tests/fixtures/campaign/`, a campaign with a spacing axis:

- has a term `rows_in_view` over a declared selector, and the variant with the tighter rows has the
  higher count and ranks before the looser one on that term;
- measures the primary action so that a variant that only shrinks the control's height does not rank
  worse than the page as shipped on that term;
- accepts an `always_visible` item that names its scroll container, and counts no miss for an element
  that is clipped only by that container;
- each of the three is held by a test that failed before the change.

## Provenance

Drafted on 2026-11-26 from the generic findings of the first run in the private repository
(`story:trial-target`, units 2 and 3).
