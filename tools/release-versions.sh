#!/bin/sh
# release-versions.sh <version> [<binary directory>]: exit 0 when every file that carries the
# release's version carries <version> and, with a binary directory, each released binary in it
# (ux-browser, ux-static) answers `--version` with exactly `<name> <version>` and a newline on
# stdout and exit 0; print each disagreement as `<file>: <what>` and exit 1 otherwise.
#
# The carriers, all written by `sh tools/release-set-version.sh <version>`:
# - the two plugin manifests and the plugin entry of the Claude marketplace;
# - every crate under engine/ and generators/, and the packages of this repository in the two lock
#   files (Cargo.lock, engine/search/Cargo.lock: the packages without a `source`);
# - the pages and the data that name the plugin version or the current release
#   (website/docs/index.md, website/data/status.json, docs/roadmap.md) and the pages generated
#   from them (website/docs/status.md, reference/skill.md, roadmap.md);
# - CHANGELOG.md: a section `## <version> — <date>` with at least one entry.
# A carrier that is not there is a disagreement. A recording names the tool it was recorded with
# and is no carrier: a recorded campaign (engine/search/tests/fixtures/) and the two example
# documents of ux-browser (engine/browser/examples/), each of a browser run.
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

# carries <file> <what> <sed program>: the program prints, with `sed -n`, each version the file
# names in that place. There is at least one, and each is the release's.
carries() {
  if [ ! -f "$root/$1" ]; then
    echo "$1: no such file, release $version"
    bad=1
    return
  fi
  found=$(sed -n "$3" "$root/$1" | sort -u)
  if [ -z "$found" ]; then
    echo "$1: no $2, release $version"
    bad=1
    return
  fi
  for one in $found; do
    if [ "$one" != "$version" ]; then
      echo "$1: $2 $one, release $version"
      bad=1
    fi
  done
}

manifest='s/^ *"version": *"\([^"]*\)".*/\1/p'
for file in plugin/.claude-plugin/plugin.json plugin/.codex-plugin/plugin.json .claude-plugin/marketplace.json; do
  carries "$file" version "$manifest"
done

crates=0
for crate in "$root"/engine/*/Cargo.toml "$root"/generators/*/Cargo.toml; do
  [ -f "$crate" ] || continue
  crates=$((crates + 1))
  found=$(sed -n 's/^version *= *"\([^"]*\)".*/\1/p' "$crate" | head -1)
  if [ "$found" != "$version" ]; then
    echo "${crate#"$root"/}: version $found, release $version"
    bad=1
  fi
done
if [ "$crates" -eq 0 ]; then
  echo "engine/*/Cargo.toml: no crate, release $version"
  bad=1
fi

for lock in Cargo.lock engine/search/Cargo.lock; do
  if [ ! -f "$root/$lock" ]; then
    echo "$lock: no such file, release $version"
    bad=1
    continue
  fi
  # A package of this repository has no `source`: its name, then its version, then another key.
  local_packages=$(awk '
    { line[NR] = $0 }
    END {
      for (i = 1; i < NR; i++)
        if (line[i] ~ /^name = "/ && line[i + 1] ~ /^version = "/ && line[i + 2] !~ /^source = /) {
          name = line[i]; sub(/^name = "/, "", name); sub(/"$/, "", name)
          found = line[i + 1]; sub(/^version = "/, "", found); sub(/"$/, "", found)
          print name, found
        }
    }' "$root/$lock")
  if [ -z "$local_packages" ]; then
    echo "$lock: no package of this repository, release $version"
    bad=1
    continue
  fi
  while read -r name found; do
    if [ "$found" != "$version" ]; then
      echo "$lock: package $name $found, release $version"
      bad=1
    fi
  done <<EOF
$local_packages
EOF
done

carries website/docs/index.md 'plugin version' 's/.*plugin version `\([^`]*\)`.*/\1/p'
carries website/data/status.json 'plugin version' 's/.*"detail": "Plugin \([0-9][^ ]*\) for Claude Code and Codex.*/\1/p'
carries website/docs/status.md 'plugin version' 's/.*| Plugin \([0-9][^ ]*\) for Claude Code and Codex.*/\1/p'
carries website/docs/reference/skill.md 'plugin version' 's/^This page is the skill itself: .*, plugin version `\([^`]*\)`\. .*/\1/p'
current='s/.*The current release is \([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\).*/\1/p'
carries docs/roadmap.md 'current release' "$current"
carries website/docs/roadmap.md 'current release' "$current"

if [ ! -f "$root/CHANGELOG.md" ]; then
  echo "CHANGELOG.md: no such file, release $version"
  bad=1
elif ! grep -q "^## $version — " "$root/CHANGELOG.md"; then
  echo "CHANGELOG.md: no dated section $version"
  bad=1
elif ! awk -v heading="## $version — " '
    index($0, heading) == 1 { on = 1; next }
    on && /^## / { exit }
    on && /[^[:space:]]/ { entry = 1 }
    END { exit !entry }' "$root/CHANGELOG.md"; then
  echo "CHANGELOG.md: the section $version has no entry"
  bad=1
fi

exit $bad
