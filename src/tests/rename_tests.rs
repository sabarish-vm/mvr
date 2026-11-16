#[cfg(test)]
mod tests {
    use crate::file_ops::atomic_rename;

    // Check case-only filename changes are done by renaming fxn
    #[test]
    fn case_only_rename_with_only_source() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("HELLO.txt");
        let dst = dir.path().join("hello.txt");

        std::fs::write(&src, "data").unwrap();
        assert!(atomic_rename(&src, &dst).is_ok());

        let entries: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();

        assert_eq!(entries, vec!["hello.txt"]);
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "data");
    }

    // Check if case-only changes are rejected when
    // source and destination exists as separate files with different inodes
    // Test will run only on linux else passes
    #[test]
    fn case_only_rename_rejects_different_inode() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("HELLO.txt");
        let dst = dir.path().join("hello.txt");

        std::fs::write(&src, "data").unwrap();

        // Skip on case-insensitive filesystems where two distinct files differing only by case cannot exist
        if dst.exists() {
            return;
        }

        std::fs::write(&dst, "other data").unwrap();
        assert!(atomic_rename(&src, &dst).is_err());
        assert_eq!(std::fs::read_to_string(&src).unwrap(), "data");
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "other data");
    }

    // Ensure a hardlink (same inode, different filename) cannot be renamed into
    // the source file and vice versa
    #[test]
    fn hardlink_different_name_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("original.txt");
        let dst = dir.path().join("hardlink.txt");

        std::fs::write(&src, "data").unwrap();
        std::fs::hard_link(&src, &dst).unwrap();

        assert!(atomic_rename(&src, &dst).is_err());
        assert!(atomic_rename(&dst, &src).is_err());
        assert!(src.exists());
        assert!(dst.exists());
    }

    // File A cannot be renamed into a different existing file
    #[test]
    fn rename_rejects_existing_dest() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("data.txt");
        let dst = dir.path().join("other.txt");

        std::fs::write(&src, "data").unwrap();
        std::fs::write(&dst, "occupied").unwrap();

        assert!(atomic_rename(&src, &dst).is_err());
        assert_eq!(std::fs::read_to_string(&src).unwrap(), "data");
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "occupied");
    }
}
