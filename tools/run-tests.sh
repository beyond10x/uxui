#!/bin/sh
# Runs every test of the test binaries `cargo test <args>` builds, each test in a process of its
# own under `timeout 60`, as many at once as the machine has CPUs, and prints the ten slowest.
# Before the first test it checks that no test binary carries debug info.
# Usage: sh tools/run-tests.sh <label> <cargo test args>, e.g.
#   sh tools/run-tests.sh engine/search --manifest-path engine/search/Cargo.toml --locked
# Exit 0 when every test that is not ignored ran and passed; 1 when a test failed, was killed,
# ran no test, when no test ran at all, or when a test binary carries debug info.
#
# Why a process per test: the organization's rules ask the runner to kill a test after 60 s and
# the step to print its ten slowest tests. libtest on stable Rust does neither: its time per test
# (`--report-time`) and its JSON output need `-Z unstable-options`, and it only warns about a test
# that runs over 60 s. A process per test is the one way on stable Rust to time each test and to
# end exactly the one that hangs. A test that reads a fixture shared through a `OnceLock` builds
# it once per process here; the ten slowest show what that costs.
#
# Every test runs in the directory of its package's manifest, as `cargo test` runs it.
# UXUI_TEST_LIMIT (seconds, default 60) and UXUI_TEST_JOBS (default: the CPUs) exist for
# tools/run-tests.test.sh and for measuring; tools/gate.sh unsets both.
#
# sh tools/run-tests.sh --check-debug <file>...: exit 1 naming each file with a .debug_* section.
set -u

limit=${UXUI_TEST_LIMIT:-60}

if [ "${1:-}" = --one ]; then
	# --one <results> <logs> <label> <dir> <exe> <name>: one test, one result line.
	results=$2 logs=$3 label=$4 dir=$5 exe=$6 name=$7
	log=$logs/$(basename "$exe")-$(printf '%s' "$name" | tr -c 'A-Za-z0-9_' '_').log
	start=$(date +%s%N)
	(cd "$dir" && timeout -k 5 "$limit" "$exe" --exact "$name" --test-threads=1 >"$log" 2>&1)
	rc=$?
	end=$(date +%s%N)
	if [ "$rc" -eq 0 ] && ! grep -q '^test result: ok\. 1 passed' "$log"; then
		rc=3
	fi
	printf '%s\t%s\t%s\t%s\t%s\n' "$(((end - start) / 1000000))" "$rc" "$label" "$name" "$log" >>"$results"
	exit 0
fi

if [ "${1:-}" = --check-debug ]; then
	shift
	command -v readelf >/dev/null 2>&1 || {
		printf 'run-tests: no readelf on this machine to check the test binaries for debug info\n'
		exit 1
	}
	bad=0
	for file in "$@"; do
		sections=$(readelf -S -W "$file" 2>&1) || {
			printf 'run-tests: readelf cannot read %s\n' "$file"
			bad=1
			continue
		}
		debug=$(printf '%s\n' "$sections" | grep -o '\.debug_[a-z_]*' | sort -u | tr '\n' ' ')
		if [ -n "$debug" ]; then
			printf 'run-tests: %s carries debug info: %s\n' "$file" "$debug"
			bad=1
		fi
	done
	exit "$bad"
fi

[ "$#" -ge 1 ] || {
	printf 'usage: sh tools/run-tests.sh <label> <cargo test args>\n' >&2
	exit 2
}
label=$1
shift

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root" || exit 1
self=$root/tools/run-tests.sh
work=$root/target/run-tests/$(printf '%s' "$label" | tr -c 'A-Za-z0-9_-' '_')
rm -rf "$work"
mkdir -p "$work/logs"
tab=$(printf '\t')

# The test binaries and their manifests, from cargo's own record of what it built.
if ! cargo test "$@" --no-run --message-format=json >"$work/build.json" 2>"$work/build.err"; then
	cat "$work/build.err"
	printf 'run-tests: %s: the test binaries did not build\n' "$label"
	exit 1
fi
grep '"test":true}' "$work/build.json" |
	sed -n 's/.*"manifest_path":"\([^"]*\)".*"executable":"\([^"]*\)".*/\1\t\2/p' >"$work/binaries"
if [ ! -s "$work/binaries" ]; then
	printf 'run-tests: %s: cargo built no test binary\n' "$label"
	exit 1
fi
# The paths have no spaces: cargo's target directories under the root.
# shellcheck disable=SC2046
sh "$self" --check-debug $(cut -f 2 "$work/binaries") || {
	printf 'run-tests: %s: a test binary carries debug info; the repository builds them with debug = false (.cargo/config.toml)\n' "$label"
	exit 1
}

: >"$work/tests"
ignored=0
while IFS="$tab" read -r manifest exe; do
	dir=$(dirname "$manifest")
	(cd "$dir" && "$exe" --list --format terse) | sed -n 's/: test$//p' | LC_ALL=C sort >"$work/all"
	(cd "$dir" && "$exe" --list --format terse --ignored) | sed -n 's/: test$//p' | LC_ALL=C sort >"$work/ignored"
	ignored=$((ignored + $(wc -l <"$work/ignored")))
	LC_ALL=C comm -23 "$work/all" "$work/ignored" | while IFS= read -r name; do
		printf '%s\n%s\n%s\n' "$dir" "$exe" "$name"
	done >>"$work/tests"
done <"$work/binaries"

count=$(($(wc -l <"$work/tests") / 3))
if [ "$count" -eq 0 ]; then
	printf 'run-tests: %s: no test to run\n' "$label"
	exit 1
fi

jobs=${UXUI_TEST_JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)}
: >"$work/results"
start=$(date +%s%N)
tr '\n' '\0' <"$work/tests" |
	UXUI_TEST_LIMIT=$limit xargs -0 -n 3 -P "$jobs" sh "$self" --one "$work/results" "$work/logs" "$label"
end=$(date +%s%N)
wall=$(((end - start) / 1000000))

ran=$(wc -l <"$work/results")
passed=$(awk -F '\t' '$2 == 0' "$work/results" | wc -l)
killed=$(awk -F '\t' '$2 == 124 || $2 == 137' "$work/results" | wc -l)
failed=$((ran - passed - killed))

printf 'run-tests: %s: the ten slowest of %s tests\n' "$label" "$ran"
sort -t "$tab" -k1,1nr "$work/results" | head -10 |
	awk -F '\t' '{ printf "  %7.2f s  %s\n", $1 / 1000, $4 }'

awk -F '\t' '$2 != 0' "$work/results" | while IFS="$tab" read -r _ rc _ name log; do
	case $rc in
	124 | 137) printf 'KILLED after %s s: %s\n' "$limit" "$name" ;;
	3) printf 'FAILED %s: it ran no test (%s)\n' "$name" "$log" ;;
	*) printf 'FAILED %s (exit %s)\n' "$name" "$rc" ;;
	esac
	tail -30 "$log"
done

printf 'run-tests: %s: %s passed, %s failed, %s killed after %s s, %s ignored, of %s tests in %s binaries, %s.%03d s with %s at once\n' \
	"$label" "$passed" "$failed" "$killed" "$limit" "$ignored" "$count" "$(wc -l <"$work/binaries")" \
	"$((wall / 1000))" "$((wall % 1000))" "$jobs"
[ "$ran" -eq "$count" ] && [ "$passed" -eq "$count" ]
