# ESS gaps uxui needs, in priority order

Status: 2026-11-26. Every gap uxui has found in ESS, ordered by what blocks the engine first. Found on ESS
0.98.0; read again on 2026-11-26 against ESS's releases up to 0.99.5 (the newest, and the tag
`engine/search` builds on) and its planning store on its `main`; the CLI installed here is 0.99.2. The
owner, 2026-11-24: "make sure that all ESS gaps are implemented ASAP". Sent
to the ESS maintainers on 2026-11-24 as gap reports; ESS filed them as stories of
its own, tagged `gap-reports-2026-11-24` and `gap-p0`, `gap-p1` or `gap-p2`, and maps each item to them in
its `.engineering/waves/gap-reports-2026-11-24.md`. Below, "ESS:" names those stories by file under
`.engineering/planning/story/` of the ESS repository, with the status read on 2026-11-25; no status of an
item's story had moved on 2026-11-26, and none is `implemented`. The requirements of the generator round
that ESS has no story for yet are draft stories uxui sends ([Needs per way](#needs-per-way)). Styling is
not an ESS gap: themes and templates belong to uxui (`AGENTS.md`, Direction), and so do the UX declarations
(`uxui-declarations/1`).

Closed: 1 of the 18 items (item 9), and two needs of the generator round (the loop period and
`insert_top` of a held key, [Needs per way](#own-generator)). A closed entry says **Closed**, the ESS
release that closed it and the uxui change that adopted it. The other 17 items are open in ESS 0.99.5:
no story of theirs is in a release, and item 6 is on ESS's unreleased `main`. Releases 0.99.1 to 0.99.5
are patch releases: they change conformance synthesis and, in 0.99.2, where `ess ui check` and the
renderers read an overlay's `params:`; none adds a key to `ess-ui/1` or to the specification format.

Sources: `ess/GAPS.md` (G1-G11), `docs/roadmap.md` step 2, `docs/generator-round.md` (F1, F3, stories 4
to 7), `docs/change-list.md` (C1-C10), `docs/own-generator.md`.

## P0: without these the engine cannot exist

ESS accepted all of P0 (status `active`, tag `gap-p0`) and builds it in eight rounds, R1 to R8; R5 to R8
ship as one release, which publishes `ess-ui-resolved/1` and `ess-ui-state/1` once, complete. None of it is
in a tag yet (read at 0.99.5).

1. **UI state API** (roadmap 2a). For a UI state of an `ess-ui/1` document: the list of actions available
   in it, and a cloneable state (or a replay-free way to reach it). Exhaustive search over the state space
   is impossible without it. ESS:
   `a-design-page-fixes-how-a-program-explores-the-ui-states-of-an-ess-ui-1-document-one-session-the-steps-a-state.md`,
   `a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in.md`
   (`g01-app-forks`),
   `a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test.md`
   (`g01-session-actions`),
   `a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the.md`,
   `ess-ui-state-prints-the-state-a-list-of-ess-ui-test-1-steps-reaches-and-the-steps-that-state-offers.md`.
   The actions are `ess-ui-test/1` steps (`Session::offered`). ESS meets "replay-free" by forking a running
   session in memory (`Session::fork`); nothing restores an app from an `ess-ui-state/1` document, which is
   printed, not read back.
2. **Canonical form and digest** (G7, roadmap 2b). A normative canonical byte form and digest of an
   `ess-ui/1` document and of a UI state. "Two routes to one document are one variant" and the evaluation
   cache both rest on it. ESS:
   `an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling.md`
   (`g02-document-digest`), and for the state
   `a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the.md`.
3. **Resolved document** (F3). `ess ui load --format json` printing `ess-ui-resolved/1` with every default
   filled in. ESS took the recommendation: the loader fills every default, so every generator, stock or
   own, reads the same input. ESS:
   `ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format.md`,
   `with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and.md`
   (the model bindings), and the fill units
   `the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from.md`,
   `a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test.md`,
   `with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows.md`,
   `the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once.md`,
   `the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema.md`.

## P1: needed for evidence and for the studio

4. **Per-step trace** (roadmap 2c). An `ess-ui-test` report with one record per step: the state reached,
   the nodes shown, the geometry read. The engine's evidence and the bench's coverage count read it.
   ESS, follow-up stories outside P0:
   `a-traced-ess-ui-test-run-reports-for-every-step-the-page-shown-the-nodes-drawn-and-the-cells-each-one-occupies.md`
   (`active`),
   `each-step-of-a-traced-ess-ui-test-run-names-the-ui-state-it-reached-by-the-digest-of-that-state-s-serialised.md`
   (`draft`, after P0).
5. **Component harness** (G11, roadmap 2d). Render one component alone at given widths, with fixtures.
   Targets below page scope need it. ESS:
   `a-test-opens-one-section-overlay-or-widget-of-a-document-alone-at-a-given-terminal-width-and-the-terminal.md`,
   `the-generated-react-project-draws-one-section-overlay-or-widget-alone-at-a-given-width-and-the-playwright-spec.md`
   (both `draft`).
6. **Chart order** (F1). `chart` takes `sort: {by, dir}` and live inserts keep the order. Done on ESS's
   unreleased `main`, commit 1d208b2225, in no tag yet. ESS:
   `a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order.md`; a test
   step that asserts a chart's order:
   `a-chart-s-rows-are-addressable-in-both-renderers-and-a-test-asserts-the-order-a-chart-draws-them-in-with.md`
   (`active`).
7. **Param-scoped live channel in tests.** A channel with `scope: param` and `session: {per: campaign_id}`
   delivered no scripted event in `ess ui test` (terminal renderer); the studio works around it with
   `only_if: matches(params)`. A defect or a missing feature; not investigated further. ESS:
   `a-per-value-channel-script-that-names-no-session-is-reported.md` (`active`).
8. **A view read through a list field.** A view with a parameter over a row's list field (a variant's
   routes), so the studio can show the axis values a variant was reached by. ESS:
   `from-ess-39-a-view-s-filter-reads-the-row-a-parameter-names-one-hop-and-walks-that-row-s-list-fields.md`
   and two units after it (`draft`).
8a. **Tab focus order** (added 2026-11-24). The terminal renderer exposes no keyboard focus order of a UI
   state. Without it focus reachability, focus traps and focus order against reading order (WCAG 2.4.3, 2.1.2, 2.4.11) cannot be
   checked; no existing tool automates them, and on a declared model they are exact graph checks.
   ESS, follow-up stories outside P0:
   `the-keyboard-focus-order-of-a-ui-state-is-declared-by-the-document-and-read-from-a-traced-test-run-the.md`,
   `the-terminal-s-tab-follows-the-declared-focus-order-and-a-test-presses-tab-and-asserts-the-focused-node-in.md`,
   `the-generated-react-project-s-tab-order-is-the-declared-focus-order-and-an-open-overlay-keeps-focus-inside-it.md`
   (all `draft`). ESS exposes the order; the reachability, trap and order checks stay uxui's.

## P2: invariants the specification can only state by construction today

ESS maps items 9 to 17 to stories in its wave page named in the status above, section "P2, first report".

9. **For all over related rows** (G4). "Every Active hard rule of the target has a `pass` verdict."
   **Closed.** ESS release: 0.63.0, which let a row-set `when_related:` test each candidate's related
   rows in its `where:` (`exists: {in: {entity: …}, as, that}`), so the rule reads "no Active hard rule
   of the target without a Recorded `pass` verdict of this variant"; the gap report carried an earlier
   text of `ess/GAPS.md`, which said it could not be stated. uxui change: pull request #1 (the refusal
   `hard-rule-unchecked` of `AcceptPromotion` and `AcceptTradeOff` in `ess/domains/promotion.yaml`;
   `ess/GAPS.md`, G4), recorded here on 2026-11-26.
   ESS keeps a story for the reference text and a test of the idiom:
   `every-row-of-a-selection-has-a-related-row-that-passes-is-stated-with-a-row-set-guard-and-a-rows-quantifier.md`
   (`draft`).
10. **A list-reference guard beside another related guard** (G1).
11. **A related-row guard beside a held-state guard** (G3).
12. **A secondary effect beside a create** (G2).
13. **A guard two relations away** (G8).
14. **Comparison across list elements of two rows** (G5), for Pareto dominance.
15. **A digest computed from named inputs** (G6), for "evaluation is a pure function of its inputs".
16. **A "never reads" invariant across domains** (G10), for "a judgement never changes a gate verdict".
17. **Predicates over an `ess-ui/1` document** (G9), for mutation operator preconditions.

## Needs per way

Story 7 of the generator round (`docs/generator-round.md`): what `ess-ui/1` needs for the stock generator
and for an own generator, drawn from stories 4 to 6 (the own generator, `docs/own-generator.md`; the change
list, `docs/change-list.md`; slot's case study). Each requirement names where it stands: an ESS story (file
and status, 2026-11-25), a story ESS filed from uxui's drafts D1 to D5 (2026-11-25 15:33 CET, ESS branch `plan/gap-reports-2026-11-24` at 44848bca2b, not on ESS main; none
accepted or scheduled, a new `ess-ui/1` key waits for the owner's read), uxui's own work, or no request, with the reason. The letters G1 to G13 and F18 in
this section are slot's (its `docs/design/ess-ui-gaps.md`), not those of `ess/GAPS.md`.

### Stock way: extension points

- **Host slot content in a test** (C3, own-generator gap 5). The terminal draws a host slot's reason, so a
  test on a host-drawn cell cannot pass there. ESS:
  `a-terminal-test-checks-what-a-host-slot-is-handed-its-resolved-reads-its-commands-and-the-page-s-parameters.md`
  (`draft`): a test checks what the slot is handed; the content stays the application's to test.
- **One host component at several places** (C3, slot G7). `host` names are unique, so an identical column
  on a second page needs a second slot and a second registration. ESS:
  `one-host-component-stands-in-many-slots-that-receive-the-same-thing-each-slot-identified-by-its-node-path.md`
  (`draft`), after `a-host-slot-in-a-reused-template-is-one-registration.md` (`active`).
- **Generator options.** 0.98.0 has no `ess generate ui` option that changes what is drawn. No request:
  every fork the change list needed is a format story in this file (C1, C3, C4, C10) or uxui's theme work
  (C2, C5). C8 (a bespoke widget) is a section host slot and needs nothing new.
- **Loading a host stylesheet and script** (C2: the link line in the generated `www/index.html` is a fork).
  uxui's own: theming. On the stock way a uxui post-generate step links the theme (today a fork re-applied
  after each generate); the own generator takes `--theme`.

### Stock way: theming

- **Brand theme, series colours, light and dark modes** (F4, C2). uxui's own: `themes/studio.css` and
  `themes/theme-toggle.js`, over the stock app's `--ui-*` custom properties and `ui-` classes.
- **Names the generated app already uses** (F5: its runtime writes `data-theme` on `<html>`). uxui's own:
  the uxui theme keys its modes on `data-uxui-mode`; not asked of ESS.
- **slot's styling and layout** (slot G9: column widths, one-line cells, class hooks; its F9, a label
  beside its value; the colour per value of G11). uxui's own: `story:slot-styling-in-themes`. ESS declined
  both as styling (its decisions SL-D9 and SL-D10).
