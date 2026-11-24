#!/bin/sh
# release-notes.sh <version>: print the CHANGELOG.md section of <version> without its heading and
# without leading or trailing blank lines.
set -u
version="${1:?usage: release-notes.sh <version>}"
root=$(cd "$(dirname "$0")/.." && pwd)
awk -v heading="## $version — " '
  index($0, heading) == 1 { found = 1; next }
  found && /^## / { exit }
  found { lines[++n] = $0 }
  END {
    first = 1; while (first <= n && lines[first] ~ /^[[:space:]]*$/) first++
    last = n; while (last >= first && lines[last] ~ /^[[:space:]]*$/) last--
    for (i = first; i <= last; i++) print lines[i]
  }' "$root/CHANGELOG.md"
