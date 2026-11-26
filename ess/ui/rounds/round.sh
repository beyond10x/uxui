#!/bin/sh
# The round command of the ui-iteration-loop round on the studio (plugin/skills/ui-iteration-loop,
# § 3.2 Measure): generates the React app from ess/ui/studio.ui.yaml on its fixtures, links the
# studio theme, builds and serves it on a loopback port, and measures every page with ux-browser.
# Usage: sh ess/ui/rounds/round.sh <round id, as R-01>
# Needs: ess, node and npm (the registry, for the generated app's own packages), cargo, and
# Chromium: UXUI_CHROME, else the newest Chromium headless shell under the Playwright cache.
# Writes <target dir>/loop-round/<round id>/: versions.txt, page-checks.evidence.json, capture/,
# render/<page>.<viewport>.<mode>.json and .png, dom/<page>.html, and what trial.mjs writes
# (trial/, shots/, audit/), and round.json (summary.mjs). Nothing under ess/ui/ is written: the round's
# report takes its numbers from round.json, which is copied into ess/ui/rounds/<round id>/.
#
# It compiles (cargo, tsc, esbuild) and then measures, in one job: on a host with a build pool,
#   slot --timeout 25m --fetch target/loop-round/R-01 sh ess/ui/rounds/round.sh R-01
set -eu

round=${1:?usage: sh ess/ui/rounds/round.sh <round id>}
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
cd "$root"

target=${CARGO_TARGET_DIR:-$root/target}
work=$target/loop-round
app=$work/app
out=$work/$round
config=ess/ui/rounds/studio.checks.yaml
render=ess/ui/rounds/studio.render.yaml
# The gating viewports of the round (the skill's adapter table); the checks file measures three more.
gating='1366x768 1280x720'

if [ -z "${UXUI_CHROME:-}" ]; then
	cache=${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}
	UXUI_CHROME=$(ls -d "$cache"/chromium_headless_shell-*/chrome-headless-shell-linux64/chrome-headless-shell 2>/dev/null | sort | tail -1)
fi
if [ ! -x "${UXUI_CHROME:-}" ]; then
	echo "round: no Chromium: set UXUI_CHROME" >&2
	exit 3
fi
export UXUI_CHROME

rm -rf "$app" "$out"
mkdir -p "$out/render" "$out/dom" "$out/capture" "$work/decl"

# 1. The app: generated, themed, built.
ess generate ui --target react --path ess/ui/studio.ui.yaml --model ess --fixtures --out "$app"
# The theme is linked by an edit of the generated index.html (docs/change-list.md, the theme row).
cp themes/studio.css themes/theme-toggle.js "$app/www/"
sed -e 's|<link rel="stylesheet" href="/assets/main.css" />|&\
    <link rel="stylesheet" href="/studio.css" />\
    <script src="/theme-toggle.js"></script>|' "$app/www/index.html" >"$app/www/index.html.new"
mv "$app/www/index.html.new" "$app/www/index.html"
grep -q 'studio.css' "$app/www/index.html" || {
	echo "round: the theme was not linked into www/index.html" >&2
	exit 1
}
(cd "$app" && npm install --no-audit --no-fund && npm run build) >&2

cargo build --locked -p ux-browser >&2
bin=$target/debug/ux-browser

{
	printf 'ess: %s\n' "$(ess --version)"
	printf 'node: %s\n' "$(node --version)"
	printf 'npm: %s\n' "$(npm --version)"
	for package in react react-dom esbuild typescript; do
		printf '%s: %s\n' "$package" "$(node -p "require('$app/node_modules/$package/package.json').version")"
	done
	printf 'ux-browser: %s\n' "$("$bin" --version)"
	printf 'chromium: %s\n' "$("$UXUI_CHROME" --version)"
	printf 'studio.ui.yaml: sha256:%s\n' "$(sha256sum ess/ui/studio.ui.yaml | cut -d' ' -f1)"
	printf 'studio.css: sha256:%s\n' "$(sha256sum themes/studio.css | cut -d' ' -f1)"
	printf 'www: sha256:%s\n' "$(cd "$app/www" && find . -type f | sed 's|^\./||' | LC_ALL=C sort | xargs sha256sum | sha256sum | cut -d' ' -f1)"
} >"$out/versions.txt"

# 2. The server: esbuild's, which answers every page path with index.html.
(cd "$app" && exec ./node_modules/.bin/esbuild --servedir=www --serve-fallback=www/index.html --serve=127.0.0.1:0 --watch=forever) >"$work/serve.log" 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null || true' EXIT
trap 'exit 1' HUP INT QUIT PIPE TERM
origin=
tries=0
while [ -z "$origin" ]; do
	tries=$((tries + 1))
	if [ "$tries" -gt 100 ]; then
		echo "round: the server did not start:" >&2
		cat "$work/serve.log" >&2
		exit 3
	fi
	sleep 0.1
	origin=$(sed -n 's|.*\(http://127\.0\.0\.1:[0-9][0-9]*\).*|\1|p' "$work/serve.log" | head -1)
done

# 3. The shared page checks, on every page of the checks file.
status=0
"$bin" checks --config "$config" --url "$origin" --project uxui --assessment "studio-round-$round" \
	--capture "$out/capture" --out "$out/page-checks.evidence.json" || status=$?
if [ "$status" -ne 0 ]; then
	echo "round: ux-browser checks exited $status" >&2
	exit "$status"
fi

# 4. One render record, screenshot and DOM per page, gating viewport and mode.
paths=$(awk '/^  paths:/ { on = 1; next } on && /^    - / { print $2; next } on { exit }' "$config")
for path in $paths; do
	name=$(printf '%s' "$path" | sed -e 's|^/||' -e 's|/|.|g' -e 's|sha256%3A||g')
	"$UXUI_CHROME" --headless --disable-gpu --hide-scrollbars --window-size=1366,768 \
		--user-data-dir="$work/dump-profile" --virtual-time-budget=4000 --dump-dom "$origin$path" \
		>"$out/dom/$name.html" 2>/dev/null || echo "round: no DOM of $path" >&2
	for viewport in $gating; do
		for mode in light dark; do
			decl=$work/decl/$viewport.$mode.render.yaml
			sed -e "s|^viewport: .*|viewport: {width: ${viewport%x*}, height: ${viewport#*x}}|" \
				-e "s|^mode: .*|mode: $mode|" "$render" >"$decl"
			"$bin" render --declarations "$decl" --url "$origin$path" \
				--out "$out/render/$name.$viewport.$mode.json"
		done
	done
done
rm -rf "$work/dump-profile"

# 5. The trial without a person: the scenario cards played and each page audited.
node ess/ui/rounds/trial.mjs "$origin" ess/ui/rounds/studio.scenarios.json "$out" \
	--pages "$(printf '%s' "$paths" | tr '\n' ',' | sed 's/,$//')" --viewports "$(printf '%s' "$gating" | tr ' ' ',')"
rm -rf "$work"/trial-profile-*

# 6. The numbers of the round as one file (the round.json of § 3.2), for the round directory.
node ess/ui/rounds/summary.mjs "$round" "$out" "$out/round.json"
echo "round: $round written to $out"
