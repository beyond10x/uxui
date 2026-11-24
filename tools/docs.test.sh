#!/bin/sh
# Proves that `tools/docs.sh check` passes on this tree and fails on each broken copy of it, and
# that `tools/docs.sh provenance` binds a site to its commit. Copies go under target/docs-test/ in
# the repository; the cases run side by side. Exit 0 when every case holds.
set -u

root=$(cd "$(dirname "$0")/.." && pwd)
work=$root/target/docs-test
docs=$root/tools/docs.sh
rm -rf "$work"
mkdir -p "$work"

n=0
labels=
# case <pass|fail> <label> <mutation>: the mutation runs in a fresh copy of the tree ($w) before
# the check.
case_() {
	n=$((n + 1))
	w=$work/$n
	printf '%s\t%s\n' "$1" "$2" >"$w.case"
	(
		mkdir -p "$w/website"
		cp -r "$root/docs" "$root/ess" "$root/plugin" "$w/"
		cp -r "$root/website/docs" "$root/website/data" "$root/website/product.json" "$w/website/"
		eval "$3"
		DOCS_ROOT=$w sh "$docs" check >"$w.out" 2>&1
		echo $? >"$w.rc"
	) &
}

case_ pass "unchanged tree" ':'
case_ fail "generated page edited" 'echo "An edit by hand." >>"$w/website/docs/reference/metrics.md"'
case_ fail "generated page missing" 'rm "$w/website/docs/roadmap.md"'
case_ fail "source changed, generated page stale" 'printf "\n- One more step.\n" >>"$w/docs/roadmap.md"'
case_ fail "brace outside code" 'echo "A brace { outside code." >>"$w/website/docs/limitations.md"'
case_ fail "capitalised tag outside code" 'echo "A tag <Status/> here." >>"$w/website/docs/limitations.md"'
case_ fail "import line" 'echo "import x from \"y\";" >>"$w/website/docs/limitations.md"'
case_ fail "home path" 'printf "See /%s/someone/notes.\n" home >>"$w/website/docs/limitations.md"'
case_ fail "page names uilab" 'echo "Built on uilab." >>"$w/website/docs/limitations.md"'
case_ fail "page without description" 'sed "/^description:/d" "$w/website/docs/getting-started.md" >"$w/s" && mv "$w/s" "$w/website/docs/getting-started.md"'
case_ fail "concept page without lede" 'sed "/^lede:/d" "$w/website/docs/concepts/the-loop.md" >"$w/s" && mv "$w/s" "$w/website/docs/concepts/the-loop.md"'
case_ fail "unknown status" 'jq ".items[0].status = \"done\"" "$w/website/data/status.json" >"$w/s" && mv "$w/s" "$w/website/data/status.json"'
case_ fail "landing page of another format" 'jq ".format = \"other/1\"" "$w/website/product.json" >"$w/s" && mv "$w/s" "$w/website/product.json"'
case_ pass "braces and tags inside code" 'printf "Code: \`{a} <B/>\`\n\n\`\`\`text\n{b} <C/>\n\`\`\`\n" >>"$w/website/docs/limitations.md"'
wait

fails=0
i=0
while [ "$i" -lt "$n" ]; do
	i=$((i + 1))
	want=$(cut -f 1 "$work/$i.case")
	label=$(cut -f 2 "$work/$i.case")
	got=$(cat "$work/$i.rc")
	if [ "$want" = pass ] && [ "$got" -eq 0 ]; then
		printf 'ok   %s\n' "$label"
	elif [ "$want" = fail ] && [ "$got" -ne 0 ]; then
		printf 'ok   %s (%s)\n' "$label" "$(grep -m 1 '^FAIL' "$work/$i.out")"
	else
		printf 'FAIL %s: expected %s, exit %s\n' "$label" "$want" "$got"
		cat "$work/$i.out"
		fails=$((fails + 1))
	fi
done

# provenance
commit=0123456789abcdef0123456789abcdef01234567
site=$work/site
mkdir -p "$site/docs/a"
printf '<html><h1 id="top">x</h1><main id=main></main></html>' >"$site/index.html"
printf '<html><h2 id="b">b</h2><h2 id="a">a</h2><h2 id="a">a</h2></html>' >"$site/docs/a.html"
printf '<html><!-- b10x-trailing-slash-copy --><h2 id="b">b</h2></html>' >"$site/docs/a/index.html"
printf '<html>not found</html>' >"$site/404.html"

prov() {
	want=$1
	label=$2
	got=$3
	n=$((n + 1))
	if { [ "$want" = pass ] && [ "$got" -eq 0 ]; } || { [ "$want" = fail ] && [ "$got" -ne 0 ]; }; then
		printf 'ok   %s\n' "$label"
	else
		printf 'FAIL %s: expected %s, exit %s\n' "$label" "$want" "$got"
		cat "$work/prov.out"
		fails=$((fails + 1))
	fi
}

got=1
if DOCS_ROOT=$root sh "$docs" provenance "$site" "$commit" >"$work/prov.out" 2>&1 &&
	jq -e --arg c "$commit" '.schema == "b10x-project-site/v1" and .repository == "uxui" and .commit == $c and .baseUrl == "/uxui/"' "$site/.well-known/b10x-site.json" >/dev/null &&
	jq -e '.schema == "b10x-project-routes/v1" and .routes == [{path: "/uxui/", anchors: ["main", "top"]}, {path: "/uxui/docs/a/", anchors: ["a", "b"]}]' "$site/.well-known/b10x-routes.json" >/dev/null &&
	[ -f "$site/.nojekyll" ]; then
	got=0
fi
prov pass "provenance writes the manifest, the routes and .nojekyll" "$got"

DOCS_ROOT=$root sh "$docs" provenance "$site" ABC >"$work/prov.out" 2>&1
prov fail "provenance refuses a short commit" $?

rm -f "$site/index.html"
DOCS_ROOT=$root sh "$docs" provenance "$site" "$commit" >"$work/prov.out" 2>&1
prov fail "provenance refuses a site without a landing page" $?

rm -rf "$work"
if [ "$fails" -gt 0 ]; then
	printf 'docs.test: %d of %d case(s) failed\n' "$fails" "$n"
	exit 1
fi
printf 'pass docs.test: %d cases\n' "$n"
