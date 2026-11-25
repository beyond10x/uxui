# Cost spike: one evaluation on layout model v0

Status: method and tool written 2026-11-25 (`story:cost-spike`, roadmap step 3); the numbers below are
**measured 2026-11-25** (Results below). `docs/state-space.md` § 2.3 and § 6 assume 50 µs per layout
call and, from the scaffold spike of 2026-11-24, about 400 µs per call plus 3.8 ms per variant and page;
this report replaces both with a measurement per UI state.

## Method

`ux-search cost` (`engine/search/src/cost.rs`) drives one page of an ESS example in process through ESS's
terminal renderer (`ess_ui_tui::App` at ESS tag 0.99.4, the crates by git, no CLI and no subprocess).

- **Page.** `tickets.detail` of ESS's `examples/partner-portal/ui.yaml` at 0.99.4, copied with its
  fixtures to `engine/search/tests/fixtures/partner-portal/` (SHA-256 per file in its `README.md`),
  param `id=tk-01`; its sections are a record, a list of messages and a reply form.
- **UI states.** State 0 is the page opened; state n is state 0 after the first n keys of a key
  sequence (default `<tab>` five times: focus moves through the page). ESS 0.99.4 has no UI state API
  (`docs/ess-gaps.md` P0 1; `App` has no `Clone`, `docs/state-space.md:198-199`), so a state cannot be
  forked: every state is reached by opening the document again and replaying its keys from the open page.
- **Times per state**, each the median of the runs (`--repeat`):
  - open: `App::from_text` (loads the document and the fixtures and opens the home page), `open_page`
    and the reads due at time zero;
  - replay: the state's keys, each with the reads due at time zero; step: the last key's share of it;
  - geometry: `render_text` at the size (default 120 x 40 cells) and `regions`, the cells every node
    occupies.
- **Memory.** The process's peak resident set (`VmHWM` in `/proc/self/status`) before the first open
  and after each state. It is a high-water mark of the whole process, so it rises with the first open
  and then shows growth only.

One evaluation of a state `d` keys deep therefore costs, replayed from open:

c(d) = t_open + d x t_step + t_geometry

and, with a forkable state (ESS gap P0 1 closed), t_step + t_geometry per state plus one t_open per
variant and environment. The tool prints both, and how many fit in 120 s on one core.

## Command

Build once through the pool, then measure without it (a measurement holds no build slot), on an idle
host, from the repository root:

```console
cargo build --release --manifest-path engine/search/Cargo.toml --locked
engine/search/target/release/ux-search cost --doc engine/search/tests/fixtures/partner-portal/ui.yaml \
  --page tickets.detail --param id=tk-01 --size 120x40 --repeat 25
```

`--format json` prints the same as one `ux-search-cost/1` object.

## Results

Measured 2026-11-25 17:22 CET on the workstation (20 CPUs, load average 15.55 four minutes after a reboot), `ux-search cost --repeat 5`, median of 5 runs, the debug build of `ux-search` with its dependencies (ESS, ratatui) at `opt-level = 2` (`engine/search/Cargo.toml`, `[profile.dev.package."*"]`), not the release build named above.

| state | key | open µs | replay µs | step µs | geometry µs | regions | VmHWM KiB |
|---|---|---|---|---|---|---|---|
| 0 | - | 38,059 | 0 | 0 | 1,293 | 30 | 16,204 |
| 1 | `<tab>` | 33,725 | 131 | 131 | 1,510 | 30 | 16,204 |
| 2 | `<tab>` | 27,465 | 209 | 97 | 1,355 | 30 | 16,204 |
| 3 | `<tab>` | 29,488 | 324 | 109 | 1,145 | 30 | 16,204 |
| 4 | `<tab>` | 25,518 | 441 | 106 | 1,243 | 30 | 16,204 |
| 5 | `<tab>` | 25,251 | 547 | 108 | 1,181 | 30 | 16,204 |

Means: t_open 29,918 µs, t_step 110 µs, t_geometry 1,288 µs; VmHWM 3,512 KiB before the first open. One evaluation replayed from open: 31,206 µs at depth 0, 31,760 µs at depth 5 (about 3,800 per 120 s on one core). With a forkable state (t_step + t_geometry): 1,399 µs (about 85,800 per 120 s).

## Go or no-go per scope

Exhaustive search is **go** for a scope when its evaluations fit the call budget of
`docs/state-space.md:162`, 120 s on one core: G x c(d) ≤ 120 s, with G the layout calls of the scope and
d its mean depth in keys. The budget per evaluation is 120 s / G:

| scope | G (source) | mean depth d | budget per evaluation | c(d) measured | verdict |
|---|---|---|---|---|---|
| component: the ticket `collection` | 144 x 8 x 97 = 111,744 (`docs/state-space.md:64,70-72`) | about 3 (sort, selection, page) [assumption] | 1,074 µs | 31,500 µs replayed; 1,399 µs forkable | **no-go**: 29 times over replayed, 1.3 times over forkable |
| page, full: the ticket page | 2,304 x 6 x 1,501 = 20,749,824 (`docs/state-space.md:85-88`) | about 8 (journeys of 7 to 9 steps, `docs/state-space.md:173`) | 5.8 µs | 32,100 µs replayed; 1,399 µs forkable | **no-go** either way |
| page, stage 2: journey paths | 345,600 (`docs/state-space.md:173`) | about 8 | 347 µs | 32,100 µs replayed; 1,399 µs forkable | **no-go**: 4 times over even forkable |
| page, stage 3: frontier only | 450,300 (`docs/state-space.md:174`) | about 8 | 266 µs | 32,100 µs replayed; 1,399 µs forkable | **no-go**: 5 times over even forkable |
| composition, product | not enumerated (`docs/state-space.md:67-68`) | | | | no-go by design: bottom-up |
| toy climb: theme x section order | 18 variants x 2 sizes, one state each (`ux-search climb`) | 0 | 6.7 s per variant | about 300 ms (two climbs of 18 variants: 5.4 s and 5.9 s, each variant checked, opened once and drawn 4 times) | **go**: about 20 times under |

What the 2026-11-24 numbers already say (`docs/state-space.md:74-78`: open 1.2 ms, draw 0.4 ms at host
load about 100): with replay, c(d) is at least t_open + t_geometry, about 1.6 ms, which is over every
page budget above whatever a step costs; so the page scope is expected to be **no-go while states are
replayed from open**, and to turn on t_step + t_geometry once ESS gap P0 1 gives a forkable state. The
component scope sits near its budget and is decided by the measurement. The toy climb fits by three
orders of magnitude. Measured (above): the expectations hold for the page scopes and the toy climb. The component scope is no-go too: 1,399 µs per evaluation even with a forkable state, against a budget of 1,074 µs; it needs a cheaper evaluation (draw one size, read only the changed regions) or a smaller space before it is exhaustive. The open, about 30 ms, is 95 % of a replayed evaluation, so ESS gap P0 1 (a forkable state) gains about 22 times.
