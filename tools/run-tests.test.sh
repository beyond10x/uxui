#!/bin/sh
# Proves tools/run-tests.sh on a crate of its own under target/run-tests-test/:
# - a test over the limit is killed and counts as failed, a failing test fails the step, an
#   ignored test is not run, and the ten slowest are printed;
# - a crate whose tests all pass ends 0 with every count in its summary line;
# - a crate with no test to run fails the step;
# - a binary with a .debug_* section fails the debug-info check, the fixture's test binary passes it.
# The limit is 2 s here (UXUI_TEST_LIMIT), so the case of a killed test takes seconds, not a minute.
# Exit 0 when every case holds.
set -u

root=$(cd "$(dirname "$0")/.." && pwd)
work=$root/target/run-tests-test
run=$root/tools/run-tests.sh
fails=0
rm -rf "$work"
mkdir -p "$work/fixture/src"

ok() { printf 'ok   %s\n' "$1"; }
bad() {
	printf 'FAIL %s\n' "$1"
	fails=$((fails + 1))
}

printf '%s\n' '[package]' 'name = "run-tests-fixture"' 'version = "0.0.0"' 'edition = "2024"' \
	'publish = false' '' '[workspace]' '' '[features]' 'bad = []' 'empty = []' \
	>"$work/fixture/Cargo.toml"
printf '%s\n' \
	'#[cfg(all(test, not(feature = "empty")))]' \
	'mod tests {' \
	'    #[test]' \
	'    fn passes() {}' \
	'    #[test]' \
	'    fn reads_its_manifest_directory() {' \
	'        assert!(std::path::Path::new("Cargo.toml").is_file());' \
	'    }' \
	'    #[test]' \
	'    #[ignore]' \
	'    fn ignored() {' \
	'        panic!("an ignored test ran");' \
	'    }' \
	'    #[cfg(feature = "bad")]' \
	'    #[test]' \
	'    fn fails() {' \
	'        panic!("planted failure");' \
	'    }' \
	'    #[cfg(feature = "bad")]' \
	'    #[test]' \
	'    fn sleeps() {' \
	'        std::thread::sleep(std::time::Duration::from_secs(30));' \
	'    }' \
	'}' >"$work/fixture/src/lib.rs"
manifest=$work/fixture/Cargo.toml

# fixture <label> <features>: runs tools/run-tests.sh on the fixture; output in $work/out.
fixture() {
	UXUI_TEST_LIMIT=2 UXUI_TEST_JOBS=4 sh "$run" "$1" --manifest-path "$manifest" --features "$2" \
		>"$work/out" 2>&1
}
has() { grep -qF -- "$1" "$work/out"; }

fixture fixture-bad bad
got=$?
if [ "$got" -eq 1 ] && has 'KILLED after 2 s: tests::sleeps' && has 'FAILED tests::fails (exit 101)' &&
	has 'run-tests: fixture-bad: 2 passed, 1 failed, 1 killed after 2 s, 1 ignored, of 4 tests in 1 binaries' &&
	has 'run-tests: fixture-bad: the ten slowest of 4 tests' && ! has 'an ignored test ran'; then
	ok "a test over the limit is killed, a failing test fails, an ignored test is not run"
else
	bad "the crate with a failing and a sleeping test: exit $got"
	cat "$work/out"
fi
slowest=$(sed -n '/the ten slowest/{n;p;}' "$work/out")
case $slowest in
*'s  tests::sleeps') ok "the killed test heads the ten slowest ($slowest)" ;;
*)
	bad "the killed test does not head the ten slowest: '$slowest'"
	cat "$work/out"
	;;
esac

fixture fixture-good ''
got=$?
if [ "$got" -eq 0 ] &&
	has 'run-tests: fixture-good: 2 passed, 0 failed, 0 killed after 2 s, 1 ignored, of 2 tests in 1 binaries'; then
	ok "a crate whose tests pass ends 0 ($(tail -1 "$work/out"))"
else
	bad "the crate whose tests pass: exit $got"
	cat "$work/out"
fi

fixture fixture-empty empty
got=$?
if [ "$got" -eq 1 ] && has 'run-tests: fixture-empty: no test to run'; then
	ok "a crate with no test to run fails"
else
	bad "the crate with no test: exit $got"
	cat "$work/out"
fi

# A binary built with debug info, by rustc alone: no cargo profile and no compile cache.
printf 'fn main() {}\n' >"$work/main.rs"
if rustc -g -o "$work/with-debug" "$work/main.rs" >"$work/out" 2>&1; then
	sh "$run" --check-debug "$work/with-debug" >"$work/out" 2>&1
	got=$?
	if [ "$got" -eq 1 ] && has 'carries debug info: .debug_'; then
		ok "a binary with debug info fails the check ($(head -1 "$work/out" | sed 's|.*/||'))"
	else
		bad "a binary built with -g: exit $got"
		cat "$work/out"
	fi
else
	bad "rustc -g did not build the binary with debug info"
	cat "$work/out"
fi
exe=$(sed -n 's/.*"executable":"\([^"]*\)".*/\1/p' "$root/target/run-tests/fixture-good/build.json" | head -1)
if [ -n "$exe" ] && sh "$run" --check-debug "$exe" >"$work/out" 2>&1; then
	ok "the fixture's test binary, built with the repository's profile, has no debug info"
else
	bad "the fixture's test binary '$exe' fails the debug-info check"
	cat "$work/out"
fi

rm -rf "$work" "$root/target/run-tests/fixture-bad" "$root/target/run-tests/fixture-good" \
	"$root/target/run-tests/fixture-empty"
[ "$fails" -eq 0 ]
