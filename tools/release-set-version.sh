#!/bin/sh
# release-set-version.sh <version> [<date>]: the one command of a release. It writes <version>, a
# bare SemVer (`0.12.0`), into every file that carries the release's version, regenerates the
# pages made from them, and ends with `sh tools/release-versions.sh <version>`, whose exit code is
# its own. The carriers are listed in tools/release-versions.sh; tools/release.test.sh holds the
# two scripts to each other and to that list.
#
# CHANGELOG.md decides what the command does, before it writes a file:
# - <version> is greater than every dated section (the three numbers compared as numbers) and
#   entries stand under `## Unreleased`: a cut. The entries move under a new heading
#   `## <version> — <date>` (<date> as <YYYY-MM-DD>, a day of the calendar; today without the
#   argument), and an empty `## Unreleased` stays above it.
# - <version> is the section directly below `## Unreleased`: entries written under
#   `## Unreleased` since the cut move into that section, in front of the ones it has; the heading
#   and its date stay. With none there, nothing of the changelog changes.
# - <version> is the second dated section, the one directly below `## Unreleased` is greater and
#   nothing stands under `## Unreleased`: the cut above is taken back. Its heading is removed, so
#   its entries stand under `## Unreleased` again, and every byte the cut wrote is as before.
# - Everything else is refused, CHANGELOG.md named, no file written: no entry to release, a version
#   that is not greater than every dated section, a section further down, entries under
#   `## Unreleased` when a cut is to be taken back.
# The command does not know whether a section is committed or tagged: taking back the section
# directly below `## Unreleased` is right only while that cut is not released.
#
# It starts no build and no browser: the lock files get the new version of this repository's own
# packages as text, and no recording is touched (a recorded campaign and an example document name
# the tool they were recorded with). Each file is written beside its place and moved there only
# when its rewrite ended well and is not empty; else the command names the file, leaves it as it
# was and exits 1.
# Exit 0 when every carrier names <version>, 1 naming each that does not (what was written before
# the failure stays written; `git diff` shows it), 2 on a usage error, nothing written.
set -u

usage() {
	echo "usage: sh tools/release-set-version.sh <version> [<date>]: a bare SemVer such as 0.12.0 (no number over 9 digits), and a date as <YYYY-MM-DD> that is a day of the calendar" >&2
	exit 2
}
[ $# -ge 1 ] && [ $# -le 2 ] || usage
version=$1
number='(0|[1-9][0-9]{0,8})'
[ "$(printf '%s\n' "$version" | wc -l)" -eq 1 ] || usage
printf '%s\n' "$version" | grep -qxE "$number\.$number\.$number" || usage
date=${2:-$(date +%Y-%m-%d)}
# The pattern holds the whole text: a date of two lines is none.
case $date in
[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]) ;;
*) usage ;;
esac
printf '%s\n' "$date" | awk -F- '{
	year = $1 + 0; month = $2 + 0; day = $3 + 0
	days = 31
	if (month == 4 || month == 6 || month == 9 || month == 11) days = 30
	if (month == 2) days = (year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)) ? 29 : 28
	exit !(year >= 1 && month >= 1 && month <= 12 && day >= 1 && day <= days)
}' || usage

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root" || exit 1

# What CHANGELOG.md asks for: `cut`, `move`, `keep`, `back <start of the heading to remove>`, or
# `refuse <why>`.
if [ ! -f CHANGELOG.md ]; then
	echo "CHANGELOG.md: no such file, release $version" >&2
	exit 1
fi
plan=$(awk -v version="$version" '
	function part(text, i,   parts) { split(text, parts, "."); return parts[i] + 0 }
	function greater(a, b,   i) {
		for (i = 1; i <= 3; i++) {
			if (part(a, i) > part(b, i)) return 1
			if (part(a, i) < part(b, i)) return 0
		}
		return 0
	}
	/^## / {
		section = ""
		if ($0 == "## Unreleased" && !unreleased) {
			unreleased = 1
			section = "unreleased"
		} else if ($0 ~ /^## [0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]* — /) {
			dated[++n] = $2
			section = $2
			if (!unreleased) early = 1
		}
		next
	}
	/[^[:space:]]/ { if (section != "") entries[section]++ }
	END {
		if (!unreleased) { print "refuse no heading `## Unreleased`"; exit }
		if (early) { print "refuse a dated section stands above `## Unreleased`"; exit }
		if (n >= 1 && dated[1] == version) {
			if (entries["unreleased"]) print "move"
			else if (entries[version]) print "keep"
			else print "refuse no entry under `## Unreleased` and none in the section " version
			exit
		}
		if (n >= 2 && dated[2] == version) {
			if (!greater(dated[1], version))
				print "refuse the section above " version " is " dated[1] ", which is not greater: no cut to take back"
			else if (entries["unreleased"])
				print "refuse entries stand under `## Unreleased`: taking " dated[1] " back would mix them with its own"
			else
				print "back ## " dated[1] " — "
			exit
		}
		for (i = 1; i <= n; i++)
			if (!greater(version, dated[i])) {
				print "refuse " version " is not greater than the section " dated[i] "; only the cut directly below `## Unreleased` is taken back"
				exit
			}
		if (!entries["unreleased"]) { print "refuse no entry under `## Unreleased`, release " version; exit }
		print "cut"
	}' CHANGELOG.md)
