# Rounds on the studio

One row per round. Every number is lower-is-better and comes from the round's `round.json`
(`metrics`). A round without people has no questionnaire columns.

| round | gates failed | scenarios not completed | runs failed | looks | always-visible misses | overflow blocks | pages with document scroll | pages without title | text under 14 px | low-contrast runs, worst page | layout shift, max | findings open (S1) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-01 | 1, 2, 3 | 6 of 11 | 2 of 44 | 16 | 24 | 14 | 2 of 10 | 4 of 10 | 0.564 | 12 | 0.0151 | 22 (4) |
| R-02 | 1, 2, 3 | 5 of 11 | 2 of 44 | 12 | 20 | 14 | 2 of 10 | 4 of 10 | 0 | 0 | 0.0176 | 20 (2) |
| R-03 | 1, 3 | 4 of 11 | 0 of 44 | 8 | 0 | 14 | 2 of 10 | 0 of 10 | 0 | 0 | 0.0284 | 20 (0) |

Not measured in any round: gate 4 (the screen-only user) and gate 5 (accessibility by axe-core).

R-02 was measured on another machine than R-01, with the same browser build. R-01's input measured
again on it is `R-02/before.round.json`: every count of R-01's row is equal, the layout shift is
0.026. That value changes from run to run on the same input (UXF-023).

From R-02 on the always-visible misses count one item more, the Owner's action on a promotion: with
it, R-01's input gives 40.

R-03: R-02's input measured again is `R-03/before.round.json`: every count of R-02's row is equal,
the layout shift is 0.0229. Ten runs of the start form on R-03's theme gave 0.0062 to 0.0284, seven
on R-02's theme 0.003 to 0.0284 (UXF-023). From R-03 on `round.json` counts two keys more, which
this table does not show: the cells of the page audit under a floor (`audit.cells_under_a_floor`:
16 of 40 on R-02's input, 0 in R-03) and the runs that showed a notice with a command's name in it
(`trial.runs_notice_expectation_missed`: 26 of 44 on R-02's input, 28 in R-03; UXF-010, a known
loss). The 20 findings open after R-03 are 11 S2 and 9 S3; three of them are new.
