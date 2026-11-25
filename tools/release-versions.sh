#!/bin/sh
# release-versions.sh <version> [<binary directory>]: exit 0 when the plugin manifests, every
# workspace crate and the changelog carry <version> and, with a binary directory, each released
# binary in it (ux-browser, ux-static) answers `--version` with exactly `<name> <version>` and a
# newline on stdout and exit 0; print each disagreement and exit 1 otherwise.
set -u
version="${1:?usage: release-versions.sh <version> [<binary directory>]}"
bins="${2:-}"
root=$(cd "$(dirname "$0")/.." && pwd)
bad=0
nl='
'

if [ -n "$bins" ]; then
  for name in ux-browser ux-static; do
    if [ ! -x "$bins/$name" ]; then
      echo "$bins/$name: no binary"
      bad=1
      continue
    fi
    # The trailing `x` keeps the answer's own newlines from the command substitution.
    answer=$("$bins/$name" --version 2>/dev/null; status=$?; printf x; exit $status)
    status=$?
    answer=${answer%x}
    if [ "$status" -ne 0 ]; then
      echo "$bins/$name --version: exit $status"
      bad=1
    elif [ "$answer" != "$name $version$nl" ]; then
      echo "$bins/$name --version: answers '${answer%"$nl"}', release $version"
      bad=1
    fi
  done
fi

for manifest in plugin/.claude-plugin/plugin.json plugin/.codex-plugin/plugin.json; do
  [ -f "$root/$manifest" ] || continue
  found=$(sed -n 's/^ *"version": *"\([^"]*\)".*/\1/p' "$root/$manifest" | head -1)
  if [ "$found" != "$version" ]; then
    echo "$manifest: version $found, release $version"
    bad=1
  fi
done

for crate in "$root"/engine/*/Cargo.toml "$root"/generators/*/Cargo.toml; do
  [ -f "$crate" ] || continue
  found=$(sed -n 's/^version *= *"\([^"]*\)".*/\1/p' "$crate" | head -1)
  if [ "$found" != "$version" ]; then
    echo "${crate#"$root"/}: version $found, release $version"
    bad=1
  fi
done

if ! grep -q "^## $version — " "$root/CHANGELOG.md"; then
  echo "CHANGELOG.md: no dated section $version"
  bad=1
fi

exit $bad
