#!/bin/sh
# Proves that tools/release-versions.sh passes when the plugin, the crates and the changelog agree
# with the release and fails on each disagreement, and that tools/release-notes.sh prints the
# section of a release. Copies go under target/release-test/. Exit 0 when every case holds.
set -u

root=$(cd "$(dirname "$0")/.." && pwd)
work=$root/target/release-test
fails=0

fresh() {
	rm -rf "$work"
	mkdir -p "$work/tools" "$work/plugin" "$work/engine/a" "$work/generators/b"
	cp "$root/tools/release-versions.sh" "$root/tools/release-notes.sh" "$work/tools/"
	mkdir -p "$work/plugin/.claude-plugin" "$work/plugin/.codex-plugin"
	printf '{\n  "name": "uxui",\n  "version": "9.8.7"\n}\n' >"$work/plugin/.claude-plugin/plugin.json"
	printf '{\n  "name": "uxui",\n  "version": "9.8.7"\n}\n' >"$work/plugin/.codex-plugin/plugin.json"
	printf '[package]\nname = "a"\nversion = "9.8.7"\n' >"$work/engine/a/Cargo.toml"
	printf '[package]\nname = "b"\nversion = "9.8.7"\n' >"$work/generators/b/Cargo.toml"
	printf '# Changelog\n\n## Unreleased\n\n## 9.8.7 — 2026-11-24\n\n- one entry\n\n## 9.8.6 — 2026-11-23\n\n- older\n' >"$work/CHANGELOG.md"
}

expect() {
	want=$1
	label=$2
	sh "$work/tools/release-versions.sh" 9.8.7 >"$work.out" 2>&1
	got=$?
	if [ "$want" = pass ] && [ "$got" -eq 0 ]; then
		printf 'ok   %s\n' "$label"
	elif [ "$want" = fail ] && [ "$got" -ne 0 ]; then
		printf 'ok   %s (%s)\n' "$label" "$(head -1 "$work.out")"
	else
		printf 'FAIL %s: expected %s, exit %s\n' "$label" "$want" "$got"
		cat "$work.out"
		fails=$((fails + 1))
	fi
}

fresh
expect pass "plugin, crates and changelog agree"

fresh
sed -i 's/9.8.7/9.8.6/' "$work/plugin/.codex-plugin/plugin.json"
expect fail "a plugin manifest of another version"

fresh
sed -i 's/9.8.7/9.8.6/' "$work/generators/b/Cargo.toml"
expect fail "a crate of another version"

fresh
sed -i 's/^## 9.8.7 — 2026-11-24$/## Unreleased 9.8.7/' "$work/CHANGELOG.md"
expect fail "no dated changelog section"

fresh
notes=$(sh "$work/tools/release-notes.sh" 9.8.7)
if [ "$notes" = "- one entry" ]; then
	printf 'ok   release notes are the section without its heading\n'
else
	printf 'FAIL release notes: got %s\n' "$notes"
	fails=$((fails + 1))
fi

rm -rf "$work" "$work.out"
[ "$fails" -eq 0 ]
