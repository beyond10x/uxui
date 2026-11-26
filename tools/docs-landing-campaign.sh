#!/bin/sh
# The campaign of docs/campaign-2026-11-26-docs-landing-engine.md, from the sources to its files:
# builds the documentation site, builds it once more from one variant's patched sources, and runs
# `ux-search landing run` with that second site as the check of the probe's render (`--rebuilt`).
# Usage: sh tools/docs-landing-campaign.sh <out dir>
# Needs: node and npm, Chromium (UXUI_CHROME, else the Playwright cache), and ux-search built:
#   cargo build --manifest-path engine/search/Cargo.toml --locked
# (UX_SEARCH names another binary). Writes <out dir>/measurements.json and two screenshots per
# frontier variant, prints the report (the builds print on standard error); the two site builds go
# to website/build and engine/search/target/landing/rebuilt.
#
# The second build needs the variant's sources in website/: the script copies them in from
# engine/search/tests/fixtures/docs-landing/ (variant.product.json, variant.landing.css) and puts
# the sources back when it ends, also when it fails. It refuses to start when website/'s two
# sources are not the fixture's, so it overwrites nothing that is not kept there.
set -eu

out=${1:?usage: sh tools/docs-landing-campaign.sh <out dir>}
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

bin=${UX_SEARCH:-engine/search/target/debug/ux-search}
fixtures=engine/search/tests/fixtures/docs-landing
# The variant the fixture's variant.* files are: no axis at its shipped value.
variant=studio.centred.1+3.status-first.d85.h100
rebuilt=engine/search/target/landing/rebuilt
product=website/product.json
css=website/src/css/landing.css

if [ ! -x "$bin" ]; then
	echo "docs-landing-campaign: no $bin; build it: cargo build --manifest-path engine/search/Cargo.toml --locked" >&2
	exit 1
fi
if ! cmp -s "$product" "$fixtures/product.json" || ! cmp -s "$css" "$fixtures/landing.css"; then
	echo "docs-landing-campaign: $product and $css are not the sources kept in $fixtures" >&2
	exit 1
fi

(cd website && { npm ci --allow-git=root || npm ci; } && npm run build) >&2

restore() {
	cp "$fixtures/product.json" "$product"
	cp "$fixtures/landing.css" "$css"
}
trap restore EXIT
trap 'exit 1' HUP INT QUIT PIPE TERM
cp "$fixtures/variant.product.json" "$product"
cp "$fixtures/variant.landing.css" "$css"
(cd website && npx docusaurus build --out-dir "../$rebuilt") >&2
restore
trap - EXIT HUP INT QUIT PIPE TERM

"$bin" landing run --build website/build --product "$product" --css "$css" --out "$out" \
	--rebuilt "$variant=$rebuilt"
