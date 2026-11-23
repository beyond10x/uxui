# Internal building blocks for a spec-driven ui-iteration-loop

Research, 2026-11-23. Read only. Paths are relative to the directory that holds the beyond10x repositories unless absolute. "Inferred" marks a
claim not stated in a file. Critique items are the numbers of `approach-critique.md` (same text as
`uxui/docs/reviews/2026-11-23-ui-iteration-loop-critique.md`, `cmp` equal).

## 1. ESS UI (`ess ui check`, `ess-ui-test/1`)

What `ess ui check` checks
- 51 schema checks in the table `ess/website/docs/reference/ess-ui.md:2758-2814`, declared as data
  in `ess/schemas/ui/ess-ui.schema.yaml:1745` (`checks.list`, each `{id, subject, must..., severity}`).
- Plus 25 code-only rules, `ess/crates/ui/ess-ui-check/src/lib.rs:176-237` (`document_loads`,
  `layer_rules`, `degrades_cover`, `frame_rules`, `*_in_model`, `row_fields`, `section_row`, ...).
- All are structural or referential: names unique, refs resolve (`nav_resolves`, `page_refs`,
  `section_refs`), `page_reachable`, widget expansion, filter/pager rules, `expr_paths`, token
  grammar (`token_values`), theme completeness (`theme_tokens`, `theme_choice`), model agreement
  with `--model` (lib.rs:186-215).
- Layout: only `layout_complete` = "place every section once", a warning (ess-ui.md:2770,
  schema:1757). `PageLayout` is stack/columns/areas, renderer-neutral, no viewport or breakpoint
  (ess-ui.md:630-700).
- Accessibility: only required text on leaves: icon `label` "accessible text" (ess-ui.md:1783),
  button `label` (1813), image `alt` (1932). `input` has `as`, `binds`, `placeholder`, no label
  (ess-ui.md:1880-1882; a field inside a form gets one, inferred).
- Nothing on viewport, scroll, overflow, roles, keyboard, focus, contrast, motion, reading load:
  `grep -iE 'viewport|keyboard|focus|aria|contrast|motion|wcag|overflow'` on ess-ui.md hits only
  the token named `focus` (2637) and "scroll" as component state (2357).
- Themes: tokens are typed (hex/rgb grammar, lengths, weights; ess-ui.md:2611-2617) and every
  theme defines every token (2684). No contrast pairing of `text` against `bg` (inferred:
  the grammar holds the values a contrast check would need).
- Extensibility: the rule table is data, the predicates (`must: place_every_section_once`) are
  Rust: `CHECKS` const `ess-ui-check/src/lib.rs:123`, impl `src/rules.rs`. A third party cannot add
  a check without an ESS release; uilab files missing constructs "in `../ess` and waits for an ESS
  release" (`uilab/AGENTS.md:19-23`). The only external input is `--hosts` (lib.rs:171).
- Report `ess-ui-check/1`: `{path, check, severity, message}`, `path` = canonical node path
  (`ess/website/docs/reference/formats.md:198`).

`ess-ui-test/1`
- Steps: open, select, mark, type, choose, act, move, upload, page, expect, play, advance,
  expect_command, expect_export (ess-ui-test.md:94-270). Nodes by canonical path, never by label or
  CSS (59-64, 469-495). Fixtures per view, per-view latency, virtual clock (54, 279-353).
- `expect` reads text, rows, section state (`loading/empty/stale/failed/ready`), toggle state
  (195-224). Nothing geometric.
- Report `ess-ui-test-report/1`: `{format, document, tests:[{name, status, step?, message?}]}`
  (ess-ui-test.md:355-376, formats.md:200). No timing, no step counts, no per-step record.
- UX metrics from a run: not today. Actions per task can be counted statically from the test file
  (act/type/choose/select/mark steps; inferred). Glance cost needs geometry: the terminal renderer
  records per frame where it drew each node by path (`ess_ui_tui::App::regions`,
  ess-ui-test.md:395-399), so "node in view" is computable on the TUI (inferred); the browser spec
  is "generated and not run by `ess ui test`" (504).
- "One verdict in both renderers": wherever the terminal refuses a step, the Playwright test is
  `test.fixme` with the same reason; both read refusals from one place (402-427). A missing path
  fails in both (427).
- Playwright emission: every node `page.locator('[data-ui-path="<path>"]')`, `page.clock` installed
  at zero, `advance`/`play` -> `page.clock.runFor`, `open` -> `page.goto` (437-457). Written, not run.
- Critique items it closes: 7 (virtual clock removes timing noise for state-driven checks), 12
  partly (fixtures + scripts declare the states), 9 partly (paths decouple tests from labels; does
  not test perception). Opens none of 3, 4, 5 (no a11y, one browser, no viewport concept).

## 2. ESS: formats, conformance, hardening

- A family declares its version in its bytes; a reader states which versions it implements and
  refuses the rest (`ess/website/docs/reference/spec-versions.md:9-26`). A new version when meaning,
  identity, references, names or envelope change (`ess/AGENTS.md:51-52`).
