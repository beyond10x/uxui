# Rounds on the studio

One row per round. Every number is lower-is-better and comes from the round's `round.json`
(`metrics`). A round without people has no questionnaire columns.

| round | gates failed | scenarios not completed | runs failed | looks | always-visible misses | overflow blocks | pages with document scroll | pages without title | text under 14 px | low-contrast runs, worst page | layout shift, max | findings open (S1) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R-01 | 1, 2, 3 | 6 of 11 | 2 of 44 | 16 | 24 | 14 | 2 of 10 | 4 of 10 | 0.564 | 12 | 0.0151 | 22 (4) |
| R-02 | 1, 2, 3 | 5 of 11 | 2 of 44 | 12 | 20 | 14 | 2 of 10 | 4 of 10 | 0 | 0 | 0.0176 | 20 (2) |

Not measured in R-01 and R-02: gate 4 (the screen-only user) and gate 5 (accessibility by axe-core).

R-02 was measured on another machine than R-01, with the same browser build. R-01's input measured
again on it is `R-02/before.round.json`: every count of R-01's row is equal, the layout shift is
0.026. That value changes from run to run on the same input (UXF-023).

From R-02 on the always-visible misses count one item more, the Owner's action on a promotion: with
it, R-01's input gives 40.
