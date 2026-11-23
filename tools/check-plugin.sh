#!/bin/sh
# Checks that the plugin manifests and marketplaces agree and that every skill is complete.
# Usage: sh tools/check-plugin.sh [repo root]. Exit 0 when every check passes, 1 otherwise.
set -u

root=${1:-$(cd "$(dirname "$0")/.." && pwd)}
cd "$root" || exit 1

fails=0
fail() {
	printf 'FAIL %s\n' "$*"
	fails=$((fails + 1))
}

claude_market=.claude-plugin/marketplace.json
codex_market=.agents/plugins/marketplace.json
claude_manifest=plugin/.claude-plugin/plugin.json
codex_manifest=plugin/.codex-plugin/plugin.json

for f in "$claude_market" "$codex_market" "$claude_manifest" "$codex_manifest"; do
	if [ ! -f "$f" ]; then
		fail "$f: missing"
	elif ! jq -e . "$f" >/dev/null 2>&1; then
		fail "$f: not valid JSON"
	fi
done
[ "$fails" -eq 0 ] || exit 1

name=$(jq -r .name "$claude_manifest")
version=$(jq -r .version "$claude_manifest")

check_eq() {
	label=$1
	want=$2
	got=$3
	[ "$got" = "$want" ] || fail "$label is '$got', expected '$want'"
}

check_eq "$codex_manifest .name" "$name" "$(jq -r .name "$codex_manifest")"
check_eq "$codex_manifest .version" "$version" "$(jq -r .version "$codex_manifest")"
check_eq "$claude_market .name" "$name" "$(jq -r .name "$claude_market")"
check_eq "$codex_market .name" "$name" "$(jq -r .name "$codex_market")"
check_eq "$claude_market plugin .name" "$name" "$(jq -r '.plugins[0].name' "$claude_market")"
check_eq "$claude_market plugin .version" "$version" "$(jq -r '.plugins[0].version' "$claude_market")"
check_eq "$claude_market plugin .source" "./plugin" "$(jq -r '.plugins[0].source' "$claude_market")"
check_eq "$codex_market plugin .name" "$name" "$(jq -r '.plugins[0].name' "$codex_market")"
check_eq "$codex_market plugin .source.path" "./plugin" "$(jq -r '.plugins[0].source.path' "$codex_market")"
check_eq "$codex_manifest .skills" "./skills/" "$(jq -r .skills "$codex_manifest")"
check_eq "$claude_market plugin count" "1" "$(jq -r '.plugins | length' "$claude_market")"
check_eq "$codex_market plugin count" "1" "$(jq -r '.plugins | length' "$codex_market")"

for field in description author.name license repository; do
	a=$(jq -r ".$field" "$claude_manifest")
	b=$(jq -r ".$field" "$codex_manifest")
	[ "$a" != null ] || fail "$claude_manifest .$field: missing"
	check_eq "$codex_manifest .$field" "$a" "$b"
done

awk -v v="$version" '$1 == "##" && $2 == v { found = 1 } END { exit !found }' CHANGELOG.md ||
	fail "CHANGELOG.md: no '## $version' heading"

front() {
	awk 'NR == 1 && $0 != "---" { exit } NR > 1 && $0 == "---" { exit } NR > 1 { print }' "$1"
}

skills=0
for dir in plugin/skills/*/; do
	[ -d "$dir" ] || continue
	skills=$((skills + 1))
	dir=${dir%/}
	skill=$(basename "$dir")
	md=$dir/SKILL.md
	if [ ! -f "$md" ]; then
		fail "$dir: no SKILL.md"
		continue
	fi
	fm=$(front "$md")
	sname=$(printf '%s\n' "$fm" | sed -n 's/^name:[[:space:]]*//p' | head -n 1)
	sdesc=$(printf '%s\n' "$fm" | sed -n 's/^description:[[:space:]]*//p' | head -n 1)
	[ -n "$sname" ] || fail "$md: front matter has no name"
	[ -z "$sname" ] || check_eq "$md name" "$skill" "$sname"
	[ -n "$sdesc" ] || fail "$md: front matter has no description"
	refs=$(grep -o 'references/[A-Za-z0-9._/-]*[A-Za-z0-9_-]' "$dir"/SKILL.md "$dir"/references/*.md 2>/dev/null |
		cut -d: -f2- | sort -u)
	for ref in $refs; do
		[ -e "$dir/$ref" ] || fail "$md: names $ref, which does not exist in $dir"
	done
done
[ "$skills" -gt 0 ] || fail "plugin/skills: no skill directory"

if [ "$fails" -gt 0 ]; then
	printf 'check-plugin: %d failure(s)\n' "$fails"
	exit 1
fi
printf 'pass plugin: %s %s, %d skill(s), manifests and marketplaces agree\n' "$name" "$version" "$skills"
