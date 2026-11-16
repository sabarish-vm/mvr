#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MVR="${MVR:-$ROOT_DIR/target/release/mvr}"
PASS=0
FAIL=0
MVR_TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$MVR_TMP_DIR"' EXIT

# Helper functions
assert_contains() { grep -q -- "$2" <<<"$1"; }
assert_not_contains() { ! grep -q -- "$2" <<<"$1"; }
assert_exists() { [[ -e "$1" ]]; }
assert_missing() { [[ ! -e "$1" ]]; }

# Usage: check_dry_run <flags> <regex> <repl> <src> <dst>
check_dry_run() {
    local flags="$1" regex="$2" repl="$3" src="$4" dst="$5"
    local out
    out="$("$MVR" $flags "$regex" "$repl" "$src" 2>&1)"
    assert_contains "$out" "DRY run"
    assert_exists "$src"
    assert_missing "$dst"
}

# Usage: check_exec <flags> <regex> <repl> <src> <dst>
check_exec() {
    local flags="$1" regex="$2" repl="$3" src="$4" dst="$5"
    local out
    out="$("$MVR" $flags -f "$regex" "$repl" "$src" 2>&1)"
    assert_not_contains "$out" "DRY run"
    assert_exists "$dst"
    if [[ "$flags" == "-c" ]]; then
        assert_exists "$src"
        assert_exists "$dst"
    else
        assert_missing "$src"
        assert_exists "$dst"
    fi
}

run_test() {
    local func="$1"
    local temp
    temp=$(mktemp -d -p "$MVR_TMP_DIR")
    local rc=0
    (
        set -e
        cd "$temp"
        "${func}"
    ) || rc=$?
    return "$rc"
}

# Tests
test_0_exit_code_always_zero() {
    "$MVR" -m 'C' 'c' Cargo.toml >/dev/null 2>&1
}
test_rename_cycle() {
    touch vacation.jpg
    check_dry_run "-m" '(.*)\.jpg' '${1}_backup.jpg' vacation.jpg vacation_backup.jpg
    check_exec "-m" '(.*)\.jpg' '${1}_backup.jpg' vacation.jpg vacation_backup.jpg
}

test_copy_cycle() {
    touch vacation.jpg
    check_dry_run "-c" '(.*)\.jpg' '${1}_copy.jpg' vacation.jpg vacation_copy.jpg
    check_exec "-c" '(.*)\.jpg' '${1}_copy.jpg' vacation.jpg vacation_copy.jpg
}

test_multiple_files() {
    touch a.jpg b.jpg c.jpg
    "$MVR" -m -f '(.*)\.jpg' '${1}_bak.jpg' a.jpg b.jpg c.jpg >/dev/null 2>&1
    for f in a b c; do
        assert_missing "${f}.jpg"
        assert_exists "${f}_bak.jpg"
    done
}

test_full_and_relative() {
    touch file.txt file_rel.txt
    check_exec "-m" '(.*)\.txt' '${1}_abs.txt' "$PWD/file.txt" "$PWD/file_abs.txt"
    check_exec "-m" '(.*)\.txt' '${1}_rel.txt' "file_rel.txt" "file_rel_rel.txt"
}

test_directory_input() {
    mkdir mydir otherdir
    "$MVR" -m -f '(.*)' '${1}_renamed' "$PWD/mydir" >/dev/null 2>&1
    assert_missing mydir
    assert_exists mydir_renamed

    local out
    out="$("$MVR" -c -f '(.*)' '${1}_fail' "$PWD/otherdir" 2>&1)"
    assert_contains "$out" "ECP_DIR"
    assert_exists otherdir
}

test_nested_files() {
    mkdir -p a/b
    touch a/doc.txt a/b/note.txt
    "$MVR" -m -f '(.*)\.txt' '${1}_bak.txt' "$PWD/a/doc.txt" "$PWD/a/b/note.txt" >/dev/null 2>&1
    assert_missing a/doc.txt && assert_exists a/doc_bak.txt
    assert_missing a/b/note.txt && assert_exists a/b/note_bak.txt
}

test_collision() {
    touch a.jpg b.jpg
    local out rc=0
    out="$("$MVR" -m '(.*).*\.jpg' 'collision.jpg' "$PWD/a.jpg" "$PWD/b.jpg" 2>&1)" || rc=$?
    [[ "$rc" -ne 0 ]] || assert_contains "$out" "Collision detected"
}

test_no_match() {
    touch readme.md
    "$MVR" -m -f '(.*)\.jpg' '${1}_bak.jpg' "$PWD/readme.md" >/dev/null 2>&1
    assert_exists readme.md
}

test_missing_source() {
    "$MVR" -m -f '(.*)\.txt' '${1}_bak.txt' "$PWD/nonexistent.txt" >/dev/null 2>&1
    assert_missing nonexistent_bak.txt
}

test_log_flag() {
    touch logme.txt
    "$MVR" -m -f -l '(.*)\.txt' '${1}_logged.txt' "$PWD/logme.txt" >/dev/null 2>&1
    assert_exists logme_logged.txt
    compgen -G "mvr_job_*.json" >/dev/null
}

test_graph_unchanged_not_shown() {
    touch match.jpg no_match.md
    local out
    out="$("$MVR" -m '(.*)\.jpg' '${1}_bak.jpg' "$PWD/match.jpg" "$PWD/no_match.md" 2>&1)"
    assert_contains "$out" "match"
    assert_not_contains "$out" "no_match"
}

test_noop_rename() {
    touch keep.txt
    "$MVR" -m -f '(.*)' '$0' "$PWD/keep.txt" >/dev/null 2>&1
    assert_exists keep.txt
}

echo "## Running tests"
for func in $(declare -F | awk '{print $3}' | grep '^test_'); do
    if run_test "$func"; then
        echo "  PASS  $func"
        PASS=$((PASS + 1))
    else
        echo "  FAIL  $func"
        FAIL=$((FAIL + 1))
    fi
done

echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ "$FAIL" -eq 0 ]]
