#!/bin/sh
# Proves the release tools on a copy of the files a release touches, under target/release-test/:
# - tools/release-set-version.sh sets a version in every place that carries it, and
#   tools/release-versions.sh then passes for that version and fails for the one before;
# - release-versions.sh fails when any one file the command wrote is put back as it was, one case
#   per file. The files are found by comparing the copy before and after the command, so a place
#   the command writes and the check does not hold fails here; and the files the command wrote
#   are the carriers listed here, no other;
# - the command writes the lines it is to write and no other: in a lock file the versions of the
#   packages without a `source` (the copy holds a package of another repository at the version
#   being replaced), in a crate manifest the version of the package (the copy holds a second
#   `version` line), and no recording (the copy holds a recorded campaign and the two example
#   documents);
# - setting the first version again gives every byte back, and setting a version twice changes
#   nothing;
# - the command refuses, with nothing written, a version that is no bare SemVer, a date that is no
#   day of the calendar, a release without an entry, a version that is not greater than every
#   dated section, and a cut that cannot be taken back; it fails on a carrier whose line is gone,
#   and leaves a file as it was when its rewrite fails;
# - release-versions.sh holds the built binaries to the release, and tools/release-notes.sh prints
#   the section of a release;
# - this tree carries its own version in every place.
# The copy holds the tree's own files, with these exceptions: a changelog of three sections, so no
# case depends on what stands under `## Unreleased` today, and the two additions named above. No
# cargo, no build.
# Exit 0 when every case holds.
set -u

root=$(cd "$(dirname "$0")/.." && pwd)
work=$root/target/release-test
seed=$work/seed
set=$work/set
tree=$work/tree
kept=$work/kept
out=$work/out
fails=0
cases=0

ok() {
	cases=$((cases + 1))
	printf 'ok   %s\n' "$1"
}
bad() {
	cases=$((cases + 1))
	fails=$((fails + 1))
	printf 'FAIL %s\n' "$1"
	[ ! -f "$out" ] || cat "$out"
}

# The version this tree carries: the copy starts there and goes back there.
first=$(sed -n 's/^ *"version": *"\([^"]*\)".*/\1/p' "$root/plugin/.claude-plugin/plugin.json" | head -1)

changelog() {
	printf '# Changelog\n\n## Unreleased\n\n%s## %s — 2026-11-23\n\n- older\n' "$1" "$first"
}

recorded=engine/search/tests/fixtures/docs-landing/measurements.json

rm -rf "$work"
mkdir -p "$seed/tools" "$seed/.claude-plugin" "$seed/docs" "$seed/ess/ui" "$seed/website" \
	"$seed/engine/browser/examples" "$seed/$(dirname "$recorded")"
cp "$root/tools/release-set-version.sh" "$root/tools/release-versions.sh" \
	"$root/tools/release-notes.sh" "$root/tools/docs.sh" "$seed/tools/"
cp -R "$root/plugin" "$seed/"
cp "$root/.claude-plugin/marketplace.json" "$seed/.claude-plugin/"
for crate in "$root"/engine/*/Cargo.toml "$root"/generators/*/Cargo.toml; do
	directory=$(dirname "${crate#"$root"/}")
	mkdir -p "$seed/$directory"
	cp "$crate" "$seed/$directory/"
