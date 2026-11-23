#!/bin/sh
# Proves that tools/check-plugin.sh passes on this tree and fails on each broken copy of it.
# Copies go under target/check-plugin-test/ in the repository. Exit 0 when every case holds.
set -u

root=$(cd "$(dirname "$0")/.." && pwd)
work=$root/target/check-plugin-test
check=$root/tools/check-plugin.sh
fails=0

fresh() {
	rm -rf "$work"
	mkdir -p "$work"
	cp -r "$root/.claude-plugin" "$root/.agents" "$root/plugin" "$root/CHANGELOG.md" "$work/"
}

expect() {
	want=$1
	label=$2
	sh "$check" "$work" >"$work.out" 2>&1
	got=$?
	if [ "$want" = pass ] && [ "$got" -eq 0 ]; then
		printf 'ok   %s\n' "$label"
	elif [ "$want" = fail ] && [ "$got" -ne 0 ]; then
		printf 'ok   %s (%s)\n' "$label" "$(grep -m 1 '^FAIL' "$work.out")"
	else
		printf 'FAIL %s: expected %s, exit %s\n' "$label" "$want" "$got"
		cat "$work.out"
		fails=$((fails + 1))
	fi
}

edit_json() {
	file=$1
	filter=$2
	jq "$filter" "$work/$file" >"$work/$file.new" && mv "$work/$file.new" "$work/$file"
}

skill=plugin/skills/ui-iteration-loop

fresh
expect pass "unchanged tree"

fresh
edit_json plugin/.codex-plugin/plugin.json '.version = "9.9.9"'
expect fail "codex manifest version differs"

fresh
edit_json .claude-plugin/marketplace.json '.plugins[0].version = "9.9.9"'
expect fail "marketplace version differs"

fresh
edit_json .agents/plugins/marketplace.json '.plugins[0].name = "other"'
expect fail "codex marketplace plugin name differs"

fresh
sed '/^description:/d' "$work/$skill/SKILL.md" >"$work/SKILL.md" && mv "$work/SKILL.md" "$work/$skill/SKILL.md"
expect fail "skill without description"

fresh
rm "$work/$skill/references/heuristics.md"
expect fail "named reference missing"

fresh
mkdir "$work/plugin/skills/empty"
expect fail "skill directory without SKILL.md"

fresh
printf '{' >"$work/plugin/.claude-plugin/plugin.json"
expect fail "manifest not JSON"

rm -rf "$work" "$work.out"
if [ "$fails" -gt 0 ]; then
	printf 'check-plugin.test: %d case(s) failed\n' "$fails"
	exit 1
fi
printf 'pass check-plugin.test: 8 cases\n'