- Adding a family = a row in `formats.md` (UI rows 197-200) and in spec-versions "Every other
  family" (1007-1050; `ess-ui/1` added 0.47.0, `ess-ui-test/1` 0.48.0). `ess-ui/1` is "defined as
  data in `schemas/ui/ess-ui.schema.yaml`" and its reference page is generated by `ess ui docs`
  (ess-ui.md:9).
- Conformance: `ess/suites/generated/{billing,gatepass,oracle-fixture}/suite.json`; authored
  scenarios `ess-scenario/6` (spec-versions.md:832-855); mutation audit `ess-mutation-report/8`
  (950-1005).
- Hardening rule: "A check nobody has seen fail is not evidence. Plant a defect first."
  (`agentplugins/plugins/ess/skills/hardening/SKILL.md:15-21`). Catalogue
  (23-40): mutation (1), metamorphic relations (5), guard analysis (6: dead/overlapping/gap
  guards), spec diff in the gate (7: is a change breaking, acknowledged?), design review (8).
- Conformance gate (`.../ess/skills/testing-conformance/SKILL.md:287-311`): floor `answered` not
  `passed` (flaps do not fail the gate), ceiling `skipped`, `total` may fall only with `answered`
  flat and the reason recorded, quarantine by name, verdict from the report document not the exit
  code; "a target that does nothing" check (52-66).
- Maps to critique: 7 (floor/ceiling on counts, not a monotone value), 23 (`total` may fall when
  a fiction leaves), 6/8 (technique 7: classify a baseline move as breaking + acknowledged instead
  of a free-form decision id), 15 and 10 (plant-a-defect proves a gate can fail; mutation of the
  contract shows which rules no scenario pins).
- A UX contract beside `system.yaml` (inferred): it would bind to `ess-ui/1` node paths and to
  `ess-ui-test/1` scenarios, the way `ess-ui-check/1` already reports model `coverage
  {covered, host_covered, uncovered}` with `--model` (formats.md:198). Two shapes: new constructs in
  `ess-ui/1` (viewport profiles, always-visible paths, completion state) checked by `ess ui check`,
  or a separate family (`ess-ui-ux/1`) with its own reader and report. Either needs an ESS release.

## 3. uilab

- Edits an `ess-ui/1` document by voice; agent proposes one patch per node (`uilab/README.md:3-7`).
- Its gate on patches: "A patch is refused when its result has an ESS error the document did not
  already have" (`uilab/AGENTS.md:31-32`) — a no-new-error ratchet on `ess ui check`.
- `evals/library.yaml` (137 lines): 27 cases `{target, say, expect: {op, layer, component}}`;
  reports per round `evals/reports/library-round-{1..5}.{md,json}`; round 5: 26 of 27, median
  2827 ms, 23 admitted patches checked with `ess ui check`: 0 errors, 3 warnings
  (`library-round-5.md:3, 38-42`).
- Measures the agent's edit accuracy, not the rendered UI. No layout, a11y or task metric.
  Overlap with the loop: round/report vocabulary and the "no new ESS error" gate only.

## 4. ess-ui-templates, widgets

- `ess-ui-templates`: `ess-ui check --html <page>` "exits 1 unless the page draws exactly what the
  IR declares" (`README.md:40`); 8 checks: sets per kind, IR digest, source lines, link coverage,
  nothing listed that is not drawn, examples shown, scenario replay, demo script fresh
  (`crates/ess-ui/src/check.rs:1-16`). Closes critique 2 for generated pages (content correct).
- `task tokens:check` fails on any difference of `web/src/tokens.css` and fonts from the pinned
  docs-system commit (`Taskfile.yml:77-85`; `AGENTS.md:30-31`): design tokens as a checked pin.
- `prefers-reduced-motion` honoured (`web/src/page.css:264`, `demo.css:150`, `ui.js:7`).
- `task screenshots`: headless Chrome at 1440 and 390, light and dark (`Taskfile.yml:110-118`). No
  axe, no layout assertion, no ratchet (rg for axe-core: no hit).
- `widgets/widgets/phone/widget`: `check` = typecheck + `node --test` unit tests on media logic
  (`package.json` scripts); no UI quality, a11y or token rules.

## 5. runbook / Canon: the round as a `protocol/1`

