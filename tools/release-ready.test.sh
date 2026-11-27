#!/bin/sh
# Proves that tools/release-ready.sh exits 0 only when every required workflow (Gate,
# Documentation validation, Documentation site) has a run on the commit that ended success and every
# other such run ended success or skipped, and refuses, naming the run, when one is absent, pending
# or red (a workflow whose only runs are skipped is red). `gh` is a stub on PATH
# that prints a recorded run list; no case reaches the network. Exit 0 when every case holds.
set -u

root=$(cd "$(dirname "$0")/.." && pwd)
work=$root/target/release-ready-test
fails=0
rm -rf "$work"
mkdir -p "$work/bin"

# The stub: records its arguments, prints $work/runs.json, exits $work/exit (0 without it).
cat >"$work/bin/gh" <<'STUB'
#!/bin/sh
printf '%s\n' "$@" >"$STUB_DIR/args"
cat "$STUB_DIR/runs.json"
[ -f "$STUB_DIR/exit" ] && exit "$(cat "$STUB_DIR/exit")"
exit 0
STUB
chmod +x "$work/bin/gh"

sha=5595daf8e0f048695725a64ff47143bd89f35739
other=5f74a4a0000000000000000000000000000000aa

# run <workflow> <status> <conclusion> [head]: one entry as `gh run list --json` writes it.
run() {
	printf '{"conclusion":"%s","headSha":"%s","status":"%s","workflowName":"%s"}' "$3" "${4:-$sha}" "$2" "$1"
}
# list <entry>...: the entries as one JSON array.
list() {
	out='['
	sep=
	for entry in "$@"; do
		out=$out$sep$entry
		sep=,
	done
	printf '%s]\n' "$out" >"$work/runs.json"
	rm -f "$work/exit"
}

# expect <pass|fail|usage> <label> [text the output must hold] [text the output must not hold]
expect() {
	want=$1
	label=$2
	text=${3:-}
	absent=${4:-}
	STUB_DIR=$work PATH="$work/bin:$PATH" sh "$root/tools/release-ready.sh" ${commit-$sha} >"$work/out" 2>&1
	got=$?
	case $want in pass) code=0 ;; fail) code=1 ;; usage) code=2 ;; esac
	if [ "$got" -ne "$code" ]; then
		bad "$label: expected $want (exit $code), exit $got"
		cat "$work/out"
	elif [ -n "$text" ] && ! grep -qF -- "$text" "$work/out"; then
		bad "$label: the output does not name '$text'"
		cat "$work/out"
	elif [ -n "$absent" ] && grep -qF -- "$absent" "$work/out"; then
		bad "$label: the output names '$absent'"
		cat "$work/out"
	else
		printf 'ok   %s (%s)\n' "$label" "$(tail -1 "$work/out")"
	fi
}
bad() {
	printf 'FAIL %s\n' "$1"
	fails=$((fails + 1))
}

green_gate=$(run Gate completed success)
green_validation=$(run 'Documentation validation' completed success)
green_site=$(run 'Documentation site' completed success)

# Recorded from `gh run list --repo beyond10x/uxui --commit 5595daf…` on 2026-11-25: the 0.8.0
# commit, which had no Gate run and whose release gate failed after the tag.
cat >"$work/recorded.json" <<'JSON'
[{"conclusion":"failure","headSha":"5595daf8e0f048695725a64ff47143bd89f35739","status":"completed","workflowName":"Release"},{"conclusion":"success","headSha":"5595daf8e0f048695725a64ff47143bd89f35739","status":"completed","workflowName":"Documentation site"},{"conclusion":"success","headSha":"5595daf8e0f048695725a64ff47143bd89f35739","status":"completed","workflowName":"Documentation validation"}]
JSON

list "$green_gate" "$green_validation" "$green_site"
expect pass "green: the three required runs ended success" "Gate: success"
if [ "$(tr '\n' ' ' <"$work/args")" = "run list --repo beyond10x/uxui --commit $sha --limit 100 --json conclusion,headSha,status,workflowName " ]; then
	printf 'ok   the runs are asked of beyond10x/uxui for exactly that commit\n'
else
	bad "gh was asked: $(tr '\n' ' ' <"$work/args")"
fi

list "$green_gate" "$green_validation" "$green_site" "$(run Release completed failure)" \
	"$(run Gate completed failure "$other")"
expect pass "green: a red run of another workflow, and one on another commit, do not count" "not required: Release: failure"

