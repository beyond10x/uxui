# The UX search engine: concept

Status: design, 2026-11-23. Terms as in `docs/brief.md` § Glossary. Companion documents:
`state-space.md` (the two spaces, the layout model, search), `metrics.md` (the catalogue),
`ratchet.md` (how results become ratchet evidence and gates). "[Inference]" marks a claim no source
states. Paths starting `ess/website/` or `ess/crates/` are in the ESS repository (github.com/beyond10x/ess), not in
this repository's `ess/` specification; `canon/` and `ratchet/` paths are in those repositories.

## 1. The problem with the current loop

The loop of the `ui-iteration-loop` skill and the colleague's loop it was rebuilt from measure one
hand-built UI in a browser, scenario by scenario.

| defect | evidence |
|---|---|
| Slow | R-01: browser runs 1.8 min and 50.8 s, queue waits 30, 12, 18 and 20 min for the shared build pool (`reviews/2026-11-23-r01-explainer-round.md:15`). The colleague's full theme matrix takes 20-30 min (private comparison with the colleague's repository). |
| Bound to recorded scenarios | Hermetic replays cover only the states someone recorded (`reviews/2026-11-23-ui-iteration-loop-critique.md:40`). Novel failures show only in live rounds. |
| Real I/O | Each check starts a browser, a dev server and network routes; fonts and engine differ from what people see (critique item 4, `:16-17`). |
| Few variants | The colleague's A/B compares 12 hand-built variants and 4 themes behind URL flags (private comparison with the colleague's repository). Each variant is code someone wrote. |
| Metrics cannot rank designs | Two metrics gave the same value on all 28 views: 12 cue changes and a median reading load of 8 (private comparison with the colleague's repository). "Nothing that decides the winner has been measured yet" (`:236-237`). |
| Trade-offs are blocked | R-01: a navigation that cut one value from 23 to 0 made 20 of 168 gating values worse and landed only by a decision id (`r01-explainer-round.md:19-21`). |
| Metric defects | R-01: `reading_load` moved +2 to +80 only because the walk stopped scrolling elsewhere (`r01-explainer-round.md:22-23`). An accessibility rule was counted 12 times, once per journey (`research/2026-11-23-build-proposal.md:48`). |

## 2. The engine

One sentence: generate the design variants of an `ess-ui/1` document by rule, evaluate each against
every reachable UI state with no I/O, and search for the best compositions under declared rules.

```
base document ──► design space ──► hard rules ──► evaluation ──► metrics ──► search ──► frontier
 (ess-ui/1)       axes x operators   (static,      (layout model,  (pure      (strategy,    │
                                      prune)        fixtures,       functions) budget)      ▼
                                                    virtual clock)                       studio ──► promotion ──► ratchet
```

1. **Design space from the spec.** Axes are read from what `ess-ui/1` already makes a choice:
   themes (`ess/website/docs/reference/ess-ui.md:2680`), page layouts `stack | columns | areas`
   (`:630`), `collection.style` `table | cards | list | tree` (`:909`), overlay `kind` `drawer |
   dialog | fullscreen | popover` (`:1128`), `form.save` `on_submit | on_change | on_blur` (`:981`),
   action `as` (`:2070`). Mutation operators add structural moves (reorder sections, move an action
   to the header, split a form into tabs). Every variant passes `ess ui check`
   (`ess/website/docs/reference/cli.md:1823`); one that does not is never evaluated.
2. **No-I/O evaluation.** Reads are answered from ESS fixtures (`ess-ui.md:2143`), live events come
   from fixture scripts on a virtual clock that starts at zero
   (`ess/website/docs/reference/ess-ui-test.md:349-353`), command refusals are declared per test
   (`ess-ui-test.md:312`). A deterministic layout model turns (variant, UI state, environment) into
   geometry and an accessibility tree. No browser, no network, no wall clock. Details:
   `state-space.md` § 3-4.
3. **Metrics as pure functions.** A fixed catalogue (`metrics.md`), each metric typed with unit,
   direction, scope and kind. Hard rules prune; KPIs score.
4. **Search.** Exhaustive when the space is small, otherwise beam, hill climbing with restarts or
   Pareto evolution (`state-space.md` § 6). Every evaluation is cached by the digest of its inputs.
   The result is a **frontier**: the variants no other variant beats on every KPI.
5. **Studio.** A live view of the climb (best KPI values per generation), the frontier, two or more
   variants side by side in the same state and environment, and manual axis switches. The studio is
   itself an `ess-ui/1` document (brief, Glossary).
6. **Promotion.** A person picks a frontier variant. The engine writes the `ess-ui/1` diff, the
   ratchet evidence and a proposed protected reference. Merging the diff is a normal reviewed
   change; ratchet's merge gate holds it (`ratchet.md`).

The line-by-line replacement of the current loop is the table in `docs/brief.md` § "What it
replaces". The browser does not disappear. A browser assessor (`ux-walk@1` in `build-proposal.md:32-34`) runs
the promoted variant's journeys in a real engine, and its geometry is compared with the layout
model's. That comparison is the model's own quality measure [Inference].

## 3. Targets and their scope

A campaign has one **target**: the part of the document it varies and evaluates. `ess-ui/1` has the
units for each scope.

| scope | what it is in `ess-ui/1` | journeys and states come from | rendered in |
|---|---|---|---|
| component | one built-in composite kind with its props: `collection`, `form`, `choice`, `confirm`, ... (`ess-ui.md:1564-1620`) | the composite's own state: sort, page, selection, section lifecycle (`ess-ui.md:832-848`) | a harness container at declared widths |
| widget | an app-defined composite with typed params (`ess-ui.md:1359`) | its params bound to fixture rows; the states of the composites in its body | a harness container |
| section | one section of a page: one read, its own lifecycle (`ess-ui.md:806`) | the section's read and live updates | a harness page with only that section |
| page | one route: layout, header, sections, overlays (`ess-ui.md:554`) | `ess-ui-test/1` journeys on that page | the shell at declared viewports |
| composition | several targets optimised together, e.g. the shell, navigation and pages one journey crosses | journeys across pages | the shell |
| product | the whole document | all journeys | the shell |

**A component campaign** has its own journeys: sort by a column, page to page 3, select a row, run a
row action. Its states are few: a table's lifecycle states x sort keys x pages x selection. It is
rendered in a harness container at declared widths (for example 320, 480, 768 and 1200 px), not in a
page. Its design space and state space are small enough to search exhaustively (`state-space.md`
§ 2.3: 144 variants x 97 states x 16 environments).

**Promotion propagates.** A promoted widget variant changes one entry under `widgets:`; every use
expands it (`ess-ui.md:1412-1420`), so every page and product campaign that holds the widget gets a
new input digest and is evaluated again. Ratchet's merge gate then checks each subject that uses it
(`ratchet.md` § 5). For a built-in composite there is no single place for "default props" outside a
widget or a page kind (`ess-ui.md:710`), so a promoted component variant lands as a widget or a page
kind default. That is a gap to raise with ESS (`state-space.md` § 8).

**A page or product campaign treats components in one of two ways**, declared per component:
**fixed** (pinned at its promoted variant, by digest) or **axes** (its component campaign's top k
frontier variants become one axis of k values). Search runs bottom-up: components, then pages with
components fixed or as small axes, then product-level axes (theme, navigation, shell).

## 4. What stays human

The colleague's data and the external research agree: hermetic metrics do not rank designs, and KLM
predicts differences between close designs badly (mean error 3.5-5.5 s on three sites,
`research/2026-11-23-external-machine-checkable-ux.md:13`). Aesthetics, taste, comprehension and
trust are not machine-checkable here.

- The engine **ranks what is declared** and shows the rest side by side. It never claims a frontier
  variant looks better.
- Design proxies (emphasis per region, colour share, type scale) are computed and labelled as
  proxies (`metrics.md` § 11). They narrow a frontier; they do not decide it.
- **Judgement records** carry taste into the engine. A judgement record is a person's or a review
  agent's pairwise preference ("B over A in state s, environment e, because ...") or a finding on a
  variant. It is never a gate input. It enters in one of three ways:
  - a **design rule**: a recurring finding becomes a hard or soft rule ("no more than one primary
    action per region");
  - a **KPI weight**: repeated preferences adjust the declared weights of KPIs that already exist;
    the change is a policy change, reviewed like any threshold (ratchet design § 5,
    `ratchet/docs/design.md:126`);
  - a **finding**: a preference no metric explains is filed as a finding; it asks for a new metric or
    stays judgement.
- A review agent's record is labelled with its producer; two records from one model family count as
  one source [Inference, following `canon`'s `independent_on`,
  `research/2026-11-23-internal-building-blocks.md:124-127`].
- The final pick on a frontier is a person's (or an authorized policy's). Its record names the
  trade-off it accepts (`ratchet.md` § 6).

## 5. Worked example: list, detail and form on one page

A ticket desk page: a filter bar, a list of tickets, a detail record of the selected ticket, a
drawer to edit it, and a confirm before closing it. Illustrative; not run through `ess ui check`.

```yaml
format: ess-ui/1
app: desk
model: desk.platform
placement_profile: hybrid
fixtures: {dir: fixtures, index: fixtures/index.yaml}
navigation: {home: tickets.list, sections: [{name: work, label: Work, pages: [tickets.list]}]}
shells: {app: {regions: {main: {kind: page_outlet}}}}
themes: {standard: {}, compact: {space: {xs: 0.125rem, sm: 0.25rem, md: 0.5rem, lg: 0.75rem}}, calm: {color: {accent: "#3a5a8c"}}}
theme: {default: standard}
pages:
  tickets.list:
    kind: list_page
    title: Tickets
    layout: {columns: [{name: main, sections: [filters, list], width: wide}, {name: side, sections: [detail], width: narrow}]}
    sections:
      - name: list
        component: collection
        reads: {view: tickets.Page, params: {q: state.search}, paging: server}
        style: table
        selection: single
        columns: [{field: subject, sortable: true}, {field: priority, as: badge}, {field: updated_at, as: date, sortable: true}]
        row_actions:
          - {name: edit, opens: edit, label: Edit}
          - {name: close, does: tickets.CloseTicket, bind: {id: row.id}, confirm: {title: Close ticket}}
      - {name: detail, component: record, depends_on: list, reads: {view: tickets.ById}, fields: [subject, requester, priority, body]}
    overlays:
      edit: {kind: drawer, component: form, does: tickets.UpdateTicket, fields: [subject, priority]}
```

**Axes** (each value is a valid `ess-ui/1` construct):

| axis | kind | values | n |
|---|---|---|---|
| theme | theme | `standard`, `compact` (space only), `calm` (colour only) | 3 |
| page layout | layout | stack; columns list wide / detail narrow; columns list narrow / detail wide; areas with filters on top | 4 |
| detail placement | interaction | section beside the list; drawer opened by the row; inline `expand` | 3 |
| list presentation | component | `table`, `cards`, `list` | 3 |
| edit surface | interaction | `drawer`, `dialog`, inline form in place of the detail | 3 |
| save | interaction | `on_submit`, `on_blur` | 2 |
| close confirm | interaction | inline confirm, dialog with type-to-confirm, none | 3 |
| edit entry point | layout | row action, action on the detail, both | 3 |
| edit label | copy | "Edit", "Edit ticket" | 2 |

**Size of the design space.**

- Product of the value counts: 3 x 4 x 3 x 3 x 3 x 2 x 3 x 3 x 2 = **11,664**.
- Operator preconditions remove impossible combinations. Layouts 2-4 place `detail` in a column or
  area, so they need detail as a section: 4 layouts with a section detail, 1 otherwise. The inline
  edit form replaces the detail section, so it also needs a section detail. Layout x detail x edit
  surface: 4 x 1 x 3 + 1 x 1 x 2 + 1 x 1 x 2 = 16 combinations instead of 36. Valid variants:
  16 x 3 x 3 x 2 x 3 x 3 x 2 = **5,184**.
- Hard rule "a command that ends an entity's lifecycle has a confirm" prunes "close confirm: none"
  statically, before any layout: **3,456** variants are evaluated. [Inference: the rule reads the ESS
  model's terminal states; it is an engine rule, not an `ess ui check` today.]
- Environments: 3 viewports (1440x900, 1280x720, 390x844) x 2 theme modes x 2 input modes x 2 text
  scales = **24**.
- Journeys: J1 find a ticket by search and read it; J2 change its priority; J3 close it, refused
  once by a fixture refusal, then accepted.
- UI states per (variant, environment): **1,501** (`state-space.md` § 2.2).

Full product: 3,456 x 24 x 1,501 = 124,498,944 (variant, environment, state) evaluations. Colour-only
differences (theme `calm`, theme mode) and input mode do not change geometry, so the layout model
runs 2,304 x 6 x 1,501 = 20,749,824 times. At an assumed 50 µs per layout call (a target for the
model, not a measurement) that is 1,037 CPU-seconds. Search is staged instead (`state-space.md`
§ 6): static rules on all 3,456 variants, journey-path states (about 25) on all of them, the full
state space only for the frontier.

Out of scope this round: code, the layout model's algorithm, replacing ESS's renderers, live sessions
with people (questionnaires, wall-clock timing).
