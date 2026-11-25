#!/bin/sh
# Proves that tools/check-themes.sh passes on this tree and fails on each broken copy of themes/.
# Copies go under target/check-themes-test/ in the repository; the cases run side by side. Exit 0 when
# every case holds.
set -u

root=$(cd "$(dirname "$0")/.." && pwd)
work=$root/target/check-themes-test
check=$root/tools/check-themes.sh
rm -rf "$work"
mkdir -p "$work"

n=0
# case_ <pass|fail> <label> <mutation>: the mutation runs in a fresh copy of themes/ ($w/themes)
# before the check.
case_() {
	n=$((n + 1))
	w=$work/$n
	printf '%s\t%s\n' "$1" "$2" >"$w.case"
	(
		mkdir -p "$w"
		cp -r "$root/themes" "$w/"
		eval "$3"
		sh "$check" "$w" >"$w.out" 2>&1
		echo $? >"$w.rc"
	) &
}

# edit <file under themes/> <sed script>
edit() {
	sed "$2" "$w/themes/$1" >"$w/edit" && mv "$w/edit" "$w/themes/$1"
}

# Drops the dark mode of a stylesheet: the @media block and the picked rule after it.
drop_dark() {
	awk '/^@media \(prefers-color-scheme: dark\) \{$/ { skip = 1 } !skip { print } skip && /^:root\[data-uxui-mode="dark"\] \{$/ { picked = 1 } picked && /^\}$/ { skip = 0; picked = 0 }' \
		"$w/themes/$1" >"$w/edit" && mv "$w/edit" "$w/themes/$1"
}

case_ pass "unchanged tree" ':'
case_ pass "a comment that names a rule" 'printf "/* .x { width: 1px; } */\n" >>"$w/themes/slot-jobs.css"'

case_ fail "every column width deleted (no slot document given)" "edit slot-jobs.css '/^table\.uxui-dense \.uxui-field-/d'"
case_ fail "a width through a custom property no top-level rule sets" "edit slot-jobs.css 's/--uxui-slot-w-repo: 160px; //'"
case_ fail "max-width as a column width" "edit slot-jobs.css 's/\.uxui-field-cpu_s { width:/.uxui-field-cpu_s { max-width:/'"
case_ fail "the dark mode loses a value colour" "edit slot-jobs.css '/--uxui-slot-danger-text: #ff8a93;/d'"
case_ fail "the dark mode only in a comment" 'drop_dark slot-jobs.css && edit slot-jobs.css "1i\\
/* prefers-color-scheme: dark :not([data-uxui-mode=\"light\"]) [data-uxui-mode=\"dark\"] */"'
case_ fail "the colour-mark rule deleted" "edit dense.css 's/^\[data-uxui-value\]::before {/[data-uxui-never]::before {/'"

case_ fail "no width for repo" "edit slot-jobs.css '/uxui-field-repo { width/d'"
case_ fail "a width only in a comment" "edit slot-jobs.css 's|^table\.uxui-dense \.uxui-field-repo { width.*|/* & */|'"
case_ fail "no colour for cancelled" "edit slot-jobs.css 's/data-uxui-value=\"cancelled\"/data-uxui-value=\"x\"/'"
case_ fail "a value colour only in a comment" "edit slot-jobs.css 's|^\.uxui-field-priority\[data-uxui-value=\"high\"\].*|/* & */|'"
case_ fail "dense without the picked dark mode" "edit dense.css 's/^:root\[data-uxui-mode=\"dark\"\] {/:root.x {/'"
case_ fail "slot-jobs without the system's dark mode" "edit slot-jobs.css 's/prefers-color-scheme: dark/prefers-color-scheme: light/'"
case_ fail "a dark mode without color-scheme" "edit dense.css '/^ *color-scheme: dark;/d'"
case_ fail "dense without one-line cells" "edit dense.css 's/ white-space: nowrap; overflow: hidden;/ overflow: hidden;/'"
case_ fail "dense without a label beside its value" "edit dense.css 's/grid-template-columns/grid-template-rows/'"
case_ fail "README misses a file" "edit README.md '/examples\/slot-jobs.html/d'"
case_ fail "a new stylesheet without a dark mode" 'printf ":root { color-scheme: light; --a: #fff; }\n" >"$w/themes/new.css"'
case_ fail "a var() of a property nobody sets" "edit dense.css 's/background: var(--uxui-dense-hover)/background: var(--uxui-dense-hovered)/'"
case_ fail "the hover lights the empty row" "edit dense.css 's/tr:not(\.uxui-empty):hover/tr:hover/'"
case_ fail "dense without the reach into the margin" "edit dense.css 's/width: calc(100% + 2 \* var(--uxui-dense-reach))/width: 100%/'"
case_ fail "slot-jobs without slot's 6 px reach" "edit slot-jobs.css 's/ --uxui-dense-reach: 6px;//'"
case_ fail "the fixture edited" "edit fixtures/slot.ui.yaml 's/label: job}/label: id}/'"
case_ fail "a new column of slot's document without a width" 'edit fixtures/slot.ui.yaml "s/^          - {field: cpu_s, label: CPU s, as: number}\$/&\\
          - {field: exit, as: number}/" &&
	sum=$(sha256sum "$w/themes/fixtures/slot.ui.yaml" | cut -d " " -f 1) &&
	edit fixtures/README.md "s/[0-9a-f]\{64\}/$sum/"'
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

printf 'check-themes.test: %s cases, %s failed\n' "$n" "$fails"
rm -rf "$work"
[ "$fails" -eq 0 ]
