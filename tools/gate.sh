#!/bin/sh
# The checks that need no local login: the gate the pull-request workflow (`.github/workflows/ci.yml`),
# the release workflow's Gate job and `task check:pool` run, all three through this one script.
# Usage: sh tools/gate.sh. Exit 0 when every step passes, 1 otherwise; every step runs.
#
# What the forge's runner does differently, made the same here:
# - it has no browser: the Playwright cache is an empty directory and UXUI_CHROME is unset, so a
#   test that looks for Chrome fails here as it fails there (0.8.0's release gate, run 45545202319);
# - it builds with the lock file as committed: cargo runs with --locked;
# - it builds without debug info and without incremental compilation: .cargo/config.toml says
#   so for every build in the repository, the same as the host's one value and the forge's
#   CARGO_INCREMENTAL=0.
# Budgets: a test script is killed after 60 s; the tests run after their build, each test in a
# process of its own that is killed after 60 s (tools/run-tests.sh, which prints the ten slowest
# and fails on a test binary with debug info), and each lane is killed after 120 s.
set -u

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root" || exit 1

no_browser=$root/target/gate/no-browser
mkdir -p "$no_browser"
PLAYWRIGHT_BROWSERS_PATH=$no_browser
export PLAYWRIGHT_BROWSERS_PATH
unset UXUI_CHROME
unset UXUI_TEST_LIMIT UXUI_TEST_JOBS

failed=
step() {
	printf '== %s\n' "$*"
	"$@" || failed="$failed
  $*"
}

step sh tools/check-plugin.sh
step sh tools/check-workflows.sh
step sh tools/check-themes.sh
step sh tools/docs.sh check
for test in check-plugin check-themes release docs gate release-ready run-tests; do
	step timeout 60 sh "tools/$test.test.sh"
done
step cargo fmt --all --check
step cargo clippy --workspace --all-targets --locked -- -D warnings
step cargo test --workspace --locked --no-run
step timeout 120 sh tools/run-tests.sh workspace --workspace --locked
step timeout 120 sh tools/repeat-tests.sh 4
# engine/search is a workspace of its own (its ESS crates come by git, AGENTS.md § Ownership), so
# the workspace steps above do not reach it.
step cargo fmt --manifest-path engine/search/Cargo.toml --check
step cargo clippy --manifest-path engine/search/Cargo.toml --all-targets --locked -- -D warnings
step cargo test --manifest-path engine/search/Cargo.toml --locked --no-run
step timeout 120 sh tools/run-tests.sh engine/search --manifest-path engine/search/Cargo.toml --locked

if [ -n "$failed" ]; then
	printf 'gate: failed:%s\n' "$failed"
	exit 1
fi
printf 'gate: every step passed\n'
