# Fixtures of the theme check

`tools/check-themes.sh` reads these files so that the gate checks a theme against its consumer without
the consumer's repository.

| file | source | sha256 |
|---|---|---|
| `slot.ui.yaml` | `spec/ui/slot.ui.yaml` of `github.com/beyond10x/slot`, commit `a3c91b6` (branch `ess-ui-job-views`), unchanged | `d37dce4fd77a5ecb39afa134516f9665df84bb74fd9449514ede3cd52845a87b` |

The check compares each file with its sha256 here. A new copy from slot replaces the file and its row
in one change.
