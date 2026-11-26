# Trial 2026-11-26: a declared campaign on a page outside this repository

Status: run twice on 2026-11-26 (`story:trial-target`, roadmap step 6). The first campaign of
`ux-search` on a product UI it was not built around: one list page of an admin console in a private
repository, generated from an `ess-ui/1` document and served on its fixtures. This document holds
the generic findings only. The declarations, the measurements, the reports and the screenshots stay
in that repository, uncommitted; this repository does not name the product, the page or anything on
it.

The first run ended with a ranked frontier and measured the page in a state no visitor rests in.
Its findings about the tool were built the same day; the second run, with that engine, measured the
page at rest and ended with the same frontier.

## What ran

- The page: a list with two columns and one create action in the page header, on a build with
  fixture data and no backend, served on a loopback port by a static server with a single-page
  fallback. The fixture rows carry invented labels; the page was looked at before the run.
- The tool: `ux-search campaign run`, release 0.11.0 in the first run and the engine of this change
  in the second, with the browser adapter `ux-browser` 0.11.0 and HeadlessChrome/158.0.8127.18,
  2 pages at once.
- The declaration: 4 axes of custom properties that the theme declares in `:root` (spacing 4 values,
  an accent fill 2, the ink on the accent 2, the small type size 2), 32 variants; 6 environments
  (1366x768 and 1280x720, each light, dark set by a query, dark set by an attribute); hard rules and
  terms as the campaign on this repository's studio uses them. The second run's declaration adds
  what each dark mode expects of the page (`expect`) and nothing else.

## The two runs

| | first run | second run |
|---|---|---|
| state of the page | navigation open under the resting pointer | at rest: the pointer parked in the bottom-right corner |
| renders measured | 192 | 192 |
| run time, and of the run again | 9.4 s, 10.0 s | 10.9 s, 10.7 s |
| the run again | the same bytes of measurements and report | the same bytes of measurements and report |
| text runs counted | 38 | 18 |
| `overflow` | 1 | 0 |
| `always_visible_misses` | 1 | 0 |
| `low_contrast` | 3 or 1 | 1 or 0 |
| `text_under_14` with the theme's small type | 4 runs, 0.105 | 3 runs, 0.166 |
| `primary_px` | 24 or 26 | 24 or 26 |
| `small_targets` | 3 | 3 |
| variants that break a hard rule | 0 | 0 |
| frontier | 6 variants, 1 set of terms | the same 6 variants, 1 set of terms |
| ranking | 32 variants, 8 sets of terms | 32 variants, 8 sets of terms |
| the page as shipped | rank 28 of 32, off the frontier | rank 28 of 32, off the frontier |
| requests | 8, all `GET` to the page's origin | the check on requests passed |

In both runs `low_contrast`, `primary_px` and `text_under_14` separate variants, and `overflow`,
`always_visible_misses`, `small_targets` and `regions` do not. All 6 environments measure the same
numbers for every variant: on this page neither the viewport nor the mode moves a term.

What the frontier says of the page, from the second run: the six variants keep the theme's spacing
or widen it, change one of the accent's fill and its ink (the label of the primary action has a
contrast of 1.95 as shipped), and set the small type to 14 px. The tight spacing ranks worse for the
reason of finding 4, not because the page got worse.

A run of the same declaration with `page.pointer: [0, 0]` measured the first run's numbers again
(38 text runs, `overflow` 1, `always_visible_misses` 1): the first run's state was the hover state.

## Findings of the first run, and what became of them

1. **The page was measured in a hover state.** The headless pointer rests at (0, 0). The page's
   navigation opens when the pointer enters it, so every render measured the page with the
   navigation open over the title, the first column and half of the filter field. Two terms existed
   only in that state. No check caught it: the state was the same in all of the tool's page loads.
   *Built: the run parks the pointer before every measurement and screenshot (`page.pointer`, else
   the bottom-right corner). Holds in the second run.*
2. **A mode set by a query was not read back**, and screenshots were written for the first and the
   last environment only. Light and dark measure the same numbers on this page, so a mistyped query
   would have measured light twice without a sign. *Built: a mode's `expect` is read back in every
   render, and every mode gets a screenshot. Holds: a mistyped query was refused with the mode, the
   environment and the value the page had.*
