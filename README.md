# Mover (mvr)

A tool inspired by `zmv` from `zsh` for linux and macOS. A cli tool that can move and also copy based on regex patterns.

# Core principles (subject to change):

- mvr constructs a graph connecting old filenames to new filenames.
- The plan is validated before execution. Collisions (A→C and B→C) and chains (A→B and B→C) are detected by checking the graph before any file is touched.
- mvr never walks or traverses any directory tree. It operates only on the files/directories you explicitly pass as arguments.
- mvr does not copy directories.
- Regex applies to filenames only, never full paths. That is, directory components in the full path are invisible to the engine.
- Batch operations continue on error. If one file fails, mvr prints the error and moves to the next. Similar behavior to `rsync`.
- Exit code is always 0. Even on errors. mvr never exits non-zero.
- Destinations strictly stay in the source file's directory.
- Dry run is the default.

# Features and TODO:

- [x] Testing with `tempfile`
- [x] Copying files
- [ ] Copying a directory recursively
- [x] Renaming files and directories (without TOCTOU check)
- [ ] TOCTOU full rename function
- [ ] add support for windows?
