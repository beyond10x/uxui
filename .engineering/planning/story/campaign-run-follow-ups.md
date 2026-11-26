---
format: aep.planning-md/3
id: story:campaign-run-follow-ups
kind: story
status: draft
title: 'Campaign run: refuse early, fold the notes, name an idle environment'
summary: what the second run on a page outside this repository left open in ux-search campaign
relations:
- decomposes: epic:targets
- depends_on: story:trial-target
revision: 2
---
# Story: Campaign run: refuse early, fold the notes, name an idle environment

## Outcome

A declared campaign refuses a wrong mode before it renders the space, its notes can be read, and the
run says which environments and which properties change nothing.

## Context

The second run of `ux-search campaign` on a page outside this repository (2026-11-26,
`docs/trial-2026-11-26-outside-target.md`, "Findings of the second run") confirmed the fixes of the
first run and left these:

- A refusal of a mode's `expect` came after 4.9 s, when the renders of the mode before it had run.
- `campaign report --notes` printed 205 lines beside a report of 65: each element once per
  environment and frontier variant, although it was the same in all of them.
- `text_under_14` is a share: it rose from 0.105 to 0.166 while its count fell from 4 to 3, because
  the state of the page changed the number of text runs.
- The run names an axis that changes nothing, and not an environment (a viewport, a mode) that
  changes no number; and not a property that changes nothing beside a live one in the same value.
- A run cannot be restricted to one variant or one environment.
- Screenshots differed by a few bytes between two runs of one page (6 of 24).
- `ux-search --version` was the same for two engines with different measurements formats, and the
  declaration gained keys under the unchanged name `ux-search-campaign/1`.
- The refusal of an attribute mode says "it sets the attribute itself"; a page that writes the
  attribute when it starts is measured, and only a later change is refused.

## Acceptance

On fixture pages in `engine/search/tests/fixtures/campaign/`, each line held by a test that failed
before the change:

- a mode whose `expect` the page does not show is refused before the first variant is rendered;
- `campaign report --notes` prints an element once when it is the same in every environment and
  variant it is printed for, and says for how many;
- the run prints one line for a mode or a viewport whose renders measure the same numbers as another
  one's for every variant;
- `campaign run` takes a restriction to one variant and to one environment, and a restricted run
  writes no measurements file that the report would rank as a whole campaign;
- two runs of one declaration write screenshots of the same bytes, or the document says which bytes
  may differ and a test holds the pixels;
- the refusal of an attribute mode says that the page changed the attribute after it was ready.

Decided in the story, not before it: whether `text_under_14` becomes a count, and whether a new key
of the declaration needs a new format name.

## Provenance

Drafted on 2026-11-26 from the generic findings of the second run in the private repository
(`story:trial-target`).
