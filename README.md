# Mover (mvr)

A tool inspired by `zmv` from `zsh` for linux and macOS. A cli tool that can move and also copy based on regex patterns.
Usage examples are given [here](./docs/usage.md).

## Core behavior :

The core behavior listed here is subject to minor revisions.

- `mvr` constructs a graph connecting old filenames to new filenames.
  - The plan is validated before execution. Collisions (A→C and B→C) and chains (A→B and B→C) are detected by checking the graph before any file is touched.
- Cross-filesystem operations are not supported. Source and destination must be on the same filesystem (or mount point).
- `mvr` never walks or traverses any directory tree. It operates only on the files/directories explicitly passed as arguments.
- `mvr` does not copy directories.
- Regex applies to filenames only, never full paths. That is, directory components in the full path are invisible to the engine.
- Batch operations continue on error. If one file fails, mvr prints the error and moves to the next. Similar behavior to `rsync`.
- Exit code is always 0. Even on errors. mvr never exits non-zero.
- Destinations strictly stay in the source file's directory.
- Dry run is the default.

## Roadmap:

- [x] Testing with `tempfile`
- [x] Copying files
- [ ] Copying a directory recursively
- [x] Renaming files and directories (without TOCTOU check)
- [ ] TOCTOU full rename function
- [ ] add support for windows?

## AI usage

Testing routines and docs were developed with the assistance of AI tools.