- **Hiding columns on a narrow screen** (C5). uxui's own: a media query in the theme. On the stock way the
  body cells carry no column mark, so the rule counts columns unless a uxui template adds column marks (the
  class hooks of G9); the own generator marks each cell.

### Stock way: per-node overrides and value rendering

- **Chart order** (F1, C1). Item 6 above.
- **A list of records as text** (C3, slot G4 and G5). The KPI readings print as JSON. ESS ends the JSON:
  `a-structured-value-reads-as-one-line-of-its-members-in-every-renderer.md` and
  `a-list-field-names-the-text-that-joins-its-items-in-every-renderer.md` (both `active`). The words the
  request asks for (`effort.actions 6`) need text composed from chosen members: D5, filed as a need of
  `a-ui-expression-joins-texts.md` (`draft`, its fit review owed).
- **A cell that opens a page** (C4). `as: link` takes the value as an external address and `act` on a cell
  is refused. D3, filed as `a-column-or-record-field-opens-a-page-of-the-document-and-a-test-acts-on-that-cell.md`
  (`draft`, accepted in its fit review; the key would be `navigate`).
- **Time since an instant** (C10). D4, filed as a need of
  `a-duration-field-shows-the-time-between-two-instants-of-its-row-the-clock-standing-in-for-an-absent-end.md`
  (`draft`), slot G10's story below; "20 s ago" wording is held by ESS as looks.
