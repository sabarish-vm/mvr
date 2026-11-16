# Usage scenarios

All usage examples follow the overall CLI pattern `mvr <flags> '<pattern>' '<replacement>' <files...>`
The rules on allowed regular expressions can be found in [regex](./regex.md) docs.

---

## 1. Basic rename

```
$ mvr -m -f '(.*)\.jpg' '${1}_backup.jpg' vacation.jpg
```

| Before         | After                 |
| -------------- | --------------------- |
| `vacation.jpg` | `vacation_backup.jpg` |

---

## 2. Basic copy

```
$ mvr -c -f '(.*)\.jpg' '${1}_copy.jpg' vacation.jpg
```

| Before         | After                                |
| -------------- | ------------------------------------ |
| `vacation.jpg` | `vacation.jpg` + `vacation_copy.jpg` |

- Directories cannot be copied yet.

---

## 3. Multiple files in one folder

```
$ mvr -m -f '(.*)\.jpg' '${1}_bak.jpg' a.jpg b.jpg c.jpg
```

| Before  | After       |
| ------- | ----------- |
| `a.jpg` | `a_bak.jpg` |
| `b.jpg` | `b_bak.jpg` |
| `c.jpg` | `c_bak.jpg` |

- All stay in the same directory (destination uses source's parent).

---

## 4. Full and relative paths

```
$ mvr -m -f '(.*)\.txt' '${1}_abs.txt' /some/dir/file.txt
$ mvr -m -f '(.*)\.txt' '${1}_rel.txt' file_rel.txt
```

| Before               | After                    |
| -------------------- | ------------------------ |
| `/some/dir/file.txt` | `/some/dir/file_abs.txt` |
| `file_rel.txt`       | `file_rel_rel.txt`       |

- Path type (absolute/relative) is irrelevant, only the filename is matched.
- Destination directory = source's parent directory.

---

## 5. Directory as input

```
$ mvr -m -f '(.*)' '${1}_renamed' mydir
```

| Before   | After            |
| -------- | ---------------- |
| `mydir/` | `mydir_renamed/` |

- **Rename works** — `fs::rename()` can rename directories.
- **Copy fails** — `mvr -c` on a directory prints `ECP_DIR` and skips it.

---

## 6. Nested directories

```
$ mvr -m -f '(.*)\.txt' '${1}_bak.txt' a/doc.txt a/b/note.txt
```

| Before         | After              |
| -------------- | ------------------ |
| `a/doc.txt`    | `a/doc_bak.txt`    |
| `a/b/note.txt` | `a/b/note_bak.txt` |

- Each file stays in its own parent directory.
- The regex sees only the filename (`doc.txt`, `note.txt`), not the full path.

---

## 7. Collision detection

```
$ mvr -m '(.*).*\.jpg' 'collision.jpg' a.jpg b.jpg
```

| Before           | Should happen                          |
| ---------------- | -------------------------------------- |
| `a.jpg`, `b.jpg` | Both map to `collision.jpg` → rejected |

```
Collision detected: Mapping between input to output files is not one-to-one
```

- The graph is validated before any file is touched.
- Two files mapping to the same destination → panic.
- An output that is also an input (chain) → panic.

---

## 8. No regex match (unchanged source)

```
$ mvr -m -f '(.*)\.jpg' '${1}_bak.jpg' readme.md
```

| Before      | After                   |
| ----------- | ----------------------- |
| `readme.md` | `readme.md` (unchanged) |

- File doesn't match the pattern → excluded from graph, left untouched.

---

## 9. Missing source file

```
$ mvr -m -f '(.*)\.txt' '${1}_bak.txt' nonexistent.txt
```

| Before             | After               |
| ------------------ | ------------------- |
| _(does not exist)_ | _(nothing created)_ |

- Missing file is included in the graph but the operation (`fs::rename`) fails on it.
- The error is printed; other files in the batch continue processing.

---

## 10. Log flag

```
$ mvr -m -f -l '(.*)\.txt' '${1}_logged.txt' logme.txt
```

| Before      | After                      |
| ----------- | -------------------------- |
| `logme.txt` | `logme_logged.txt`         |
| —           | `mvr_job_<timestamp>.json` |

- `-l` writes a JSON log of the operation to the current directory.
- If the log file already exists (`create_new`), the operation fails.

---

## 11. Force flag required

```
$ mvr -m '(.*)\.txt' '${1}_gone.txt' muststay.txt
```

| Before           | After                         |
| ---------------- | ----------------------------- |
| `muststay.txt`   | `muststay.txt` (unchanged)    |

- Without `-f`, mvr only prints the plan (dry run) — nothing is executed.

---

## 12. Graph hides unchanged files

```
$ mvr -m '(.*)\.jpg' '${1}_bak.jpg' match.jpg no_match.md
```

Output:

```
This is a DRY run for renaming/moving
match.jpg  -->  match_bak.jpg
```

- Only files that matched the pattern appear in the graph output.
- `no_match.md` is silently excluded.

---

## 13. No-op rename (same name)

```
$ mvr -m -f '(.*)' '$0' keep.txt
```

| Before     | After             |
| ---------- | ----------------- |
| `keep.txt` | `keep.txt` (same) |

- `$0` is the full match — source and dest are identical.
- File is renamed to itself — effectively a no-op.

---

## 14. Exit code is always 0

```
$ mvr -m -f '(.*)' '${1}_x' nope.txt        # missing source
$ mvr -f '(.*)' '${1}_x' exists.txt          # missing -m/-c flag
```

- mvr always exits 0, even on errors.
- Errors are printed to stdout, not stderr.
- The binary never exits with a non-zero code.


