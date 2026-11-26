#!/bin/sh
# The campaign of docs/campaign-2026-11-26-docs-landing-engine.md, from the sources to its files:
# builds `ux-search` and the documentation site, builds the site once more from one variant's
# patched sources, runs `ux-search landing run` with that second site as the check of the probe's
# render (`--rebuilt`), and records the result where the tests read it.
# Usage: sh tools/docs-landing-campaign.sh [--pool]
# Needs: node and npm, cargo, and Chromium: UXUI_CHROME, else the headless shell of the Playwright
# build named below (`chrome_build`), the build the committed measurements were recorded with.
# UX_SEARCH names another ux-search binary than the one it builds.
# Writes, and nothing else in the tree:
#   engine/search/tests/fixtures/docs-landing/measurements.json and report.txt
#   engine/search/tests/fixtures/docs-landing/promote.evidence.json, the golden evidence of the
#     tests of `landing promote`, through their own switch (UX_SEARCH_BLESS)
#   docs/campaign-2026-11-26-docs-landing-engine/*.png, the screenshots of the frontier (the ones
#     that were there are removed)
# and prints the report (the builds and the run print on standard error). The numbers in the
# campaign's document are written by hand from the report. The two site builds go to
# website/build and engine/search/target/landing/rebuilt, the run's own files to
# engine/search/target/landing/run.
#
# The second site is built in a copy of website/ under engine/search/target/landing/ that holds
# the variant's two sources (variant.product.json, variant.landing.css of the fixture) and links
# website/node_modules: website/ itself is not written. The script refuses to start when website/'s
# two sources are not the fixture's, so it measures what is kept there.
#
# It compiles (cargo, two site builds) and then measures. On a host with a build pool that is one
# job of the pool, and `--pool` is the one command there: it starts this script as that job, which
# runs on a build node and copies the written files back, and then removes the screenshots the
# run did not write (the copy adds files and removes none). Start no other job of the same
# checkout until it ends: the next job's copy of the checkout would put the fixture's old files
# back before the new ones are fetched.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

fixtures=engine/search/tests/fixtures/docs-landing
shots=docs/campaign-2026-11-26-docs-landing-engine
manifest=engine/search/Cargo.toml
work=engine/search/target/landing
run=$work/run
site=$work/variant-site
rebuilt=$work/rebuilt
# The variant the fixture's variant.* files are: no axis at its shipped value.
variant=studio.centred.1+3.status-first.d115.h100
product=website/product.json
css=website/src/css/landing.css
# The Playwright build of the Chromium headless shell the campaign is recorded with.
chrome_build=1254

case ${1:-} in
'') ;;
--pool)
	if [ $# -ne 1 ]; then
		echo "usage: sh tools/docs-landing-campaign.sh [--pool]" >&2
		exit 2
	fi
	rm -rf "$run"
	slot --timeout 30m --fetch "$fixtures" --fetch "$shots" --fetch "$run" \
		sh tools/docs-landing-campaign.sh
	for shot in "$shots"/*.png; do
		[ -e "$shot" ] || continue
		if [ ! -e "$run/$(basename "$shot")" ]; then
			rm -f "$shot"
			echo "docs-landing-campaign: removed $shot, no screenshot of this frontier" >&2
		fi
	done
	exit 0
	;;
*)
	echo "usage: sh tools/docs-landing-campaign.sh [--pool]" >&2
	exit 2
	;;
esac

if [ -z "${UXUI_CHROME:-}" ]; then
	cache=${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}
	UXUI_CHROME=$cache/chromium_headless_shell-$chrome_build/chrome-headless-shell-linux64/chrome-headless-shell
fi
if [ ! -x "$UXUI_CHROME" ]; then
	echo "docs-landing-campaign: no Chromium at $UXUI_CHROME: set UXUI_CHROME, or install the headless shell of Playwright build $chrome_build" >&2
	exit 3
fi
export UXUI_CHROME
if ! cmp -s "$product" "$fixtures/product.json" || ! cmp -s "$css" "$fixtures/landing.css"; then
	echo "docs-landing-campaign: $product and $css are not the sources kept in $fixtures" >&2
	exit 1
fi

# 1. The engine. Cargo may build into a target directory outside the checkout (a build node
# does), so the binary is where cargo says its target directory is.
if [ -n "${UX_SEARCH:-}" ]; then
	bin=$UX_SEARCH
else
	cargo build --manifest-path "$manifest" --locked >&2
	target=$(cargo metadata --manifest-path "$manifest" --locked --no-deps --format-version 1 |
		sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
	bin=$target/debug/ux-search
fi
if [ ! -x "$bin" ]; then
	echo "docs-landing-campaign: no $bin" >&2
	exit 1
fi

# 2. The site as shipped.
(cd website && { npm ci --allow-git=root || npm ci; } && npm run build) >&2

# 3. The site of one variant, built in a copy of website/ with the variant's sources.
rm -rf "$site" "$rebuilt"
mkdir -p "$site"
for entry in website/* website/.[!.]*; do
	[ -e "$entry" ] || continue
	case $(basename "$entry") in
	node_modules | build | .docusaurus) continue ;;
	esac
	cp -R "$entry" "$site/"
done
ln -s "$root/website/node_modules" "$site/node_modules"
cp "$fixtures/variant.product.json" "$site/product.json"
cp "$fixtures/variant.landing.css" "$site/src/css/landing.css"
(cd "$site" && npx docusaurus build --out-dir "$root/$rebuilt") >&2

# 4. The campaign.
rm -rf "$run"
mkdir -p "$run"
echo "docs-landing-campaign: $("$bin" --version), $("$UXUI_CHROME" --version)" >&2
"$bin" landing run --build website/build --product "$product" --css "$css" --out "$run" \
	--profile "$work/profile" --rebuilt "$variant=$rebuilt" >"$run/report.txt"

# 5. The record: the measurements, the report over them, the frontier's screenshots.
cp "$run/measurements.json" "$fixtures/measurements.json"
"$bin" landing report --measurements "$fixtures/measurements.json" >"$fixtures/report.txt"
cmp "$fixtures/report.txt" "$run/report.txt" || {
	echo "docs-landing-campaign: the report over the written measurements is not the report of the run" >&2
	exit 1
}
mkdir -p "$shots"
rm -f "$shots"/*.png
cp "$run"/*.png "$shots/"

# 6. The golden evidence of the tests of `landing promote`, which names the measurements' digest.
UX_SEARCH_BLESS=1 cargo test --manifest-path "$manifest" --locked landing_promote >&2

cat "$fixtures/report.txt"
echo "docs-landing-campaign: recorded $fixtures/measurements.json, $fixtures/report.txt and promote.evidence.json, and $(ls "$run"/*.png | wc -l | tr -d ' ') screenshots in $shots" >&2