done
cp "$root/Cargo.lock" "$seed/"
cp "$root/engine/search/Cargo.lock" "$seed/engine/search/"
cp "$root"/engine/browser/examples/*.json "$seed/engine/browser/examples/"
cp "$root/$recorded" "$seed/$recorded"
cp "$root/docs/metrics.md" "$root/docs/brief.md" "$root/docs/roadmap.md" "$seed/docs/"
cp "$root/ess/system.yaml" "$seed/ess/"
cp -R "$root/ess/domains" "$seed/ess/"
cp "$root/ess/ui/studio.ui.yaml" "$seed/ess/ui/"
cp -R "$root/website/docs" "$root/website/data" "$root/website/product.json" "$seed/website/"
changelog '- one entry
- another entry

' >"$seed/CHANGELOG.md"
# A package of another repository at the version this tree carries, in each lock file, and a
# second `version` line in a crate manifest: neither is the release's to write.
for lock in Cargo.lock engine/search/Cargo.lock; do
	printf '\n[[package]]\nname = "release-test-third-party"\nversion = "%s"\nsource = "registry+https://example.invalid/index"\nchecksum = "00"\n' "$first" >>"$seed/$lock"
done
printf '\n[dependencies.release-test-dependency]\nversion = "%s"\n' "$first" >>"$seed/generators/own/Cargo.toml"

# fresh [<directory>]: the working copy is that state again (default: the copy with 9.8.7 set).
fresh() {
	rm -rf "$tree"
	cp -R "${1:-$set}" "$tree"
}

# keep: the working copy as it is now, to compare with after a command that must write nothing.
keep() {
	rm -rf "$kept"
	cp -R "$tree" "$kept"
}

# same <directory>: the working copy holds its files, byte for byte, and no other.
same() {
	diff -r "$1" "$tree" >>"$out" 2>&1
}

# run <argument>...: the command on the working copy; its exit code is $got.
run() {
	sh "$tree/tools/release-set-version.sh" "$@" >"$out" 2>&1
	got=$?
}

# refused <label> <exit code, or `fail` for any but 0> [<file>]: the command that ran last (run)
# was refused, named the file, and the working copy is the one kept before it (keep).
refused() {
	if [ "$2" = fail ]; then
		[ "$got" -ne 0 ]
	else
		[ "$got" -eq "$2" ]
	fi &&
		{ [ -z "${3:-}" ] || grep -qF -- "$3: " "$out"; } && same "$kept"
	if [ $? -eq 0 ]; then
		ok "$1, nothing written${3:+ ($(grep -F -- "$3: " "$out" | head -1))}"
	else
		bad "$1: exit $got; expected ${2}${3:+, the file $3 named} and no file changed"
	fi
}

# under_unreleased <line>: the line stands under `## Unreleased` of the working copy, a blank line
# above it.
under_unreleased() {
	awk -v entry="$1" '{ print } $0 == "## Unreleased" { print ""; print entry }' "$tree/CHANGELOG.md" >"$work/page" &&
		cp "$work/page" "$tree/CHANGELOG.md"
}

# differing <file> <file>: the numbers of the lines that differ, and `length` when one is longer.
differing() {
	awk 'NR == FNR { before[FNR] = $0; n = FNR; next }
		{ if (FNR > n || $0 != before[FNR]) print FNR; m = FNR }
		END { if (m != n) print "length" }' "$1" "$2"
}

# The command.

cp -R "$seed" "$set"
sh "$set/tools/release-set-version.sh" 9.8.7 2026-11-24 >"$out" 2>&1
got=$?
if [ "$got" -eq 0 ]; then
	ok "the command sets 9.8.7 from $first: exit 0"
else
	bad "the command sets 9.8.7 from $first: exit $got"
fi

expect() {
	want=$1
	label=$2
	shift 2
	sh "$tree/tools/release-versions.sh" 9.8.7 "$@" >"$out" 2>&1
	got=$?
	if [ "$want" = pass ] && [ "$got" -eq 0 ]; then
		ok "$label"
	elif [ "$want" = fail ] && [ "$got" -ne 0 ]; then
		ok "$label ($(head -1 "$out"))"
	else
		bad "$label: expected $want, exit $got"
	fi
}

# disagrees <label> <file>: release-versions.sh fails on the working copy and names the file.
disagrees() {
	sh "$tree/tools/release-versions.sh" 9.8.7 >"$out" 2>&1
	got=$?
	if [ "$got" -ne 0 ] && grep -qF -- "$2: " "$out"; then
		ok "$1 ($(grep -F -- "$2: " "$out" | head -1))"
	else
		bad "$1: exit $got; the check must fail and name $2"
	fi
}

fresh
expect pass "plugin, crates, changelog and every other carrier agree after the command"

fresh
sh "$tree/tools/release-versions.sh" "$first" >"$out" 2>&1
got=$?
if [ "$got" -ne 0 ]; then
	ok "the version before no longer passes ($(wc -l <"$out" | tr -d ' ') disagreements)"
else
	bad "the version before, $first, still passes after the command set 9.8.7"
fi

# One case per file the command wrote, added or removed: put back as it was, the check fails and
# names it.
changed=$({
	cd "$set" && find . -type f
	cd "$seed" && find . -type f
} | sed 's|^\./||' | LC_ALL=C sort -u | while IFS= read -r file; do
	cmp -s "$seed/$file" "$set/$file" || printf '%s\n' "$file"
done)
written=0
fresh
for file in $changed; do
	written=$((written + 1))
	if [ -f "$seed/$file" ]; then
		cp "$seed/$file" "$tree/$file"
	else
		rm -f "$tree/$file"
	fi
	disagrees "$file as it was before fails the check" "$file"
	# The working copy is the one with 9.8.7 set again for the next file.
	if [ -f "$set/$file" ]; then
		cp "$set/$file" "$tree/$file"
	else
		rm -f "$tree/$file"
	fi
done
if [ "$written" -gt 0 ] && printf '%s\n' "$changed" | grep -qx 'plugin/.claude-plugin/plugin.json'; then
	ok "the command wrote $written files, each held by the check"
else
	bad "the command wrote $written files and the plugin manifest is not among them: the cases above checked nothing"
fi

# The carriers: the files a release writes. A file the command wrote that is not here, and a file
# here that it did not write, fails.
carriers=$({
	printf '%s\n' .claude-plugin/marketplace.json CHANGELOG.md Cargo.lock docs/roadmap.md \
		engine/search/Cargo.lock plugin/.claude-plugin/plugin.json plugin/.codex-plugin/plugin.json \
		website/data/status.json website/docs/index.md website/docs/reference/skill.md \
		website/docs/roadmap.md website/docs/status.md
	cd "$seed" && ls engine/*/Cargo.toml generators/*/Cargo.toml
} | LC_ALL=C sort)
if [ "$changed" = "$carriers" ]; then
	ok "the files the command wrote are the $written carriers and no other"
else
	{
		echo "written by the command:"
		printf '%s\n' "$changed"
		echo "the carriers:"
		printf '%s\n' "$carriers"
	} >"$out"
	bad "the files the command wrote are not the carriers"
fi

for file in "$recorded" engine/browser/examples/llm-docs.evidence.json engine/browser/examples/uxui-landing.render.json website/product.json; do
	if [ -s "$seed/$file" ] && cmp -s "$seed/$file" "$set/$file"; then
		ok "the command leaves $file, a recording or a source of one"
	else
		rm -f "$out"
		bad "the command wrote $file, a recording or a source of one"
	fi
done

# own_versions <lock file>: the numbers of the lines that hold the version of a package without a
# `source`, read package by package.
own_versions() {
	awk 'function package_ends() { if (at && !third) print at; at = 0; third = 0 }
		/^\[\[package\]\]$/ { package_ends(); next }
		/^version = "/ && !at { at = NR }
		/^source = / { third = 1 }
		END { package_ends() }' "$1"
}
for lock in Cargo.lock engine/search/Cargo.lock; do
	own=$(own_versions "$seed/$lock")
	moved=$(differing "$seed/$lock" "$set/$lock")
	other=$(for at in $moved; do sed -n "${at}p" "$set/$lock" 2>&1; done | grep -vx 'version = "9.8.7"')
	if [ -n "$own" ] && [ "$own" = "$moved" ] && [ -z "$other" ]; then
		ok "$lock: the command wrote the versions of the $(printf '%s\n' "$own" | wc -l | tr -d ' ') packages without a source and no other line"
	else
		{
			echo "lines of the packages without a source: $(echo $own)"
			echo "lines the command changed: $(echo $moved)"
			echo "changed to something else than the release: $other"
		} >"$out"
		bad "$lock: the command must change the versions of the packages without a source, and nothing else"
	fi
done

manifest=generators/own/Cargo.toml
moved=$(differing "$seed/$manifest" "$set/$manifest")
if [ "$(printf '%s\n' "$moved" | wc -l)" -eq 1 ] && [ "$moved" != length ] &&
	[ "$(tail -1 "$set/$manifest")" = "version = \"$first\"" ]; then
	ok "$manifest: the command wrote the version of the package and left the second version line"
else
	diff "$seed/$manifest" "$set/$manifest" >"$out" 2>&1
	bad "$manifest: the command must change one line, the version of the package"
fi

fresh
run "$first"
if [ "$got" -eq 0 ] && same "$seed"; then
	ok "setting $first again gives every byte back"
else
	bad "setting $first again: exit $got, and the copy must be the one before the command"
fi

fresh
run 9.8.7 2026-11-25
if [ "$got" -eq 0 ] && same "$set"; then
	ok "setting the version it has changes nothing"
else
	bad "setting 9.8.7 a second time: exit $got, and no file may change"
fi

for version in '' 1.2 v1.2.3 1.2.3-rc.1 '1.2.3 4' 9.8.1234567890; do
	fresh
	: >"$out"
	sh "$tree/tools/release-set-version.sh" "$version" >>"$out" 2>&1
	got=$?
	if [ "$got" -eq 2 ] && same "$set"; then
		ok "'$version' is refused as no bare SemVer, nothing written"
	else
		bad "'$version': exit $got; expected 2 and no file changed"
	fi
done

fresh
: >"$out"
sh "$tree/tools/release-set-version.sh" 9.8.8 24.11.2026 >>"$out" 2>&1
got=$?
if [ "$got" -eq 2 ] && same "$set"; then
	ok "a date that is not <YYYY-MM-DD> is refused, nothing written"
else
	bad "the date 24.11.2026: exit $got; expected 2 and no file changed"
fi

# Defect: a date of the right shape that is no day of a calendar becomes the section's heading.
fresh
: >"$out"
sh "$tree/tools/release-set-version.sh" 9.8.8 2026-13-45 >>"$out" 2>&1
got=$?
if [ "$got" -eq 2 ] && same "$set"; then
	ok "a date that is no day of a calendar is refused, nothing written"
else
	grep -n '^## 9.8.8 ' "$tree/CHANGELOG.md" >>"$out" 2>&1
	bad "the date 2026-13-45: exit $got; expected 2 and no file changed"
fi

# The working copy has an entry to release, so only the date stands between it and a cut.
for date in 2026-02-30 2027-02-29 0000-01-01 2026-00-10 '2026-11-24
- a line of its own'; do
	fresh
	under_unreleased '- an entry to release'
	keep
	run 9.8.8 "$date"
	refused "the date '$(printf '%s' "$date" | tr '\n' '|')' is no day of the calendar on one line" 2
done

fresh "$seed"
sed 's/plugin version `[^`]*`/the plugin/' "$tree/website/docs/index.md" >"$work/page" &&
	cp "$work/page" "$tree/website/docs/index.md"
run 9.8.7 2026-11-24
if [ "$got" -ne 0 ] && grep -qF 'website/docs/index.md: ' "$out"; then
	ok "a carrier whose line is gone fails the command ($(grep -F 'website/docs/index.md: ' "$out" | head -1))"
else
	bad "a page without its version line: exit $got; the command must fail and name it"
fi

# A rewrite that fails.

# stand_in <tool> <pattern> <commands>: a tool of that name under $work/stand-in that runs the
# commands when its last argument matches the pattern, and is the real tool for every other call.
stand_in() {
	rm -rf "$work/stand-in"
	mkdir -p "$work/stand-in"
	{
		printf '#!/bin/sh\n'
		printf 'for argument in "$@"; do last=$argument; done\n'
		printf 'case ${last:-} in\n%s) %s ;;\nesac\n' "$2" "$3"
		printf 'exec %s "$@"\n' "$(command -v "$1")"
	} >"$work/stand-in/$1"
	chmod +x "$work/stand-in/$1"
}

# failed_rewrite <label> <file>: the command, run with the stand-in first in PATH, failed, named
# the file, left it as it was, emptied no file and left no file of its own behind.
failed_rewrite() {
	PATH=$work/stand-in:$PATH sh "$tree/tools/release-set-version.sh" 9.8.7 2026-11-24 >"$out" 2>&1
	got=$?
	left=$(find "$tree" -name '*.release-set-version.*')
	emptied=$(cd "$tree" && find . -type f -size 0 | while IFS= read -r file; do
		[ ! -s "$seed/$file" ] || printf '%s\n' "$file"
	done)
	if [ "$got" -ne 0 ] && grep -q "^$2: " "$out" && [ -z "$left" ] && [ -z "$emptied" ] && cmp -s "$seed/$2" "$tree/$2"; then
		ok "$1 ($(grep "^$2: " "$out" | head -1))"
	else
		echo "left behind: $left; emptied: $emptied" >>"$out"
		bad "$1: exit $got; the command must fail, name $2, leave it as it was and empty no file"
	fi
}

fresh "$seed"
stand_in awk '*Cargo.lock' 'echo partial; exit 1'
failed_rewrite "a filter that fails after part of its output leaves the file as it was" Cargo.lock

fresh "$seed"
stand_in sed '*website/docs/index.md' 'exit 0'
failed_rewrite "a filter that prints nothing leaves the file as it was" website/docs/index.md

# The changelog.

fresh
changelog '## 9.8.7 — 2026-11-24

- one entry
- another entry

' >"$work/expected"
if cmp -s "$work/expected" "$tree/CHANGELOG.md"; then
	ok "the entries under Unreleased stand under the dated heading, an empty Unreleased above it"
else
	diff "$work/expected" "$tree/CHANGELOG.md" >"$out" 2>&1
	bad "CHANGELOG.md after the command is not the expected text"
fi

fresh "$seed"
changelog '' >"$tree/CHANGELOG.md"
keep
run 9.8.7 2026-11-24
refused "a release without an entry under Unreleased is refused" fail CHANGELOG.md

# Defect: the refused release above leaves an empty dated section, and the command never fills it:
# with the missing entry written under Unreleased, the same command fails again.
fresh "$seed"
changelog '' >"$tree/CHANGELOG.md"
sh "$tree/tools/release-set-version.sh" 9.8.7 2026-11-24 >/dev/null 2>&1
awk '{ print } $0 == "## Unreleased" { print ""; print "- the missing entry" }' "$tree/CHANGELOG.md" >"$work/page" &&
	cp "$work/page" "$tree/CHANGELOG.md"
sh "$tree/tools/release-set-version.sh" 9.8.7 2026-11-24 >"$out" 2>&1
got=$?
notes=$(sh "$tree/tools/release-notes.sh" 9.8.7)
if [ "$got" -eq 0 ] && [ "$notes" = "- the missing entry" ]; then
	ok "after a release refused for its empty Unreleased, the entry and the command again cut the release"
else
	sed -n '1,8p' "$tree/CHANGELOG.md" >>"$out" 2>&1
	bad "an entry under Unreleased after a refused release: exit $got, notes '$notes'; expected exit 0 and the entry under ## 9.8.7"
fi

# An entry written under Unreleased after the cut goes into the release with the command again.
# The date is a leap day: a day of the calendar, and not the section's, which keeps its own.
fresh
under_unreleased '- a late entry'
run 9.8.7 2028-02-29
changelog '## 9.8.7 — 2026-11-24

- a late entry
- one entry
- another entry

' >"$work/expected"
if [ "$got" -eq 0 ] && cmp -s "$work/expected" "$tree/CHANGELOG.md"; then
	ok "an entry under Unreleased after the cut moves into the version's section, in front of its entries"
else
	diff "$work/expected" "$tree/CHANGELOG.md" >>"$out" 2>&1
	bad "an entry under Unreleased after the cut: exit $got, and CHANGELOG.md must hold it under ## 9.8.7"
fi

# A dated section without an entry directly below Unreleased: refused with nothing to release,
# filled with what stands under Unreleased.
empty_section() {
	printf '# Changelog\n\n## Unreleased\n\n%s## 9.8.7 — 2026-11-24\n\n## %s — 2026-11-23\n\n- older\n' "$1" "$first"
}
fresh "$seed"
empty_section '' >"$tree/CHANGELOG.md"
keep
run 9.8.7 2026-11-24
refused "a section without an entry and nothing under Unreleased is refused" fail CHANGELOG.md

fresh "$seed"
empty_section '- the missing entry

' >"$tree/CHANGELOG.md"
run 9.8.7 2026-11-24
changelog '## 9.8.7 — 2026-11-24

- the missing entry

' >"$work/expected"
if [ "$got" -eq 0 ] && cmp -s "$work/expected" "$tree/CHANGELOG.md"; then
	ok "a section without an entry takes the entries under Unreleased"
else
	diff "$work/expected" "$tree/CHANGELOG.md" >>"$out" 2>&1
	bad "a section without an entry and an entry under Unreleased: exit $got, and the entry must stand under ## 9.8.7"
fi

# The order of versions.

fresh
printf '# Changelog\n\n## Unreleased\n\n## 9.8.8 — 2026-11-25\n\n- newest\n\n## 9.8.7 — 2026-11-24\n\n- one entry\n\n## %s — 2026-11-23\n\n- older\n' "$first" >"$tree/CHANGELOG.md"
keep
run "$first"
refused "only the cut directly below Unreleased is taken back: two dated sections above the version" fail CHANGELOG.md

fresh
under_unreleased '- an entry to release'
keep
run 0.0.1 2026-11-25
refused "a version below the newest dated section is refused" fail CHANGELOG.md

fresh
under_unreleased '- an entry written after the cut'
keep
run "$first"
refused "a cut is not taken back over entries under Unreleased" fail CHANGELOG.md

# The three numbers are compared as numbers. 9.9.1 is not greater than 9.10.0: no cut. And 9.10.0
# is greater than 9.9.0: it is the cut above 9.9.0, which the entry under Unreleased keeps from
# being taken back (compared as text, 9.10.0 would be no cut above 9.9.0 at all).
fresh
printf '# Changelog\n\n## Unreleased\n\n- an entry to release\n\n## 9.10.0 — 2026-11-24\n\n- one entry\n\n## 9.9.0 — 2026-11-23\n\n- older\n' >"$tree/CHANGELOG.md"
keep
run 9.9.1 2026-11-25
refused "9.9.1 is refused above 9.10.0" fail CHANGELOG.md
run 9.9.0
if grep -q '^CHANGELOG.md: .*taking 9.10.0 back' "$out"; then
	refused "9.10.0 is the cut above 9.9.0, not taken back over an entry under Unreleased" fail CHANGELOG.md
else
	bad "9.9.0 below 9.10.0 with an entry under Unreleased: exit $got; the refusal must name the cut 9.10.0 it does not take back"
fi

# The first entry directly under the heading, no blank line between: the cut, dated today without
# a date, and its way back.
fresh "$seed"
printf '# Changelog\n\n## Unreleased\n- one entry\n\n## %s — 2026-11-23\n\n- older\n' "$first" >"$tree/CHANGELOG.md"
keep
before=$(date +%Y-%m-%d)
run 9.8.7
cut=$got
after=$(date +%Y-%m-%d)
notes=$(sh "$tree/tools/release-notes.sh" 9.8.7)
if [ "$cut" -eq 0 ] && { grep -qx "## 9.8.7 — $before" "$tree/CHANGELOG.md" || grep -qx "## 9.8.7 — $after" "$tree/CHANGELOG.md"; }; then
	ok "without a date the section is dated today"
else
	bad "without a date: exit $cut, and CHANGELOG.md must have the heading ## 9.8.7 — $after"
fi
run "$first"
if [ "$cut" -eq 0 ] && [ "$notes" = "- one entry" ] && [ "$got" -eq 0 ] && same "$kept"; then
	ok "a cut and its way back give every byte back when no blank line follows the heading Unreleased"
else
	bad "no blank line under Unreleased: the cut exit $cut with notes '$notes', the way back exit $got; every byte must be as before"
fi

# The named disagreements of release-versions.sh.

fresh
sed -i 's/9.8.7/9.8.6/' "$tree/plugin/.codex-plugin/plugin.json"
disagrees "a plugin manifest of another version" plugin/.codex-plugin/plugin.json

fresh
sed -i 's/9.8.7/9.8.6/' "$tree/generators/own/Cargo.toml"
disagrees "a crate of another version" generators/own/Cargo.toml

fresh
awk '$0 == "version = \"9.8.7\"" && ++n == 2 { $0 = "version = \"9.8.6\"" } { print }' "$tree/Cargo.lock" >"$work/page" &&
	cp "$work/page" "$tree/Cargo.lock"
if [ "$(differing "$set/Cargo.lock" "$tree/Cargo.lock" | wc -l)" -eq 1 ]; then
	disagrees "the second package of this repository in a lock file at another version" Cargo.lock
else
	rm -f "$out"
	bad "Cargo.lock has no second package of this repository: the case checked nothing"
fi

fresh
sed -i 's/^## 9.8.7 — 2026-11-24$/## Unreleased 9.8.7/' "$tree/CHANGELOG.md"
disagrees "no dated changelog section" CHANGELOG.md

fresh
rm "$tree/website/docs/status.md"
disagrees "a carrier that is no longer there" website/docs/status.md

# The built binaries.

# bin <name> <answer> [<exit>]: a stand-in for a built binary under $tree/bin that prints <answer>
# (a printf format) to `--version` and exits <exit> (default 0).
bin() {
	mkdir -p "$tree/bin"
	printf '#!/bin/sh\n[ "$1" = --version ] || exit 2\nprintf '\''%s'\''\nexit %s\n' "$2" "${3:-0}" >"$tree/bin/$1"
	chmod +x "$tree/bin/$1"
}

bins() {
	bin ux-browser 'ux-browser 9.8.7\n'
	bin ux-static 'ux-static 9.8.7\n'
}

fresh
bins
expect pass "the built binaries answer the release" "$tree/bin"

fresh
bins
bin ux-static 'ux-static 9.8.6\n'
expect fail "a built binary answers another version" "$tree/bin"

fresh
bins
bin ux-browser 'ux-browser 9.8.7\nmore\n'
expect fail "a built binary answers more than its line" "$tree/bin"

fresh
bins
bin ux-browser 'ux-browser 9.8.7'
expect fail "a built binary answers without the newline" "$tree/bin"

fresh
bins
bin ux-browser 'ux-static 9.8.7\n'
expect fail "a built binary answers another name" "$tree/bin"

fresh
bins
bin ux-browser 'ux-browser 9.8.7\n' 1
expect fail "a built binary exits non-zero" "$tree/bin"

fresh
bins
rm "$tree/bin/ux-static"
expect fail "a built binary is missing" "$tree/bin"

fresh
expect fail "the binary directory does not exist" "$tree/no-such-dir"

# The notes.

fresh
notes=$(sh "$tree/tools/release-notes.sh" 9.8.7)
if [ "$notes" = "- one entry
- another entry" ]; then
	ok "release notes are the section without its heading"
else
	rm -f "$out"
	bad "release notes: got $notes"
fi

# This tree.

sh "$root/tools/release-versions.sh" "$first" >"$out" 2>&1
got=$?
if [ "$got" -eq 0 ]; then
	ok "this tree carries $first in every place"
else
	bad "this tree does not carry $first in every place: exit $got"
fi

rm -rf "$work"
if [ "$fails" -gt 0 ]; then
	printf 'release.test: %d of %d case(s) failed\n' "$fails" "$cases"
	exit 1
fi
printf 'pass release.test: %d cases\n' "$cases"
