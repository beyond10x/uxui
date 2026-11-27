#!/bin/sh
# Proves the pieces of the forge gate:
# - tools/prose-only.sh says "prose only" for a change of planning and prose files alone and
#   "run the gate" for every other change, and no prose file it lets skip is read by a tool;
# - tools/check-workflows.sh passes on this tree and fails on each broken copy of its workflows;
# - tools/gate.sh runs every first run of the tests through tools/run-tests.sh.
# Copies go under target/gate-test/ in the repository. Exit 0 when every case holds.
set -u

root=$(cd "$(dirname "$0")/.." && pwd)
work=$root/target/gate-test
classify=$root/tools/prose-only.sh
fails=0
rm -rf "$work"
mkdir -p "$work"

ok() { printf 'ok   %s\n' "$1"; }
bad() {
	printf 'FAIL %s\n' "$1"
	fails=$((fails + 1))
}

# paths <prose|gate> <label> <paths, one per line>
paths() {
	want=$1
	label=$2
	printf '%s' "$3" | sh "$classify" >"$work/out" 2>&1
	got=$?
	if [ "$want" = prose ] && [ "$got" -eq 0 ]; then
		ok "prose only: $label"
	elif [ "$want" = gate ] && [ "$got" -eq 1 ]; then
		ok "runs the gate: $label ($(tail -1 "$work/out"))"
	else
		bad "$label: expected $want, exit $got"
		cat "$work/out"
	fi
}

paths prose "a planning file" '.engineering/planning/story/forge-gate-parity.md
'
paths prose "planning files that are not Markdown" '.engineering/planning/epic/engine-slices.md
.engineering/aep.yaml
'
paths prose "the agents' and the readers' files" 'AGENTS.md
CLAUDE.md
README.md
'
paths prose "research, reviews and a design document" 'research/README.md
reviews/2026-11-24-uxsearch-spec-review.md
docs/concept.md
'
paths prose "a last line without a newline" '.engineering/planning/story/a.md
docs/ratchet.md'

paths gate "no path at all" ''
paths gate "a tool" 'tools/gate.sh
'
paths gate "a planning file and a source file" '.engineering/planning/story/a.md
engine/browser/src/chrome.rs
'
paths gate "a page of the built documentation site" 'website/docs/index.md
'
paths gate "the documentation site's own README" 'website/README.md
'
paths gate "the skill" 'plugin/skills/ui-iteration-loop/SKILL.md
'
paths gate "a reference of the skill" 'plugin/skills/ui-iteration-loop/references/heuristics.md
'
paths gate "the changelog, which tools/check-plugin.sh reads" 'CHANGELOG.md
'
paths gate "docs/metrics.md, a source of a generated page" 'docs/metrics.md
'
paths gate "docs/brief.md, a source of a generated page" 'docs/brief.md
'
paths gate "docs/roadmap.md, a source of a generated page" 'docs/roadmap.md
'
paths gate "the studio campaign's document, which tools/studio-campaign.sh writes and a test reads" 'docs/campaign-2026-11-26-studio-engine.md
'
paths gate "a Markdown test fixture" 'engine/browser/tests/fixtures/values/findings/UXF-005.md
'
paths gate "an image beside the design documents" 'docs/campaign-2026-11-24/chosen-dark-390.png
'
paths gate "a research file that is not Markdown" 'research/notes.txt
'
paths gate "the specification's gaps" 'ess/GAPS.md
'
paths gate "a workflow" '.github/workflows/ci.yml
'

# unread <tools directory>: prints each Markdown file of the tree that the classifier lets skip and
# a tool of the directory names outside its comments (a file a check reads is not prose), and
# last the number of prose Markdown files it checked.
unread() {
	checked=0
	for md in $(cd "$root" && find . -name '*.md' -not -path './target/*' -not -path './.git/*' \
		-not -path '*/node_modules/*' -not -path './.engineering/*' | sed 's|^\./||' | LC_ALL=C sort); do
		printf '%s\n' "$md" | sh "$classify" >/dev/null 2>&1 || continue
		checked=$((checked + 1))
		pattern=$(printf '%s' "$md" | sed 's/[.]/[.]/g')
		for tool in "$1"/*.sh; do
			case $tool in */prose-only.sh | *.test.sh) continue ;; esac
			if grep -v '^[[:space:]]*#' "$tool" | grep -qE "(^|[^A-Za-z0-9_./-])$pattern([^A-Za-z0-9_-]|\$)"; then
				printf '%s is classified prose and %s names it\n' "$md" "$(basename "$tool")"
			fi
		done
	done
	printf '%s\n' "$checked"
}

found=$(unread "$root/tools")
checked=$(printf '%s\n' "$found" | tail -1)
named=$(printf '%s\n' "$found" | sed '$d')
if [ -n "$named" ]; then
	bad "$named"
