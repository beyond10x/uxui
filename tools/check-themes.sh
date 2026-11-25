#!/bin/sh
# Checks the themes under themes/. Comments are stripped and each stylesheet is read as its rules, so a
# check holds only for a declaration that a browser applies:
# - every stylesheet has a light and a dark mode: a top-level `:root` rule with `color-scheme: light`,
#   and the dark mode in `@media (prefers-color-scheme: dark)` on `:root:not([data-uxui-mode="light"])`
#   and on `:root[data-uxui-mode="dark"]`; both dark rules set `color-scheme: dark` and every property
#   of the light rule;
# - every `var(--x)` without a fallback names a property set by a top-level rule of the same stylesheet
#   or of the one it is layered over (slot-jobs.css over dense.css);
# - themes/README.md names every file under themes/, and every fixture matches its sha256 in
#   themes/fixtures/README.md;
# - dense.css declares the class hooks' layout: fixed column widths, one-line cells, a label beside
#   its value, a colour mark and a badge fill per value, no hover on the empty row;
# - slot-jobs.css gives every column of slot's job list a `width` (not `max-width`), and a rule to
#   every value of every tone map a column or field of slot's document is toned by.
# Usage: sh tools/check-themes.sh [root] [slot-ui-document]. The document defaults to
# themes/fixtures/slot.ui.yaml. Exit 0 when every check holds, 1 otherwise.
set -u

root=${1:-$(cd "$(dirname "$0")/.." && pwd)}
themes=$root/themes
doc=${2:-$themes/fixtures/slot.ui.yaml}
work=$root/target/check-themes
rm -rf "$work"
mkdir -p "$work" || exit 1
trap 'rm -rf "$work"' EXIT
fails=0

fail() {
	printf 'FAIL %s\n' "$*"
	fails=$((fails + 1))
}

# rules <css>: one line per style rule, `<at-rule preludes>\t<selectors>\t<declarations>`, comments
# stripped, white space collapsed; the first field is empty for a top-level rule.
rules() {
	tr '\n' '\001' <"$1" | sed -E 's:/\*([^*]|\*+[^*/])*\*+/::g' | tr '\001' '\n' | awk '
		function norm(s) { gsub(/[ \t\n]+/, " ", s); sub(/^ /, "", s); sub(/ $/, "", s); return s }
		{ buf = buf $0 "\n" }
		END {
			n = length(buf); d = 0; tok = ""
			for (i = 1; i <= n; i++) {
				c = substr(buf, i, 1)
				if (c == "{") {
					d++; pre = norm(tok); tok = ""
					isat[d] = substr(pre, 1, 1) == "@"; head[d] = pre
				} else if (c == "}") {
					if (d > 0 && !isat[d]) {
						ctx = ""
						for (k = 1; k < d; k++) if (isat[k]) ctx = ctx (ctx == "" ? "" : " ") head[k]
						printf "%s\t%s\t%s\n", ctx, head[d], norm(tok)
					}
					tok = ""; if (d > 0) d--
				} else tok = tok c
			}
		}'
}

# decl <rules> <context> <selector> <property> [value]: a rule in that context lists that selector and
# declares that property, its value containing [value] when given.
decl() {
	awk -F '\t' -v ctx="$2" -v sel="$3" -v prop="$4" -v val="${5-}" '
		$1 == ctx {
			ns = split($2, ss, ","); hit = 0
			for (i = 1; i <= ns; i++) { s = ss[i]; gsub(/^ +| +$/, "", s); if (s == sel) hit = 1 }
			if (!hit) next
			nd = split($3, ds, ";")
			for (j = 1; j <= nd; j++) {
				k = index(ds[j], ":"); if (!k) continue
				p = substr(ds[j], 1, k - 1); v = substr(ds[j], k + 1)
				gsub(/^ +| +$/, "", p); gsub(/^ +| +$/, "", v)
				if (p == prop && (val == "" || index(v, val))) found = 1
			}
		}
		END { exit !found }' "$1"
}

need() {
	file=$1
	shift
	what=$1
	shift
	decl "$work/$file.rules" "$@" || fail "themes/$file: no '$2 { $3: ${4-…} }'${1:+ in $1} ($what)"
}

