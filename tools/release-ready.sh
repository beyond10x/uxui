#!/bin/sh
# Whether a commit may carry a release tag: every required workflow has a run on it and every such
# run ended success. Run before `git tag` (AGENTS.md, Release): `task release:ready -- <sha>`.
# Usage: sh tools/release-ready.sh <sha>, the full 40-character commit id.
#
# It asks the forge with `gh run list --repo beyond10x/uxui --commit <sha>` (POSIX sh and jq).
# Required are the workflows Gate (.github/workflows/ci.yml), Documentation validation
# (pages.yml) and Documentation site (b10x-docs-site.yml), by their exact names. A required
# workflow without a run on the commit is absent; a run of it that is not completed is pending;
# a completed run whose conclusion is not exactly success is red. Runs of other workflows and runs
# on other commits are printed and do not count.
# Exit 0 when every required workflow ended success; 1 naming each absent, pending or red one, or
# when the runs cannot be read; 2 on a usage error.
set -u

repository=beyond10x/uxui
required='Gate
Documentation validation
Documentation site'
limit=100

sha=${1:-}
case $sha in *[!0-9a-f]* | '')
	echo "usage: sh tools/release-ready.sh <sha>, the full commit id in lower case (git rev-parse <ref>)" >&2
	exit 2 ;;
esac
if [ $# -ne 1 ] || [ "${#sha}" -ne 40 ]; then
	echo "usage: sh tools/release-ready.sh <sha>, the full commit id in lower case (git rev-parse <ref>)" >&2
	exit 2
fi

refuse() {
	printf 'release-ready: refused: %s\n' "$*" >&2
	exit 1
}

runs=$(gh run list --repo "$repository" --commit "$sha" --limit "$limit" \
	--json conclusion,headSha,status,workflowName) ||
	refuse "the workflow runs of $sha were not read: gh run list failed"

printf '%s\n' "$runs" | jq -e '
	type == "array"
	and all(.[]; type == "object"
		and (.conclusion | type) == "string" and (.headSha | type) == "string"
		and (.status | type) == "string" and (.workflowName | type) == "string")' >/dev/null 2>&1 ||
	refuse "the workflow runs of $sha were not read: the answer is not a list of runs with a workflowName, status, conclusion and headSha"

count=$(printf '%s\n' "$runs" | jq length)
[ "$count" -lt "$limit" ] ||
	refuse "the workflow runs of $sha were not read whole: the list holds $count runs and may be cut at its limit of $limit"

report=$(printf '%s\n' "$runs" | jq -r --arg sha "$sha" --arg required "$required" '
	($required | split("\n")) as $required
	| def state: if .status != "completed" then (if .status == "" then "no status" else .status end)
		elif .conclusion == "" then "no conclusion" else .conclusion end;
	(map(select(.headSha == $sha))) as $mine
	| (.[] | select(.headSha != $sha)
		| "note\tnot counted: \(.workflowName): \(state) on \(.headSha), another commit"),
	  ($mine[] | select(.workflowName as $n | $required | index([$n]) | not)
		| "note\tnot required: \(.workflowName): \(state)"),
	  ($mine[] | select(.workflowName as $n | $required | index([$n]))
		| "note\t\(.workflowName): \(state)",
		  (if .status != "completed" then "block\t\(.workflowName): pending (\(state))"
		   elif .conclusion != "success" then "block\t\(.workflowName): red (\(state))"
		   else empty end)),
	  ($required[] as $n | select([$mine[] | select(.workflowName == $n)] | length == 0)
		| "block\t\($n): absent")') ||
	refuse "the workflow runs of $sha were not read: jq failed on them"

printf '%s\n' "$report" | sed -n 's/^note	/release-ready: /p'
blocking=$(printf '%s\n' "$report" | sed -n 's/^block	/  /p')
if [ -n "$blocking" ]; then
	printf 'release-ready: refused: no release tag on %s; these required runs did not end success:\n%s\n' \
		"$sha" "$blocking" >&2
	exit 1
fi
printf 'release-ready: %s may be tagged: Gate, Documentation validation and Documentation site ended success\n' "$sha"
