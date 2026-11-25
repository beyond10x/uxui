#!/bin/sh
# Runs the workspace's test binaries <count> more times after the gate's first run.
# Usage: sh tools/repeat-tests.sh <count>. Exit 0 when every run passes, 1 on the first failure.
#
# Tests run in parallel threads. Two that share a scratch directory, or a file one writes while
# the other forks, fail only on some runs: #38's forge Gate (run 45568883701) and 0.9.0's release
# gate (run 45586279102) failed that way after a green local gate. Repeating the runs shows such a
# race here more often. Doc tests are left out: they compile on every run.
set -u

count=${1:-4}
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root" || exit 1

n=0
while [ "$n" -lt "$count" ]; do
	n=$((n + 1))
	if ! out=$(cargo test --workspace --locked --tests -q 2>&1); then
		printf '%s\n' "$out" | tail -40
		printf 'repeat-tests: the tests failed on repeat %s of %s\n' "$n" "$count"
		exit 1
	fi
done
printf 'repeat-tests: %s repeats passed\n' "$count"