3. **"In force" was true for a property that nothing reads.** The theme declares spacing properties
   in `:root` that no rule of this page consumes; a second, hand-written layer of the style sheet
   sets its own sizes. Finding out which properties are live took a probe of our own. *Built: the
   run says per axis how many values change a box or a computed style. Holds per value: an axis of
   the dead property alone is named; a dead property beside a live one in the same value is not
   seen.*
4. **`primary_px` ranks a smaller control worse.** The primary action is centred in the page header,
   so a tighter control height moved its top edge from 24 to 26 px. *Open:
   `story:campaign-terms-for-list-pages`.*
5. **Counts without names.** One `low_contrast` run in every variant was a text run with fully
   transparent ink, where an icon stands in for the character; the report named no element for any
   count. *Built: transparent ink is not judged, and `campaign report --notes` names the first
   elements per count. Holds: the notes were enough to find each element (the primary action's
   label; a disclosure icon of 18 x 18 px and two sort buttons of 16 and 17 px height as the three
   small targets).*
6. **What the catalogue cannot say for a list page.** Rows in view; "fully visible, or clipped on
   purpose inside a scroll box" for an always-visible element; the ink of a disabled control.
   `regions` and `primary_px` are trivial when the action sits in the page header. *Open:
   `story:campaign-terms-for-list-pages`.*
7. **The documents, followed from nothing.** `--help` answered with an error; the run line lacked
   `--jobs` and `--profile`; without `--profile` the browser profile went to `target/` under the
   working directory; the state of the browser while it measures was written nowhere;
   `measurements.json` embeds the declaration and the report prints the page's path. *Built:
   `--help`, a profile under the system's temp directory, and in `docs/targets.md` the state of the
   browser, what a private target's files hold and a checklist. Holds: no directory appeared in the
   working directory.*
8. **What a foreign repository needs before the first render**: a build with fixture data and no
   backend; the build's output and the app's packages on the machine that serves it; a static server
   with a single-page fallback; a free loopback port read from the server's log, and a stop by
   process id; a look at the page and its data before the run. *Written into the checklist.*

What worked without a change in both runs: the ready selector with a settle time of 0 (the fixtures
are compiled into the bundle), the guard on requests, the attribute mode, two runs with the same
bytes of measurements.

## Findings of the second run

All open, in `story:campaign-run-follow-ups` unless another story is named.

1. **A refusal of `expect` comes late.** The mistyped query was refused after 4.9 s, when the other
   page in flight had finished the 32 renders of the light mode. One load per mode before the
   campaign starts would refuse at once.
2. **The notes repeat.** 205 lines of notes beside a report of 65: each small target is printed 42
   times, the same in every environment and frontier variant. There is no line "the same in every
   environment" and no difference against the page as shipped.
3. **A share moves with the state of the page, a count does not.** `text_under_14` went from 0.105
   to 0.166 while its count fell from 4 to 3, because the navigation's texts left the denominator.
4. **An environment that changes nothing is not named.** Modes and viewports cost 6 times the
   renders here and moved no number; the run says so for an axis and not for an environment.
5. **A run cannot be restricted** to one variant or one environment; a check of one state needs a
   cut-down declaration.
6. **Screenshots are not the same bytes on a second run** on this page: 6 of 24 differ by 2 to 21
   bytes of file size. Measurements and report are the same bytes.
7. **The version does not tell the engines apart.** Both binaries print `ux-search 0.11.0`, while the
   measurements format went from `/1` to `/2` and the declaration gained keys under the unchanged
   name `ux-search-campaign/1`.
8. **Two sentences of `docs/targets.md` were wrong** when followed: the one on focus (the adapter
   turns focus emulation on) and the one on a page that writes the mode's attribute itself (it is
   measured; only a later change of the attribute is refused). Both are corrected with this document;
   the refusal's own words in the engine still say "it sets the attribute itself".

## What one had to know about the app

None of it is in the tool's documents, and each item decided a line of the declaration:

- The theme does not follow `prefers-color-scheme`: the app writes a root attribute from a query,
  else from a stored choice, and writes it again after mount.
- The style sheet has a generated layer and a hand-written layer; which `:root` properties are live
  differs per page.
- The dark block has the specificity of `:root`, so a later `:root` rule wins in dark too; the accent
  and its ink are not set per mode.
- The navigation opens on pointer-enter and on keyboard focus.
- The list reports its state in an attribute, and elements carry stable path attributes that serve
  as selectors.
