#!/bin/sh
# release-versions.sh <version>: exit 0 when the plugin manifests, every workspace crate and the
# changelog carry <version>; print each disagreement and exit 1 otherwise.
set -u
version="${1:?usage: release-versions.sh <version>}"
root=$(cd "$(dirname "$0")/.." && pwd)
bad=0

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
