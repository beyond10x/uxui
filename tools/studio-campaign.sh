#!/bin/sh
# The campaign of docs/campaign-2026-11-26-studio-engine.md, from the sources to its files:
# generates the React app from ess/ui/studio.ui.yaml on its fixtures, links the studio theme,
# builds and serves it on a loopback port, runs `ux-search campaign run` over the declaration
# engine/search/tests/fixtures/campaign/studio/frontier.campaign.yaml, runs it a second time and
# compares the two measurements files byte for byte, and records the result. Before the runs it
# runs the engine's tests that start a browser (`campaign_live`), which the gate does not run.
# Usage: sh tools/studio-campaign.sh [--pool]
# Needs: ess, node and npm (the registry, for the generated app's own packages), cargo, and
# Chromium: UXUI_CHROME, else the newest Chromium headless shell under the Playwright cache.
# UX_SEARCH names another ux-search binary than the one it builds.
# Writes, and nothing else in the tree:
#   engine/search/tests/fixtures/campaign/studio/measurements.json and report.txt
#   docs/campaign-2026-11-26-studio-engine/*.png, the screenshots of the frontier (the ones that
#     were there are removed)
#   docs/campaign-2026-11-26-studio-engine.md, between its `recorded` and its `versions` marks
# and prints the report (the builds and the runs print on standard error). The app, the two runs
# and the server's log are under <target dir>/studio-campaign/.
#
# It compiles (cargo, tsc, esbuild) and then measures the app it built. On a host with a build
# pool that is one job of the pool, and `--pool` is the one command there: it starts this script
# as that job, which runs on a build node and copies the written files back, and then removes the
# screenshots the document no longer names (the copy adds files and removes none).
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

fixtures=engine/search/tests/fixtures/campaign/studio
shots=docs/campaign-2026-11-26-studio-engine
document=docs/campaign-2026-11-26-studio-engine.md
target=${CARGO_TARGET_DIR:-$root/engine/search/target}
work=$target/studio-campaign
app=$work/app

case ${1:-} in
'') ;;
--pool)
	if [ $# -ne 1 ]; then
		echo "usage: sh tools/studio-campaign.sh [--pool]" >&2
		exit 2
	fi
	slot --timeout 20m --fetch "$fixtures" --fetch "$shots" --fetch "$document" \
		sh tools/studio-campaign.sh
	for shot in "$shots"/*.png; do
		[ -e "$shot" ] || continue
		if ! grep -qF "($(basename "$shots")/$(basename "$shot"))" "$document"; then
			rm -f "$shot"
			echo "studio-campaign: removed $shot, no screenshot of this frontier" >&2
		fi
	done
	exit 0
	;;
*)
	echo "usage: sh tools/studio-campaign.sh [--pool]" >&2
	exit 2
	;;
esac

if [ -z "${UXUI_CHROME:-}" ]; then
	cache=${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}
	UXUI_CHROME=$(ls -d "$cache"/chromium_headless_shell-*/chrome-headless-shell-linux64/chrome-headless-shell 2>/dev/null | sort | tail -1)
fi
if [ ! -x "${UXUI_CHROME:-}" ]; then
	echo "studio-campaign: no Chromium: set UXUI_CHROME" >&2
	exit 3
fi
export UXUI_CHROME
for mark in recorded versions; do
	for edge in begin end; do
		if [ "$(grep -c "^<!-- $mark: $edge -->\$" "$document")" != 1 ]; then
			echo "studio-campaign: $document does not have the line <!-- $mark: $edge --> once" >&2
			exit 1
		fi
	done
done

rm -rf "$work"
mkdir -p "$work/first" "$work/second"

# 1. The app: generated, themed, built.
ess generate ui --target react --path ess/ui/studio.ui.yaml --model ess --fixtures --out "$app" >&2
# The theme is linked by an edit of the generated index.html (docs/change-list.md, the theme row).
cp themes/studio.css themes/theme-toggle.js "$app/www/"
sed -e 's|<link rel="stylesheet" href="/assets/main.css" />|&\
    <link rel="stylesheet" href="/studio.css" />\
    <script src="/theme-toggle.js"></script>|' "$app/www/index.html" >"$app/www/index.html.new"
mv "$app/www/index.html.new" "$app/www/index.html"
grep -q 'studio.css' "$app/www/index.html" || {
	echo "studio-campaign: the theme was not linked into www/index.html" >&2
	exit 1
}
(cd "$app" && npm install --no-audit --no-fund && npm run build) >&2

# 2. The engine.
if [ -n "${UX_SEARCH:-}" ]; then
	bin=$UX_SEARCH
else
	cargo build --manifest-path engine/search/Cargo.toml --locked >&2
	bin=$target/debug/ux-search
fi
if [ ! -x "$bin" ]; then
	echo "studio-campaign: no $bin" >&2
	exit 1
fi

# The engine's tests that need a browser (module `campaign_live`: the pointer, a mode's read-back,
# the screenshots, the notes). The gate has no browser and starts none of them, so a recording is
# where they run: a campaign is recorded only with an engine that passes them.
cargo test --manifest-path engine/search/Cargo.toml --locked -- --ignored campaign_live >&2