elif [ "$checked" -gt 0 ]; then
	ok "none of the $checked prose Markdown files of the tree is named by a tool"
else
	bad "no Markdown file of the tree is classified prose; the case above checked nothing"
fi

mkdir -p "$work/tools"
cp "$root"/tools/*.sh "$work/tools/"
printf '#!/bin/sh\n# docs/ratchet.md in a comment is no read.\nawk 1 docs/concept.md\n' >"$work/tools/reads-prose.sh"
named=$(unread "$work/tools" | sed '$d')
if [ "$named" = "docs/concept.md is classified prose and reads-prose.sh names it" ]; then
	ok "a tool that reads a prose file is found ($named)"
else
	bad "a tool that reads docs/concept.md and names docs/ratchet.md in a comment: found '$named'"
fi

# workflows <pass|fail> <label> <mutation>: the mutation runs in a fresh copy of the workflows,
# the toolchain file and tools/gate.sh ($w) before tools/check-workflows.sh reads it.
workflows() {
	want=$1
	label=$2
	w=$work/workflows
	rm -rf "$w"
	mkdir -p "$w/.github" "$w/tools"
	cp -r "$root/.github/workflows" "$w/.github/"
	cp "$root/rust-toolchain.toml" "$w/"
	cp "$root/tools/gate.sh" "$w/tools/"
	(cd "$w" && eval "$3")
	sh "$root/tools/check-workflows.sh" "$w" >"$work/out" 2>&1
	got=$?
	if [ "$want" = pass ] && [ "$got" -eq 0 ]; then
		ok "workflows: $label"
	elif [ "$want" = fail ] && [ "$got" -eq 1 ]; then
		ok "workflows: $label ($(grep -m 1 '^FAIL' "$work/out"))"
	else
		bad "workflows: $label: expected $want, exit $got"
		cat "$work/out"
	fi
}

# cache_first <workflow>: moves the step of Swatinem/rust-cache in front of the toolchain's step.
cache_first() {
	awk '/^      - uses: Swatinem\/rust-cache@/ { on = 1 } on && /^      - / && !/rust-cache/ { on = 0 } on' "$1" >cache.step
	awk '/^      - uses: Swatinem\/rust-cache@/ { on = 1 } on && /^      - / && !/rust-cache/ { on = 0 }
		!on && /^      - uses: dtolnay\/rust-toolchain@/ { while ((getline l < "cache.step") > 0) print l }
		!on' "$1" >cache.yml && mv cache.yml "$1" && rm cache.step
}

workflows pass "this tree" ':'
workflows fail "no pull-request workflow" 'rm .github/workflows/ci.yml'
workflows fail "the release gate runs cargo test itself" \
	'sed "s|^\(  *\)run: sh tools/gate.sh\$|\1run: cargo test --workspace|" .github/workflows/release.yml >x && mv x .github/workflows/release.yml'
workflows fail "the pull-request gate runs on another image" \
	'sed "s|runs-on: ubuntu-latest|runs-on: ubuntu-22.04|" .github/workflows/ci.yml >x && mv x .github/workflows/ci.yml'
workflows fail "a workflow names its own toolchain" \
	'sed "s|toolchain: \${{ steps.toolchain.outputs.channel }}|toolchain: 1.99.1|" .github/workflows/release.yml >x && mv x .github/workflows/release.yml'
workflows fail "an action pinned by a tag" \
	'sed "s|actions/checkout@[0-9a-f]*|actions/checkout@v4|" .github/workflows/ci.yml >x && mv x .github/workflows/ci.yml'
workflows fail "the pull-request gate does not ask which files changed" \
	'sed "s|sh tools/prose-only.sh|cat|" .github/workflows/ci.yml >x && mv x .github/workflows/ci.yml'
workflows fail "the pull-request workflow does not run on a push to main" \
	'sed "/^  push:\$/,/^    branches: \[main\]\$/d" .github/workflows/ci.yml >x && mv x .github/workflows/ci.yml'
workflows fail "the release gate installs no clippy and rustfmt" \
	'sed "/^ *components: clippy, rustfmt\$/d" .github/workflows/release.yml >x && mv x .github/workflows/release.yml'
workflows fail "the pull-request gate keeps no dependency cache" \
	'awk "/uses: Swatinem\/rust-cache@/ { skip = 1; next } skip && /^      - / { skip = 0 } !skip" .github/workflows/ci.yml >x && mv x .github/workflows/ci.yml'
workflows fail "the release gate caches one workspace less" \
	'sed "/^ *engine\/search -> target\$/d" .github/workflows/release.yml >x && mv x .github/workflows/release.yml'
workflows fail "the search cache names its target from the repository root" \
	'sed "s|engine/search -> target\$|engine/search -> engine/search/target|" .github/workflows/ci.yml >x && mv x .github/workflows/ci.yml'
workflows fail "the two gates share no cache key" \
	'sed "s|shared-key: gate|shared-key: release|" .github/workflows/release.yml >x && mv x .github/workflows/release.yml'
workflows fail "the cache step runs before the toolchain is installed" 'cache_first .github/workflows/ci.yml'
workflows fail "a workflow empties RUSTC_WRAPPER" \
	'sed "s|^\(  *\)run: sh tools/gate.sh\$|\1env:\n\1  RUSTC_WRAPPER: \"\"\n\1run: sh tools/gate.sh|" .github/workflows/ci.yml >x && mv x .github/workflows/ci.yml'
workflows fail "the gate empties CARGO_BUILD_RUSTC_WRAPPER" \
	'printf "CARGO_BUILD_RUSTC_WRAPPER= cargo build\n" >>tools/gate.sh'
workflows fail "a cargo config empties rustc-wrapper" \
	'mkdir -p .cargo && printf "[build]\nrustc-wrapper = \"\"\n" >.cargo/config.toml'
workflows pass "a workflow and the gate that name a wrapper" \
	'sed "s|^\(  *\)run: sh tools/gate.sh\$|\1env:\n\1  RUSTC_WRAPPER: sccache\n\1run: sh tools/gate.sh|" .github/workflows/ci.yml >x && mv x .github/workflows/ci.yml && printf "RUSTC_WRAPPER=sccache cargo build\n" >>tools/gate.sh'

# The gate holds the workspace's format and lints, and engine/search's (a workspace of its own)
# format, lints and tests: tools/gate.sh runs each command, and the format check fails on a copy
# of the sources with one planted format diff (rustfmt compiles nothing).
for want in 'cargo fmt --all --check' 'cargo clippy --workspace --all-targets --locked -- -D warnings' \
	'cargo fmt --manifest-path engine/search/Cargo.toml --check' \
	'cargo clippy --manifest-path engine/search/Cargo.toml --all-targets --locked -- -D warnings' \
	'timeout 120 sh tools/run-tests.sh workspace --workspace --locked' \
	'timeout 120 sh tools/run-tests.sh engine/search --manifest-path engine/search/Cargo.toml --locked'; do
	if grep -qxF "step $want" "$root/tools/gate.sh"; then
		ok "the gate runs $want"
	else
		bad "tools/gate.sh does not run: step $want"
	fi
done
# Every first run of the tests goes through tools/run-tests.sh, which kills a test after 60 s and
# prints the ten slowest; a plain `cargo test` step does neither.
plain=$(grep -E '^step .*cargo test' "$root/tools/gate.sh" | grep -v -- '--no-run')
if [ -z "$plain" ]; then
	ok "the gate runs no test with a plain cargo test"
else
	bad "tools/gate.sh runs tests outside tools/run-tests.sh: $plain"
fi
if grep -qE '^for test in .*[ ]run-tests( |;)' "$root/tools/gate.sh"; then
	ok "the gate runs tools/run-tests.test.sh"
else
	bad "tools/gate.sh does not run tools/run-tests.test.sh"
fi
if grep -qE '^unset .*UXUI_TEST_LIMIT' "$root/tools/gate.sh" && grep -qE '^unset .*UXUI_TEST_JOBS' "$root/tools/gate.sh"; then
	ok "the gate unsets the limit and the job count of tools/run-tests.sh"
else
	bad "tools/gate.sh does not unset UXUI_TEST_LIMIT and UXUI_TEST_JOBS"
fi
l=$work/lint
rm -rf "$l"
mkdir -p "$l"
cp "$root/Cargo.toml" "$root/Cargo.lock" "$root/rust-toolchain.toml" "$l/"
for d in engine generators; do
	(cd "$root" && find "$d" -path '*/target' -prune -o \( -name '*.rs' -o -name Cargo.toml \) -print) |
		while IFS= read -r p; do
			mkdir -p "$l/$(dirname "$p")"
			cp "$root/$p" "$l/$p"
		done
done
if (cd "$l" && cargo fmt --all --check >"$work/out" 2>&1); then
	ok "format: the workspace copy is formatted"
else
	bad "format: the workspace copy is not formatted"
	tail -5 "$work/out"
fi
printf 'pub fn planted ( )   {}\n' >>"$l/generators/own/src/lib.rs"
(cd "$l" && cargo fmt --all --check >"$work/out" 2>&1)
if [ $? -eq 1 ]; then
	ok "format: a planted format diff fails the check"
else
	bad "format: a planted format diff passed the check"
	tail -5 "$work/out"
fi

rm -rf "$work"
[ "$fails" -eq 0 ]
