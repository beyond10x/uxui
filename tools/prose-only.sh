#!/bin/sh
# Whether a change touches only planning and prose files, which no check of the gate reads.
# Usage: <changed paths, one per line> | sh tools/prose-only.sh
# Exit 0 when there is at least one path and every path is planning or prose: the pull-request
# gate then ends green without building. Exit 1 otherwise, naming the first path that runs the
# gate. An empty list runs the gate.
#
# This is the one list. A path is planning or prose when it is
# - anything under .engineering/ (the AEP planning store), or
# - one of these Markdown files outside the built documentation site, which no tool reads:
#   AGENTS.md, CLAUDE.md, README.md, research/*.md, reviews/*.md, and docs/*.md except the three
#   that tools/docs.sh generates pages from (docs/metrics.md, docs/brief.md, docs/roadmap.md) and
#   the one that tools/studio-campaign.sh writes its tables into and a test of engine/search
#   reads (docs/campaign-2026-11-26-studio-engine.md).
# Everything else runs the gate, among it website/ (the built site), plugin/ (the skill),
# CHANGELOG.md (tools/check-plugin.sh reads it), ess/, engine/ and generators/ (test fixtures
# included), tools/ and .github/. tools/gate.test.sh fails when a tool names a file this list
# lets skip.
set -u

prose() {
	case $1 in
	.engineering/*) return 0 ;;
	docs/metrics.md | docs/brief.md | docs/roadmap.md) return 1 ;;
	docs/campaign-2026-11-26-studio-engine.md) return 1 ;;
	AGENTS.md | CLAUDE.md | README.md) return 0 ;;
	research/*.md | reviews/*.md | docs/*.md) return 0 ;;
	*) return 1 ;;
	esac
}

count=0
while IFS= read -r path || [ -n "$path" ]; do
	[ -n "$path" ] || continue
	count=$((count + 1))
	if ! prose "$path"; then
		printf 'gate: %s is not a planning or prose file\n' "$path"
		exit 1
	fi
	printf 'prose: %s\n' "$path"
done

if [ "$count" -eq 0 ]; then
	printf 'gate: no changed path was named\n'
	exit 1
fi
printf 'prose only: %d planning or prose file(s); the gate does not build\n' "$count"