cp "$work/recorded.json" "$work/runs.json"
rm -f "$work/exit"
expect fail "absent: the recorded 0.8.0 commit has no Gate run" "Gate: absent"

list "$green_gate" "$green_validation"
expect fail "absent: Documentation site" "Documentation site: absent"

list "$green_validation" "$green_site" "$(run Gate completed success "$other")"
expect fail "absent: a green Gate on another commit is not one on this" "Gate: absent"

list "$(run Gate in_progress '')" "$green_validation" "$green_site"
expect fail "pending: Gate in progress" "Gate: pending (in_progress)"

list "$green_gate" "$(run 'Documentation validation' queued '')" "$green_site"
expect fail "pending: Documentation validation queued" "Documentation validation: pending (queued)"

list "$green_gate" "$green_validation" "$(run 'Documentation site' completed failure)"
expect fail "red: Documentation site failed" "Documentation site: red (failure)"

list "$(run Gate completed cancelled)" "$green_validation" "$green_site"
expect fail "red: Gate cancelled" "Gate: red (cancelled)"

list "$green_gate" "$(run Gate completed failure)" "$green_validation" "$green_site"
expect fail "red: one of two Gate runs on the commit failed" "Gate: red (failure)"

# One concurrency group: a run of Documentation site on the commit can end skipped beside one that
# ended success (the 7268398 case: one run skipped, re-run skipped again, one run success).
skipped_site=$(run 'Documentation site' completed skipped)
list "$green_gate" "$green_validation" "$skipped_site" "$green_site" "$skipped_site"
expect pass "skipped beside a success: Documentation site may be tagged" \
	"not counted: Documentation site: skipped beside a success"

list "$green_gate" "$green_validation" "$skipped_site"
expect fail "only skipped: Documentation site is red" "Documentation site: red (skipped)"

list "$green_gate" "$green_validation" "$skipped_site" "$skipped_site"
expect fail "only skipped twice: Documentation site is red" "Documentation site: red (skipped)"

list "$green_gate" "$green_validation" "$skipped_site" "$(run 'Documentation site' in_progress '')"
expect fail "skipped beside pending: Documentation site is pending, not red" \
	"Documentation site: pending (in_progress)" "red (skipped)"

list "$green_gate" "$green_validation" "$(run 'Documentation site' completed failure)" "$green_site"
expect fail "failure beside a success: Documentation site is red" "Documentation site: red (failure)"

list "$green_gate" "$green_validation" "$skipped_site" "$green_site" \
	"$(run 'Documentation site' completed cancelled)"
expect fail "cancelled beside a success and a skipped run: Documentation site is red" \
	"Documentation site: red (cancelled)" "red (skipped)"

list "$green_gate" "$green_validation" "$green_site" "$(run 'Documentation site' completed skipped "$other")"
expect pass "a skipped run on another commit does not count" \
	"not counted: Documentation site: skipped on $other, another commit"

list "$(run Gate completed failure)" "$(run 'Documentation validation' in_progress '')"
expect fail "every blocking run is named: Gate red" "Gate: red (failure)"
expect fail "every blocking run is named: Documentation validation pending" "Documentation validation: pending (in_progress)"
expect fail "every blocking run is named: Documentation site absent" "Documentation site: absent"

list
expect fail "no run on the commit at all" "Gate: absent"

printf 'not json\n' >"$work/runs.json"
expect fail "an answer that is not JSON" "not read"

printf '[{"headSha":"%s","status":"completed","workflowName":"Gate"}]\n' "$sha" >"$work/runs.json"
expect fail "a run without a conclusion" "not read"

list "$green_gate" "$green_validation" "$green_site"
echo 1 >"$work/exit"
expect fail "gh fails" "not read"

entries=
i=0
while [ "$i" -lt 100 ]; do
	entries="$entries $i"
	i=$((i + 1))
done
# shellcheck disable=SC2086
set -- $entries
out='['
sep=
for _ in "$@"; do
	out=$out$sep$(run Gate completed success)
	sep=,
done
printf '%s]\n' "$out" >"$work/runs.json"
rm -f "$work/exit"
expect fail "a list at the limit may be cut" "limit"

list "$green_gate" "$green_validation" "$green_site"
commit=5595daf expect usage "a short commit id"
commit=5595DAF8E0F048695725A64FF47143BD89F35739 expect usage "an upper-case commit id"
commit= expect usage "no commit"

rm -rf "$work"
[ "$fails" -eq 0 ]