# props <rules> <context> <selector>: the properties the rules of that context and selector declare.
props() {
	awk -F '\t' -v ctx="$2" -v sel="$3" '
		$1 == ctx && $2 == sel {
			nd = split($3, ds, ";")
			for (j = 1; j <= nd; j++) { k = index(ds[j], ":"); if (!k) continue; p = substr(ds[j], 1, k - 1); gsub(/^ +| +$/, "", p); print p }
		}' "$1" | sort -u
}

base_of() {
	case $1 in
	slot-jobs.css) echo dense.css ;;
	esac
}

[ -d "$themes" ] || {
	fail "themes/: missing"
	exit 1
}

for css in $(cd "$themes" && find . -name '*.css' | sed 's|^\./||' | sort); do
	mkdir -p "$work/$(dirname "$css")"
	rules "$themes/$css" >"$work/$css.rules"
done

dark_media='@media (prefers-color-scheme: dark)'
for css in $(cd "$themes" && find . -name '*.css' | sed 's|^\./||' | sort); do
	r=$work/$css.rules
	if ! decl "$r" '' ':root' color-scheme light; then
		fail "themes/$css: no top-level ':root { color-scheme: light }' rule (the light mode)"
		continue
	fi
	light=$(awk -F '\t' '$1 == "" && $2 == ":root" && $3 ~ /(^|; ?)color-scheme: light/' "$r" |
		awk -F '\t' '{ nd = split($3, ds, ";"); for (j = 1; j <= nd; j++) { k = index(ds[j], ":"); if (!k) continue; p = substr(ds[j], 1, k - 1); gsub(/^ +| +$/, "", p); print p } }' | sort -u)
	need "$css" 'dark mode from the system setting' "$dark_media" ':root:not([data-uxui-mode="light"])' color-scheme dark
	need "$css" 'dark mode a person picked' '' ':root[data-uxui-mode="dark"]' color-scheme dark
	media_props=$(props "$r" "$dark_media" ':root:not([data-uxui-mode="light"])')
	picked_props=$(props "$r" '' ':root[data-uxui-mode="dark"]')
	for p in $light; do
		printf '%s\n' "$media_props" | grep -qxF -- "$p" ||
			fail "themes/$css: '$p' of the light mode is not set in the system's dark mode"
		printf '%s\n' "$picked_props" | grep -qxF -- "$p" ||
			fail "themes/$css: '$p' of the light mode is not set in the picked dark mode"
	done

	defined=$(for f in "$css" $(base_of "$(basename "$css")"); do
		[ -f "$work/$f.rules" ] && awk -F '\t' '$1 == "" { print $3 }' "$work/$f.rules"
	done | tr ';' '\n' | sed -n 's/^ *\(--[A-Za-z0-9_-]*\) *:.*/\1/p' | sort -u)
	for v in $(cut -f 3 "$r" | grep -oE 'var\(--[A-Za-z0-9_-]+\)' | sed 's/^var(//; s/)$//' | sort -u); do
		printf '%s\n' "$defined" | grep -qxF -- "$v" ||
			fail "themes/$css: var($v) without a fallback, and no top-level rule sets $v"
	done
done

readme=$themes/README.md
if [ -f "$readme" ]; then
	for f in $(cd "$themes" && find . -type f ! -path ./README.md | sed 's|^\./||' | sort); do
		grep -qF -- "\`$f\`" "$readme" || fail "themes/README.md: does not name \`$f\`"
	done
else
	fail "themes/README.md: missing"
fi

fixtures=$themes/fixtures
if [ -d "$fixtures" ]; then
	for f in $(cd "$fixtures" && find . -type f ! -path ./README.md | sed 's|^\./||' | sort); do
		want=$(grep -F "| \`$f\` |" "$fixtures/README.md" 2>/dev/null | grep -oE '[0-9a-f]{64}' | head -1)
		got=$(sha256sum "$fixtures/$f" | cut -d ' ' -f 1)
		[ -n "$want" ] && [ "$want" = "$got" ] ||
			fail "themes/fixtures/$f: sha256 $got, themes/fixtures/README.md records '${want:-none}'"
	done
fi