Canon evaluates one case under one protocol: claims are TRUE/FALSE/UNKNOWN from evidence bound to
an artifact revision (`canon/README.md:5-9`); evidence for a superseded revision leaves a claim
UNKNOWN, so a Build that revises `ui` voids the old measurements and forces "measure again"
(§ 3.6). Actions have preconditions, `requires: capability` (approval-required until granted) and
`may_produce` (`canon/website/docs/concepts/protocols.md:93-129`); a person action is a binding
`executor: {kind: person, principal: ...}` (dev-operator `bindings/write-jira-comment.yaml:33-36`).
Outcomes are `legitimate` only when their predicate is TRUE or a named decision is given
(protocols.md:269-303) — `merged`, `reverted`, `kept-with-reason`. `independent_on` makes a match
read a record only from a producer independent on declared dimensions
(`canon/website/docs/reference/protocol.md:211-216, 348`): two reviews of one model family or by the
build's author would not count as two sources (critique 13, 14). `author: program|judgement`
marks judgement steps (protocol.md:98, 307-314; critique 18). `canon check` decides every outcome
over its state space and reports bypasses: an outcome resting on evidence an authority-requiring
action may produce without an authority decision (`reference/cli.md:126`) — a baseline write
guarded by a capability the build agent lacks (critique 15). `canon simulate` reports stuck states,
unreachable outcomes, cycles (cli.md:153); `canon generate` writes one witness scenario per outcome.
Runbook composes child cases (`case_open`, `children` form), `retry_when`/`budget.max_runs`, and
`max_age` expiry (dev-operator AGENTS.md: needs-you, refresh-read). Limits: 65536 states per slice;
dev-operator split composers into child protocols to stay under it (AGENTS.md "Bound 65536").

## 6. Existing UI measurement in the org

`rg` over the beyond10x repositories (node_modules, target, dist excluded):
- The loop is already built 7 times, in 3 languages, each with its own `baseline.json`:
  - `modelspec/ui/ux/` (TS: `contract.ts`, `core/metrics.ts`, `core/bot.ts`, `ratchet.spec.ts`,
    `self-test.spec.ts` plants one defect per gate, 1-7; one injected font stack so CI and local
    measure alike, `modelspec/AGENTS.md:183-185` — partial answer to critique 4).
  - `runbook/ui/ux/` (mjs: `core/contract.mjs`, `core/ratchet.mjs`; improvement only outside the
    tolerance band so a noise sample does not become the floor, `ratchet.mjs:43-44` — critique 7).
  - `conductor/crates/conductor/dashboard/ux/` (TS: `breaks.ts` "break-tests ... A contract that
    cannot fail gates nothing", 1-3; `gate.ts`, `contract.ts`) and `conductor/ux/` (mjs).
  - `slot/website/ux/` and `slot/ui/ux/` (TS: `ratchet-lib.ts`, `gates.spec.ts`,
    `screen-only.spec.ts`, `viewports.ts`).
  - `substrate/website/ux/` + `substrate/xtask/src/site_ux.rs` (Rust: reading load from Markdown,
    link clicks per task from built HTML, 1-8).
- `uxui/` is the plugin's source; it "ships no scripts" (`uxui/README.md:12-13`);
  `uxui/docs/research/README.md` is a stub.
- Other axe-core: `devcenter/frontend/e2e/devcenter.spec.ts`, `runbook/website/tests/variants.ts`.
- `getByRole`: `docs-system/tests/ess-viewer.browser.mjs`, `connectors/website/tests/*.mjs`,
  `devcenter/frontend/acceptance/*.spec.ts`, `runbook/ui/test/*.spec.mjs`.
- Blinded evaluation: `runbook/eval/quality/blind.mjs:1` shuffles products to p1..p9 with a
  `key.json`, `rubric.ts:1` runs the same steps on each (critique 10, 13: a judge who does not
  know whose UI it is).
- `layout_shift` appears only in modelspec and runbook ux code; no repo measures it elsewhere.

## Building blocks that exist

- Canonical node paths, the same in TUI and React (`data-ui-path`), stable under restyle.
- Hermetic UI states: `ess-ui-test/1` fixtures, channel scripts, virtual clock, Playwright emission.
- One-verdict rule across renderers (terminal refusal = `test.fixme` with the same reason).
- Typed design tokens and themes with grammar checks; tokens pin check (`tokens:check`).
- Data-declared check table (`checks.list`) and a report format keyed by node path.
- Page-held-to-IR check (`ess-ui check --html`) for generated pages: content correctness.
- ESS hardening: plant-a-defect rule, mutation audit, spec diff in the gate, design review.
- Conformance gate design: floor answered, ceiling skipped, quarantine by name, report = verdict.
- Canon: revision-bound evidence, `independent_on`, judgement marks, capabilities, decided
  outcomes, `check`/`simulate`/`generate`; runbook to run it with person actions.
- Seven working loop implementations (contract, bot, ratchet, findings test, break-tests).

## Gaps

- No layout, viewport, scroll, focus, keyboard, role, contrast, motion or reading-load construct
  or check in `ess-ui/1`; `input` carries no label.
- Checks are Rust predicates; no way to declare a new check as data outside an ESS release.
- `ess-ui-test-report/1` holds pass/fail only: no per-step trace, no counts, no geometry; the
  Playwright spec is not run by `ess`.
- No UX contract artifact (viewports, always-visible paths, completion state, task budgets) bound
  to node paths; the adapter of the skill lives in each repo's own TS/mjs.
- No shared ratchet format: seven `baseline.json` shapes and seven ratchet tools, no spec diff
  classifying a baseline move.
- No protocol for the round in `protocol/`; independence of reviewers and the build agent's write
  limits on tests/baseline are prose in SKILL.md, not checked.
- Nothing covers critique 2 (comprehension, trust), 3 (manual a11y share), 4 (other engines), 24
  (PII in recordings and L5 screenshots) in any repo found.