# 3. The server: esbuild's, which answers every page path with index.html.
(cd "$app" && exec ./node_modules/.bin/esbuild --servedir=www --serve-fallback=www/index.html --serve=127.0.0.1:0 --watch=forever) >"$work/serve.log" 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null || true' EXIT
trap 'exit 1' HUP INT QUIT PIPE TERM
origin=
tries=0
while [ -z "$origin" ]; do
	tries=$((tries + 1))
	if [ "$tries" -gt 100 ]; then
		echo "studio-campaign: the server did not start:" >&2
		cat "$work/serve.log" >&2
		exit 3
	fi
	sleep 0.1
	origin=$(sed -n 's|.*\(http://127\.0\.0\.1:[0-9][0-9]*\).*|\1|p' "$work/serve.log" | head -1)
done

# 4. The campaign, twice: the second run must write the bytes of the first.
for run in first second; do
	"$bin" campaign run --declaration "$fixtures/frontier.campaign.yaml" --url "$origin" \
		--out "$work/$run/measurements.json" --shots "$work/$run/shots" \
		--profile "$work/profile-$run" >"$work/$run/report.txt" 2>"$work/$run/run.log" || {
		cat "$work/$run/run.log" >&2
		echo "studio-campaign: the $run run failed" >&2
		exit 1
	}
	cat "$work/$run/run.log" >&2
done
cmp "$work/first/measurements.json" "$work/second/measurements.json" || {
	echo "studio-campaign: the second run did not write the measurements of the first" >&2
	exit 1
}
if [ ! -d "$work/first/shots" ]; then
	echo "studio-campaign: the campaign ends without a frontier: every variant breaks a hard rule" >&2
	exit 1
fi
same=0
all=0
for shot in "$work"/first/shots/*.png; do
	all=$((all + 1))
	if cmp -s "$shot" "$work/second/shots/$(basename "$shot")"; then
		same=$((same + 1))
	else
		echo "studio-campaign: the two runs' screenshots $(basename "$shot") are not the same bytes" >&2
	fi
done

# 5. The record: the measurements, the report over them, the frontier's screenshots.
cp "$work/first/measurements.json" "$fixtures/measurements.json"
"$bin" campaign report --measurements "$fixtures/measurements.json" >"$fixtures/report.txt"
cmp "$fixtures/report.txt" "$work/first/report.txt" || {
	echo "studio-campaign: the report over the written measurements is not the report of the run" >&2
	exit 1
}
mkdir -p "$shots"
rm -f "$shots"/*.png
cp "$work"/first/shots/*.png "$shots/"

# 6. The document: the run and the frontier as the measurements give them, and what they were
# made with.
"$bin" campaign report --measurements "$fixtures/measurements.json" --format markdown \
	--links "$(basename "$shots")/" >"$work/recorded.md"
digest() {
	printf 'sha256:%s' "$(sha256sum "$1" | cut -d' ' -f1)"
}
version() {
	node -p "require('$app/node_modules/$1/package.json').version"
}
timing() {
	sed -n 's/^ux-search: \([0-9][0-9]* renders in .*\)$/\1/p' "$work/$1/run.log"
}
{
	printf '| | |\n|---|---|\n'
	printf '| `ess/ui/studio.ui.yaml` | `%s` |\n' "$(digest ess/ui/studio.ui.yaml)"
	printf '| `themes/studio.css` | `%s` |\n' "$(digest themes/studio.css)"
	printf '| the served app, `www/` | `sha256:%s` |\n' "$(cd "$app/www" && find . -type f | sed 's|^\./||' | LC_ALL=C sort | xargs sha256sum | sha256sum | cut -d' ' -f1)"
	printf '| generator | `%s` |\n' "$(ess --version)"
	printf '| node, npm | `%s`, `%s` |\n' "$(node --version)" "$(npm --version)"
	printf '| react, react-dom, esbuild, typescript | `%s`, `%s`, `%s`, `%s` |\n' \
		"$(version react)" "$(version react-dom)" "$(version esbuild)" "$(version typescript)"
	printf '| Chromium | `%s` |\n' "$("$UXUI_CHROME" --version)"
	printf '| engine | `%s` |\n' "$("$bin" --version | head -n 1)"
	printf '| first run | %s |\n' "$(timing first)"
	printf '| second run | %s; the same bytes of `measurements.json` as the first run, %s of %s screenshots the same bytes |\n' \
		"$(timing second)" "$same" "$all"
} >"$work/versions.md"
awk -v recorded="$work/recorded.md" -v versions="$work/versions.md" '
	function put(file,    line) {
		while ((getline line < file) > 0) print line
		close(file)
	}
	/^<!-- recorded: begin -->$/ { print; put(recorded); skip = 1; next }
	/^<!-- versions: begin -->$/ { print; put(versions); skip = 1; next }
	/^<!-- (recorded|versions): end -->$/ { skip = 0 }
	!skip { print }
' "$document" >"$work/document.md"
cp "$work/document.md" "$document"

cat "$fixtures/report.txt"
echo "studio-campaign: recorded $fixtures/measurements.json, $fixtures/report.txt, $all screenshots in $shots and the numbers of $document" >&2