if [ -f "$work/dense.css.rules" ]; then
	need dense.css 'column widths hold' '' 'table.uxui-dense' table-layout fixed
	need dense.css "slot's reach into the page margin" '' 'table.uxui-dense' width '--uxui-dense-reach'
	for p in 'white-space nowrap' 'overflow hidden' 'text-overflow ellipsis'; do
		need dense.css 'one-line cells' '' 'table.uxui-dense td' $p
	done
	need dense.css 'a label beside its value' '' 'dl.uxui-dense' display grid
	need dense.css 'a label beside its value' '' 'dl.uxui-dense' grid-template-columns
	need dense.css 'a colour mark per value' '' '[data-uxui-value]::before' content 'var(--uxui-mark, none)'
	need dense.css 'a colour mark per value' '' '[data-uxui-value]::before' background '--uxui-value-color'
	need dense.css 'a badge fill per value' '' '.uxui-badge' background '--uxui-value-fill'
	if cut -f 2 "$work/dense.css.rules" | grep -qE '(^|[ ,])tr:hover'; then
		fail "themes/dense.css: a row hover that lights the empty row; exclude it with tr:not(.uxui-empty):hover"
	fi
	need dense.css 'no hover on the empty row' '' 'table.uxui-dense tbody tr:not(.uxui-empty):hover td' background
else
	fail "themes/dense.css: missing"
fi

if [ -f "$work/slot-jobs.css.rules" ]; then
	need slot-jobs.css "slot's 6 px reach into the page margin" '' 'table.uxui-dense' --uxui-dense-reach 6px
	need slot-jobs.css 'the colour mark of a toolchain' '' '.uxui-field-toolchain[data-uxui-value]' --uxui-mark
	for v in rust go; do
		need slot-jobs.css 'a colour per toolchain' '' ".uxui-field-toolchain[data-uxui-value=\"$v\"]" --uxui-value-color
	done
	if [ -f "$doc" ]; then
		fields=$(sed -n '/^  jobs\.list:/,/^  jobs\.detail:/p' "$doc" | grep -o 'field: [a-z_0-9]*' | sed 's/field: //' | sort -u)
		[ -n "$fields" ] || fail "$doc: no column of jobs.list found"
		for f in $fields; do
			need slot-jobs.css "a width for column '$f' of slot's job list" '' "table.uxui-dense .uxui-field-$f" width
		done
		pairs=$(grep -oE 'field: [a-z_0-9]+[^}]*tone_by: [a-z_0-9]+' "$doc" | sed -E 's/^field: ([a-z_0-9]+).*tone_by: ([a-z_0-9]+)$/\1 \2/' | sort -u)
		[ -n "$pairs" ] || fail "$doc: no field toned by a tone map"
		count=0
		printf '%s\n' "$pairs" | while read -r f map; do
			values=$(sed -n "s/^  $map: {\(.*\)}\$/\1/p" "$doc" | tr ',' '\n' | sed 's/:.*//; s/ //g')
			[ -n "$values" ] || echo "FAIL $doc: tone map '$map' of field '$f' not found"
			for v in $values; do
				awk -F '\t' -v sel=".uxui-field-$f[data-uxui-value=\"$v\"]" '
					$1 == "" && $3 != "" { ns = split($2, ss, ","); for (i = 1; i <= ns; i++) { s = ss[i]; gsub(/^ +| +$/, "", s); if (s == sel) found = 1 } }
					END { exit !found }' "$work/slot-jobs.css.rules" ||
					echo "FAIL themes/slot-jobs.css: no rule for '.uxui-field-$f[data-uxui-value=\"$v\"]' (tone map '$map')"
			done
		done >"$work/values.out"
		if [ -s "$work/values.out" ]; then
			cat "$work/values.out"
			fails=$((fails + $(wc -l <"$work/values.out")))
		fi
		count=$(printf '%s\n' "$pairs" | wc -l)
		printf 'slot document %s: %s columns, %s toned fields\n' "${doc#"$root"/}" "$(printf '%s\n' "$fields" | wc -l)" "$count"
	else
		fail "$doc: missing (the slot document the widths and value colours are checked against)"
	fi
else
	fail "themes/slot-jobs.css: missing"
fi

if [ "$fails" -gt 0 ]; then
	printf 'check-themes: %s failed\n' "$fails"
	exit 1
fi
printf 'check-themes: every check passed\n'
