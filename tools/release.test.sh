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

# bin <name> <answer> [<exit>]: a stand-in for a built binary under $work/bin that prints <answer>
# (a printf format) to `--version` and exits <exit> (default 0).
bin() {
	mkdir -p "$work/bin"
	printf '#!/bin/sh\n[ "$1" = --version ] || exit 2\nprintf '\''%s'\''\nexit %s\n' "$2" "${3:-0}" >"$work/bin/$1"
	chmod +x "$work/bin/$1"
}

bins() {
	bin ux-browser 'ux-browser 9.8.7\n'
	bin ux-static 'ux-static 9.8.7\n'
}

expect() {
	want=$1
	label=$2
	shift 2
	sh "$work/tools/release-versions.sh" 9.8.7 "$@" >"$work.out" 2>&1
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
bins
expect pass "the built binaries answer the release" "$work/bin"

fresh
bins
bin ux-static 'ux-static 9.8.6\n'
expect fail "a built binary answers another version" "$work/bin"

fresh
bins
bin ux-browser 'ux-browser 9.8.7\nmore\n'
expect fail "a built binary answers more than its line" "$work/bin"

fresh
bins
bin ux-browser 'ux-browser 9.8.7'
expect fail "a built binary answers without the newline" "$work/bin"

fresh
bins
bin ux-browser 'ux-static 9.8.7\n'
expect fail "a built binary answers another name" "$work/bin"

fresh
bins
bin ux-browser 'ux-browser 9.8.7\n' 1
expect fail "a built binary exits non-zero" "$work/bin"

fresh
bins
rm "$work/bin/ux-static"
expect fail "a built binary is missing" "$work/bin"

fresh
expect fail "the binary directory does not exist" "$work/no-such-dir"

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