- **An explanation in a confirm** (C6). No request: the inline `confirm:` has no `body`, and a named
  confirm overlay with `body` already expresses it.
- **An integer id drawn with grouping** (slot G3). ESS:
  `a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer.md` (`active`).
- **A tagged union drawn as raw JSON** (slot G5). ESS:
  `a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object.md`,
  `generated-codecs-and-conformance-write-and-read-a-flat-union-in-the-layout-its-model-declares.md`
  (both `active`),
  `a-tagged-union-value-reads-as-its-variant-and-its-content-when-the-document-is-bound-to-its-model.md`
  (`draft`).
- **A badge's tone from its value** (the semantic half of slot G11). ESS:
  `a-page-header-s-badge-takes-its-tone-and-words-from-its-value-as-a-badge-does.md` (`draft`).

### Both ways: slot's format and model gaps

- **A view route reads a member of a larger answer** (slot G1). ESS:
  `a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their.md`,
  `the-generated-rust-and-go-servers-answer-a-route-shared-by-several-views-with-one-object-of-their-rows.md`,
  `a-generated-ui-reads-a-view-s-rows-from-the-member-its-route-declares.md` (all `draft`).
- **A `Binary64` literal in `sets`** (slot G2). ESS:
  `a-decimal-literal-sets-or-publishes-a-binary64-field-read-by-one-grammar-the-validator-and-every-target-share.md`
  (`draft`).
