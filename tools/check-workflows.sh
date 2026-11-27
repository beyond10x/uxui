#!/bin/sh
# Checks that the pull-request gate and the release gate are the same: both workflows have a job
# `gate` that runs `sh tools/gate.sh` on the same runner image, with clippy and rustfmt installed,
# and nothing of the gate inline, the
# pull-request workflow runs on every pull request and on a push to main and asks
# tools/prose-only.sh which files changed, every Rust toolchain a workflow installs is the one of
# rust-toolchain.toml, and every action is pinned by a full commit id. Both gate jobs keep the
# same dependency cache (Swatinem/rust-cache, after the toolchain, both workspaces, one shared
# key), and nothing a workflow or a tool runs empties the compiler wrapper.
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

# The step of job $2 of workflow $1 that uses Swatinem/rust-cache, without its `if:` line.
cache_step() {
	job "$1" "$2" | awk '/^      - uses: Swatinem\/rust-cache@/ { on = 1 } on && /^      - / && !/rust-cache/ { on = 0 } on' |
		grep -v '^        if: '
}

for f in "$ci" "$release"; do
	[ -f "$f" ] || continue
	step=$(cache_step "$f" gate)
	if [ -z "$step" ]; then
		fail "$f: job gate keeps no dependency cache (Swatinem/rust-cache): it builds engine/search's crates from nothing"
		continue
	fi
	for want in '. -> target' 'engine/search -> target' 'shared-key: gate'; do
		printf '%s\n' "$step" | grep -qxF "          $want" || printf '%s\n' "$step" | grep -qxF "            $want" ||
			fail "$f: the cache step of job gate does not name '$want'"
	done
	toolchain=$(job "$f" gate | grep -n 'uses: dtolnay/rust-toolchain@' | head -1 | cut -d: -f1)
	cache=$(job "$f" gate | grep -n 'uses: Swatinem/rust-cache@' | head -1 | cut -d: -f1)
	[ -n "$toolchain" ] && [ "$cache" -gt "$toolchain" ] ||
		fail "$f: the cache step of job gate runs before the toolchain is installed; its key holds the rustc version"
done

if [ -f "$ci" ] && [ -f "$release" ]; then
	a=$(job "$ci" gate | sed -n 's/^    runs-on: *//p')
	b=$(job "$release" gate | sed -n 's/^    runs-on: *//p')
	[ -n "$a" ] && [ "$a" = "$b" ] || fail "$ci job gate runs on '$a', $release job gate on '$b'"
	[ "$(cache_step "$ci" gate)" = "$(cache_step "$release" gate)" ] ||
		fail "the cache steps of the job gate of $ci and $release differ"
fi

# The organization's one compile cache: nothing the gate or a workflow runs empties the compiler
# wrapper, so a local gate keeps the host's cache (the forge has none to keep).
empty="(\"\"|'')"
wrapper='(RUSTC_WRAPPER|CARGO_BUILD_RUSTC_WRAPPER)'
for f in .github/workflows/*.yml tools/*.sh Taskfile.yml .cargo/config.toml; do
	[ -f "$f" ] || continue
	case $f in tools/check-workflows.sh | tools/*.test.sh) continue ;; esac
	each "$f: empties the compiler wrapper" "$(grep -nE \
		-e "$wrapper=$empty?([[:space:]]|;|\$)" \
		-e "$wrapper:[[:space:]]*$empty?[[:space:]]*\$" \
		-e "rustc-wrapper[[:space:]]*=[[:space:]]*$empty[[:space:]]*\$" "$f")"
done

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
