#!/bin/sh
# Checks that the pull-request gate and the release gate are the same: both workflows have a job
# `gate` that runs `sh tools/gate.sh` on the same runner image, with clippy and rustfmt installed,
# and nothing of the gate inline, the
# pull-request workflow runs on every pull request and on a push to main and asks
# tools/prose-only.sh which files changed, every Rust toolchain a workflow installs is the one of
# rust-toolchain.toml, and every action is pinned by a full commit id.
# Usage: sh tools/check-workflows.sh [repo root]. Exit 0 when every check passes, 1 otherwise.
set -u

root=${1:-$(cd "$(dirname "$0")/.." && pwd)}
cd "$root" || exit 1

fails=0
fail() {
	printf 'FAIL %s\n' "$*"
	fails=$((fails + 1))
}
# each <message> <lines>: one failure for each non-empty line, the line after the message.
each() {
	[ -n "$2" ] || return 0
	while IFS= read -r line; do
		fail "$1: $line"
	done <<EOF
$2
EOF
}

ci=.github/workflows/ci.yml
release=.github/workflows/release.yml

# The lines of job $2 of workflow $1: from `  <job>:` to the next job or the end.
job() {
	awk -v job="$2" '
	/^jobs:/ { jobs = 1; next }
	jobs && $0 == "  " job ":" { on = 1; next }
	jobs && on && /^  [^ #][^ ]*:[ ]*$/ { exit }
	on { print }' "$1"
}

for f in "$ci" "$release"; do
	[ -f "$f" ] || { fail "$f: missing"; continue; }
	gate=$(job "$f" gate)
	[ -n "$gate" ] || { fail "$f: no job gate"; continue; }
	printf '%s\n' "$gate" | grep -qE '^ +run: sh tools/gate\.sh$' || fail "$f: job gate does not run sh tools/gate.sh"
	printf '%s\n' "$gate" | grep -qE '^ +components: clippy, rustfmt$' ||
		fail "$f: job gate installs no clippy and rustfmt, which tools/gate.sh runs"
	each "$f: job gate runs a check of tools/gate.sh itself" \
		"$(printf '%s\n' "$gate" | grep -E 'cargo |check-plugin\.sh|docs\.sh' | sed 's/^ *//')"
done

if [ -f "$ci" ] && [ -f "$release" ]; then
	a=$(job "$ci" gate | sed -n 's/^    runs-on: *//p')
	b=$(job "$release" gate | sed -n 's/^    runs-on: *//p')
	[ -n "$a" ] && [ "$a" = "$b" ] || fail "$ci job gate runs on '$a', $release job gate on '$b'"
fi

if [ -f "$ci" ]; then
	grep -qx 'name: Gate' "$ci" || fail "$ci: the workflow is not named Gate, the name tools/release-ready.sh asks for"
	grep -qx '  pull_request:' "$ci" || fail "$ci: does not run on pull_request"
	awk '$0 == "  push:" { getline; if ($0 == "    branches: [main]") found = 1 } END { exit !found }' "$ci" ||
		fail "$ci: does not run on a push to main"
	job "$ci" gate | grep -q 'sh tools/prose-only\.sh' || fail "$ci: job gate does not ask sh tools/prose-only.sh which files changed"
fi

channel=$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml 2>/dev/null)
[ -n "$channel" ] || fail "rust-toolchain.toml: no channel"

for f in .github/workflows/*.yml; do
	[ -f "$f" ] || continue
	each "$f: a toolchain that is not read from rust-toolchain.toml" \
		"$(grep -nE '^ +toolchain:' "$f" | grep -vE '^[0-9]+: +toolchain: \$\{\{ steps\.toolchain\.outputs\.channel \}\}$')"
	if grep -q 'dtolnay/rust-toolchain' "$f"; then
		grep -q 'rust-toolchain\.toml' "$f" || fail "$f: installs Rust and does not read rust-toolchain.toml"
	fi
	each "$f: an action not pinned by a full commit id" \
		"$(grep -nE '^[ -]+uses: ' "$f" | grep -vE 'uses: [^ @]+@[0-9a-f]{40}( |$)')"
done

if [ "$fails" -gt 0 ]; then
	printf 'check-workflows: %d failure(s)\n' "$fails"
	exit 1
fi
printf 'pass workflows: the pull-request gate and the release gate run sh tools/gate.sh on %s with Rust %s\n' "$a" "$channel"