- **Local time beside UTC** (slot G6). ESS:
  `a-time-date-or-timestamp-field-declares-whether-it-is-drawn-in-utc-or-the-reader-s-local-zone.md`
  (`draft`).
- **A search over several fields** (slot G8). ESS:
  `one-search-reads-several-fields-each-word-found-in-any-of-them-documented-and-pinned-in-every-renderer.md`
  (`draft`).
- **A value derived from two fields and the clock** (slot G10). ESS:
  `a-duration-field-shows-the-time-between-two-instants-of-its-row-the-clock-standing-in-for-an-absent-end.md`
  (`draft`).
- **A read-only entity without invented commands** (slot G13). ESS:
  `an-entity-whose-rows-the-deployment-provides-declares-lifecycle-moves-the-deployment-makes-with-no-command.md`,
  `synthesis-reaches-every-state-of-a-provided-entity-through-its-provider-and-never-drives-a-move-the-deployment.md`
  (both `draft`).
- **An event stream no command opens** (slot F18). ESS:
  `an-event-channel-may-be-bound-to-server-sent-events.md` (`active`),
  `a-generated-ui-s-live-channel-connects-to-the-stream-route-its-binding-declares-and-reads-its-frames.md`
  (`draft`).
- **The resolved document** (slot G12, its F16). P0 3 above.

### Own generator

- **A resolved, versioned, machine-readable form of the document** (F3; about 960 of the own generator's
  1,296 re-implementing lines). P0 3 above, with the model bindings.
- **A conformance suite every generator is held to, from `ess-ui-test/1`** (the studio's own-generator
  decision rests on it). `ess ui test --playwright` writes a spec for the generated React project only, on
  names no contract publishes. D1, filed as
  `a-page-contract-and-a-release-corpus-hold-a-page-another-generator-writes-to-the-ess-ui-test-1-files.md`
  (`draft`, fit review: defer; the parts that need a new execution surface are the owner's).
- **Behaviour the renderers left open** (`docs/own-generator.md`, "Decisions ESS left open"), which that
  suite has to pin:
  - the loop period of a fixture script (own-generator gap 3). **Closed.** ESS release: 0.98.1 (the
    generated React app repeats a looping script at its last entry's time, as the terminal and
    `ess ui test` do). uxui change: none needed in code, the own generator took the terminal's period
    from the start; this change records it in `docs/own-generator.md`. ESS:
    `a-looping-fixture-script-repeats-at-its-last-entry-s-offset-in-the-browser-too-as-in-the-terminal-and-the-test.md`
    (`draft`; it names 0.98.1 as the release that carries it);
  - an event played as the page opens (gap 4). ESS:
    `in-the-browser-a-live-event-that-arrives-before-the-section-s-read-has-answered-is-applied-once-it-answers-as.md`
    (`draft`);
  - `insert_top` of a key already held (gap 1). **Closed.** ESS release: 0.98.1 (one row per key: the
    held row takes the event's fields and moves to the top, `max_rows` applies after the move). uxui
    change: this change, `apply` in `generators/own/assets/runtime.js`, which inserted the key again
    as ESS 0.98.0 did. ESS:
    `a-live-insert-of-a-key-the-section-already-holds-shows-one-row-in-both-renderers.md` (`draft`; it
    records the fix);
  - a confirm's button text and a field's label. ESS settled both in P0 3 (the button reads `Confirm`; a
    label is the name with `_` as a space):
    `the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema.md`;
  - `resume: refetch` in fixture mode, a `by_id_from` view with no matching row, where `insert_or_patch`
    puts a new key. D2: the first two filed as
    `the-reference-states-what-a-refetching-resume-keeps-in-fixture-mode-and-what-a-by-id-from-view-answers-without-a-match.md`
    (`draft`, accepted in its fit review); the `insert_or_patch` position is covered by
    `a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order.md` (`active`).
- **A check that a live event can fill the rows it inserts** (own-generator gap 2). ESS:
  `ess-ui-check-model-warns-when-a-live-event-lacks-a-field-the-section-it-feeds-shows-or-the-field-that-matches.md`
  (`draft`).