case $plan in
cut | move | keep | "back "*) ;;
"refuse "*)
	echo "CHANGELOG.md: ${plan#refuse }" >&2
	exit 1
	;;
*)
	echo "CHANGELOG.md: could not be read, release $version" >&2
	exit 1
	;;
esac

# apply <file> <filter> [<argument>...]: the filter, called with the file as its last argument,
# prints the file's new text. It is written beside the file and takes its place only when the
# filter exited 0 and printed something; a file that is not there stays away (the check at the
# end names it).
apply() {
	file=$1
	shift
	[ -f "$file" ] || return 0
	new=$file.release-set-version.$$
	if cp -p "$file" "$new" && "$@" "$file" >"$new" && [ -s "$new" ]; then
		if cmp -s "$new" "$file"; then
			rm -f "$new"
			return 0
		fi
		mv "$new" "$file" && return 0
	fi
	rm -f "$new"
	echo "$file: could not be rewritten, the file is as it was" >&2
	exit 1
}

for file in plugin/.claude-plugin/plugin.json plugin/.codex-plugin/plugin.json .claude-plugin/marketplace.json; do
	apply "$file" sed 's/^\( *"version": *"\)[^"]*"/\1'"$version"'"/'
done

# A crate's version is the first `version` of its `[package]` table, no other line of the file.
for crate in engine/*/Cargo.toml generators/*/Cargo.toml; do
	apply "$crate" awk -v version="$version" '
		/^\[/ { package = ($0 == "[package]") }
		package && !done && /^version *= *"/ { $0 = "version = \"" version "\""; done = 1 }
		{ print }'
done

# The packages of this repository in a lock file have no `source`: their versions are the
# crates', and nothing else of the file moves.
for lock in Cargo.lock engine/search/Cargo.lock; do
	apply "$lock" awk -v version="$version" '
		{ line[NR] = $0 }
		END {
			for (i = 1; i < NR; i++)
				if (line[i] ~ /^name = "/ && line[i + 1] ~ /^version = "/ && line[i + 2] !~ /^source = /)
					line[i + 1] = "version = \"" version "\""
			for (i = 1; i <= NR; i++) print line[i]
		}'
done

apply website/docs/index.md sed 's/plugin version `[^`]*`/plugin version `'"$version"'`/'
apply website/data/status.json sed 's/\("detail": "Plugin \)[0-9][^ ]*\( for Claude Code and Codex\)/\1'"$version"'\2/'
apply docs/roadmap.md sed 's/\(The current release is \)[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*/\1'"$version"'/'

# changelog <take> <heading> <place>: <take> is the start of a heading to take out, with one blank
# line beside it (the one below it, else the one above). With <place> `yes` a heading is put under
# `## Unreleased`, a blank line above it: <heading>, or with an empty <heading> the one taken out,
# whose entries then join the ones that come to stand above them.
changelog() {
	apply CHANGELOG.md awk -v take="$1" -v heading="$2" -v place="$3" '
		{ line[NR] = $0 }
		END {
			at = 0
			if (take != "")
				for (i = 1; i <= NR && !at; i++)
					if (index(line[i], take) == 1) at = i
			if (at) {
				gone[at] = 1
				below = at + 1
				if (at < NR && line[at + 1] == "") {
					gone[at + 1] = 1
					below = at + 2
				} else if (at > 1 && line[at - 1] == "")
					gone[at - 1] = 1
				if (heading == "") {
					heading = line[at]
					if (at > 1 && line[at - 1] == "" && below <= NR && line[below] != "" && line[below] !~ /^## /)
						gone[at - 1] = 1
				}
			}
			for (i = 1; i <= NR; i++) {
				if (gone[i]) continue
				print line[i]
				if (place == "yes" && !placed && line[i] == "## Unreleased") {
					print ""
					print heading
					placed = 1
				}
			}
		}'
}
case $plan in
cut) changelog "" "## $version — $date" yes ;;
move) changelog "## $version — " "" yes ;;
"back "*) changelog "${plan#back }" "-" no ;;
esac

if ! DOCS_ROOT=$root sh tools/docs.sh generate; then
	echo "tools/docs.sh generate: failed, release $version" >&2
	exit 1
fi

if sh tools/release-versions.sh "$version"; then
	echo "release-set-version: every carrier names $version; sh tools/release-versions.sh $version exits 0"
else
	echo "release-set-version: $version is not in every carrier, see above" >&2
	exit 1
fi
