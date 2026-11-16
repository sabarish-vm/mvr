The regex renaming engine of `mvr` uses the [`regex`](https://docs.rs/regex/latest/regex/) crate.
The pattern is applied to the **filename** only (not the full path).

---

## Escaping special characters in the pattern

- `\.` → literal dot (`.` in regex = any character)
- `\?` → literal question mark
- `\(` → literal parentheses
- `\d` → digit (0-9)
- `\w` → word character (letter, digit, underscore)
- `\s` → whitespace
- `\\` → literal backslash

## Replacement syntax

mvr passes the replacement string directly to Rust's regex engine (`replace_all`).

| Expression    | Captures group                                        |
| ------------- | ----------------------------------------------------- |
| `$0`          | the entire match                                      |
| `$1`          | group `1` (only if followed by a non-identifier char) |
| `${1}`        | group `1` (explicit boundary, prefer this)            |
| `$1_backup`   | group `1_backup` (likely empty) **✗**                 |
| `${1}_backup` | group `1` + `_backup` **✓**                           |

- Identifier chars are `[a-zA-Z0-9_]`.
- Always use `${1}` over `$1` when the replacement text starts with a letter, digit, or underscore.
- Always wrap patterns in single quotes (`'...'`) in your shell so Bash/Zsh does not expand `$1` or `${1}` before `mvr` sees it.

### Example

```
$ mvr '(.*)toml' '$1txt' Cargo.toml -m

This is a DRY run for renaming/moving
Cargo.toml  -->
```

Missing group `$1txt`. The ambiguity can be resolved by adding braces, i.e.,

```
$ mvr '(.*)toml' '${1}txt' Cargo.toml -m

This is a DRY run for renaming/moving
Cargo.toml  -->  Cargo.txt
```

---

## Capture groups

| Pattern     | What is captured        | Replacement syntax      |
| ----------- | ----------------------- | ----------------------- |
| `(.*)`      | Everything              | `${1}` or `${1}_suffix` |
| `(.+)`      | Everything (1+ chars)   | `${1}` or `${1}_suffix` |
| `(.+)\.txt` | Filename without `.txt` | `${1}_bak.txt`          |
| `(.*)-(.*)` | Two groups split by `-` | `${1}_${2}`             |

---

## Common patterns

### Replace extension

```
Pattern:     (.+)\.jpg
Replacement: ${1}_backup.jpg

vacation.jpg    →  vacation_backup.jpg
sunset.jpg      →  sunset_backup.jpg
```

### Insert prefix

```
Pattern:     (.+)
Replacement: old_${1}

readme.txt  →  old_readme.txt
notes.md    →  old_notes.md
```

### Insert suffix

```
Pattern:     (.+)\.txt
Replacement: ${1}_2024.txt

report.txt  →  report_2024.txt
```

### Swap name parts (two groups)

```
Pattern:     ([^-]+)-(.+)
Replacement: ${2}_${1}

photo-vacation.jpg  →  vacation_photo.jpg
```
