#![cfg(test)]
use crate::graph::HasCollisionCheck;
use crate::graph::UniqueLinks;
use crate::structs::{Color, Opts};
use proptest::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
#[should_panic(expected = "Collision detected")]
fn test_collision_detection() {
    // Setup: Two files mapping to the same output
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("b.txt");
    fs::write(&file1, "").unwrap();
    fs::write(&file2, "").unwrap();

    let input_files = vec![file1, file2];
    let unique_links = UniqueLinks::new(
        &input_files,
        r".*",     // Match all .txt files
        "out.txt", // Both map to "out.txt" → collision
        false,
    )
    .unwrap();
    unique_links.collision_check();
}

#[test]
#[should_panic(expected = "Chain detected")] // Expect panic on chain
fn test_3_chain_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("b.txt");
    let file3 = temp_dir.path().join("c.txt");
    std::fs::write(&file1, "").unwrap();
    std::fs::write(&file2, "").unwrap();
    std::fs::write(&file3, "").unwrap();

    let input_files = vec![file1, file2, file3];
    // This generated graph is irrelevant, its fields will be overwritten below
    let unique_links = UniqueLinks::new(&input_files, r"(.*)", r"${1}", false).unwrap();

    let unique_links = UniqueLinks {
        sources: vec![0, 1, 2],      // file1 (id=0), file2 (id=1), file3 (id=2)
        destinations: vec![1, 2, 0], // file1 → file2, file2 → file3, file3->file1
        link_map: HashMap::from([(0, 1), (1, 2), (2, 0)]),
        ..unique_links
    };

    unique_links.collision_check(); // Should panic
}
#[test]
#[should_panic(expected = "Chain detected")]
fn test_2_chain_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("b.txt");
    let file3 = temp_dir.path().join("c.txt");
    std::fs::write(&file1, "").unwrap();
    std::fs::write(&file2, "").unwrap();
    std::fs::write(&file3, "").unwrap();

    let input_files = vec![file1, file2, file3];
    // This generated graph is irrelevant, its fields will be overwritten below
    let unique_links = UniqueLinks::new(&input_files, r"(.*)", r"${1}", false).unwrap();

    let unique_links = UniqueLinks {
        sources: vec![0, 1],      // file1 (id=0), file2 (id=1)
        destinations: vec![1, 0], // file1 → file2, file2 → file1
        link_map: HashMap::from([(0, 1), (1, 0)]),
        ..unique_links
    };

    unique_links.collision_check();
}

#[test]
fn test_new_valid_regex_mapping() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("b.txt");
    fs::write(&file1, "").unwrap();
    fs::write(&file2, "").unwrap();

    let input_files = vec![file1, file2];
    let unique_links = UniqueLinks::new(&input_files, r"(a|b)\.txt", "new_${0}", false).unwrap();

    assert_eq!(unique_links.input_file_count, 2);
    assert_eq!(unique_links.processing_file_count, 2);
    assert!(unique_links.unchanged_sources.is_empty());
    unique_links.collision_check();
}

#[test]
fn test_mix_changed_unchanged_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("b.txt");
    fs::write(&file1, "").unwrap();
    fs::write(&file2, "").unwrap();

    let input_files = vec![file1, file2];
    let unique_links = UniqueLinks::new(&input_files, r"a.txt", "2.txt", false).unwrap();
    assert_eq!(unique_links.input_file_count, 2);
    assert_eq!(unique_links.processing_file_count, 1);
    assert_eq!(unique_links.unchanged_sources.len(), 1);
}
#[test]
fn test_unchanged_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("b.txt");
    fs::write(&file1, "").unwrap();
    fs::write(&file2, "").unwrap();

    let input_files = vec![file1, file2];
    let unique_links = UniqueLinks::new(&input_files, "a.txt", "a.txt", false).unwrap();
    assert_eq!(unique_links.input_file_count, 2);
    assert_eq!(unique_links.processing_file_count, 0);
    assert_eq!(unique_links.unchanged_sources.len(), 2);
}

#[test]
fn test_missing_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("exists.txt");
    let file2 = temp_dir.path().join("missing.txt");
    fs::write(&file1, "").unwrap(); // create exists.txt

    let input_files = vec![file1, file2];
    let unique_links = UniqueLinks::new(&input_files, r".*\.txt", "out.txt", false).unwrap();

    assert_eq!(unique_links.input_file_count, 2);
    assert_eq!(unique_links.processing_file_count, 1);
    assert_eq!(unique_links.missing_sources.len(), 1);
}

#[test]
// check if only the file name at the end of path is renamed
fn test_path_regex_transformation() {
    let temp_base_dir = tempfile::tempdir().unwrap();
    let build_dir = temp_base_dir.path().join("build.txt");
    let _ = std::fs::create_dir(&build_dir);
    let file1 = build_dir.clone().join("a.txt");
    let file2 = temp_base_dir.path().join("b.txt");
    let file3 = temp_base_dir.path().join("c.txt");
    let files = vec![file1, file2, file3];
    let graph = UniqueLinks::new(&files, r"(.*)\.txt", "$1.bak", false).unwrap();

    let sour_p1 = graph.id_to_path.get(&graph.sources[0]).unwrap();
    let dest_p1 = graph.id_to_path.get(&graph.destinations[0]).unwrap();

    assert_eq!(dest_p1, &Path::new("build.txt/a.bak"));
    assert_eq!(
        Path::new(dest_p1).parent(),
        Path::new(sour_p1).parent(),
        "The parent directory names were changed, unwanted behaviour"
    );
}

proptest! {
    #[test]
    fn prop_collision_check(
        files in prop::collection::vec("[a-z0-9]{1,8}\\.txt", 1..2),
        dest in "[a-z0-9]{1,8}\\.bak"
    ) {
        let opts = Opts {
            source_pattern: r"(.*)\.txt".to_string(),
            dest_pattern: dest,
            files: files.into_iter().map(PathBuf::from).collect(),
            copy_bool:true,
            move_bool:false,
            force_run: false,
            log_bool: false,
        };
        let graph = UniqueLinks::new(&opts.files,&opts.source_pattern,&opts.dest_pattern,false).unwrap();
        graph.collision_check();
    }
}

#[test]
fn test_copy_execution() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("b.txt");
    fs::write(&file1, "hello").unwrap();
    fs::write(&file2, "world").unwrap();

    let input_files = vec![file1, file2];
    let unique_links = UniqueLinks::new(&input_files, r"(a|b)\.txt", "new_${0}", false).unwrap();
    unique_links.print_graph(true);
    let status = unique_links.copy();

    assert!(temp_dir.path().join("a.txt").exists());
    assert!(temp_dir.path().join("b.txt").exists());
    assert!(temp_dir.path().join("new_a.txt").exists());
    assert!(temp_dir.path().join("new_b.txt").exists());
    assert_eq!(status.files.len(), 2);
}

#[test]
fn test_rename_execution() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("b.txt");
    fs::write(&file1, "hello").unwrap();
    fs::write(&file2, "world").unwrap();

    let input_files = vec![file1, file2];
    let unique_links = UniqueLinks::new(&input_files, r"(a|b)\.txt", "new_${0}", false).unwrap();
    let status = unique_links.rename();

    assert!(!temp_dir.path().join("a.txt").exists());
    assert!(!temp_dir.path().join("b.txt").exists());
    assert!(temp_dir.path().join("new_a.txt").exists());
    assert!(temp_dir.path().join("new_b.txt").exists());
    assert_eq!(status.files.len(), 2);
}

#[test]
fn test_rename_destination_exists() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("new_a.txt"); // Pre-create destination
    fs::write(&file1, "hello").unwrap();
    fs::write(&file2, "blocking").unwrap();

    let input_files = vec![file1];
    let unique_links = UniqueLinks::new(&input_files, r"a\.txt", "new_$0", false).unwrap();

    let status = unique_links.rename();

    assert!(temp_dir.path().join("a.txt").exists());
    assert_eq!(status.status.len(), 1);
    assert!(status.status[0].0.contains("ERN001"));
}

#[test]
fn test_rename_destination_is_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let dest_dir = temp_dir.path().join("new_a.txt");
    fs::write(&file1, "hello").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    let input_files = vec![file1];
    let unique_links = UniqueLinks::new(&input_files, r"a\.txt", "new_$0", false).unwrap();

    let status = unique_links.rename();
    assert!(temp_dir.path().join("a.txt").exists());
    assert_eq!(status.status.len(), 1);
    assert!(status.status[0].0.contains("ERN001"));
}

#[test]
fn test_get_err_code() {
    // Test with a known error code
    let err = anyhow::anyhow!("ECP005: Source and destination are on different devices");
    let code = UniqueLinks::get_err_code(&err);
    assert_eq!(code.0, "ECP005");

    // Test with no colon (fallback to "UNKNOWN")
    let err = anyhow::anyhow!("Some generic error");
    let code = UniqueLinks::get_err_code(&err);
    assert_eq!(code.0, "UNKNOWN");
}

#[test]
fn test_highlight_path_diff() {
    // Test deletion (old: "a.txt", new: "b.txt" → "a" is deleted)
    let result = UniqueLinks::highlight_path_diff("a.txt", "b.txt", Color::Red);
    assert!(result == "\x1b[31ma\x1b[0m.txt");

    // Test no difference (identical strings)
    let result = UniqueLinks::highlight_path_diff("same.txt", "same.txt", Color::Red);
    assert!(result == "same.txt");
}

#[test]
fn test_copy_destination_exists() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    let file2 = temp_dir.path().join("new_a.txt");
    fs::write(&file1, "hello").unwrap();
    fs::write(&file2, "blocking").unwrap();

    let input_files = vec![file1];
    let unique_links = UniqueLinks::new(&input_files, r"a\.txt", "new_$0", false).unwrap();

    let status = unique_links.copy();

    assert!(temp_dir.path().join("a.txt").exists());
    assert_eq!(status.status.len(), 1);
    assert!(status.status[0].0.contains("ECP006"));
}

#[test]
fn test_no_matching_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    fs::write(&file1, "").unwrap();

    let input_files = vec![file1];
    let unique_links =
        UniqueLinks::new(&input_files, r"nonexistent\.txt", "new_$0.txt", true).unwrap();

    assert_eq!(unique_links.input_file_count, 1);
    assert_eq!(unique_links.processing_file_count, 0);
    assert!(unique_links.sources.is_empty());
    assert!(unique_links.destinations.is_empty());
}

#[test]
fn test_regex_special_chars_in_filenames() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("file.$$.txt");
    let file2 = temp_dir.path().join("data[1].log");
    fs::write(&file1, "").unwrap();
    fs::write(&file2, "").unwrap();

    let input_files = vec![file1, file2];
    let unique_links = UniqueLinks::new(
        &input_files,
        r"file\.\$\$\.txt|data\[1\]\.log",
        "new_$0",
        true,
    )
    .unwrap();

    assert_eq!(unique_links.processing_file_count, 2);
    unique_links.collision_check();
}

#[test]
fn test_copy_parent_dir_missing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    fs::write(&file1, "hello").unwrap();

    let input_files = vec![file1];
    let unique_links = UniqueLinks::new(&input_files, r".*", "nonexistent/b.txt", false).unwrap();

    let status = unique_links.copy();

    assert!(temp_dir.path().join("a.txt").exists());
    assert_eq!(status.status.len(), 1);
    assert!(status.status[0].0.contains("ECP004"));
}
#[test]
fn test_rename_parent_dir_missing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("a.txt");
    fs::write(&file1, "hello").unwrap();

    let input_files = vec![file1];
    let unique_links = UniqueLinks::new(&input_files, r".*", "nonexistent/b.txt", false).unwrap();

    let status = unique_links.rename();

    assert!(temp_dir.path().join("a.txt").exists());
    assert_eq!(status.status.len(), 1);
    assert!(status.status[0].0.contains("ERN003"));
}
#[test]
fn test_copy_with_symlink() {
    let temp_dir = tempfile::tempdir().unwrap();
    let target = temp_dir.path().join("target.txt");
    let link = temp_dir.path().join("link.txt");
    fs::write(&target, "original").unwrap();

    // Create symlink (Unix only)
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let input_files = vec![link];
        let unique_links =
            UniqueLinks::new(&input_files, r"link\.txt", "new_link.txt", false).unwrap();

        let status = unique_links.copy();
        assert_eq!(status.files.len(), 1);
        assert!(temp_dir.path().join("new_link.txt").exists());
    }
}

#[test]
fn test_rename_with_symlink() {
    let temp_dir = tempfile::tempdir().unwrap();
    let target = temp_dir.path().join("target.txt");
    let link = temp_dir.path().join("link.txt");
    fs::write(&target, "original").unwrap();

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let input_files = vec![link.clone()];
        let unique_links =
            UniqueLinks::new(&input_files, r"link\.txt", "new_link.txt", false).unwrap();

        let status = unique_links.rename();
        assert_eq!(status.files.len(), 1);
        assert!(!link.exists());
        assert!(temp_dir.path().join("new_link.txt").exists());
    }
}

#[test]
fn test_no_ops() {
    let input_files: Vec<PathBuf> = vec![];
    let unique_links = UniqueLinks::new(&input_files, r".*", "new_$0", true).unwrap();

    assert_eq!(unique_links.input_file_count, 0);
    assert_eq!(unique_links.processing_file_count, 0);
    assert!(unique_links.sources.is_empty());
    assert!(unique_links.destinations.is_empty());
}

#[test]
fn test_rename_with_spaces() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("my file.txt");
    let file2 = temp_dir.path().join("another file.log");
    fs::write(&file1, "").unwrap();
    fs::write(&file2, "").unwrap();

    let input_files = vec![file1.clone(), file2.clone()];
    let unique_links = UniqueLinks::new(&input_files, r".* file\..*", "new_$0", false).unwrap();

    let status = unique_links.rename();

    assert!(!file1.exists());
    assert!(!file2.exists());
    assert!(temp_dir.path().join("new_my file.txt").exists());
    assert!(temp_dir.path().join("new_another file.log").exists());
    assert_eq!(status.files.len(), 2);
}

#[test]
fn test_copy_preserves_permissions() {
    let temp_dir = tempfile::tempdir().unwrap();
    let base_path = temp_dir.path();
    let file1 = base_path.join("a.txt");
    fs::write(&file1, "hello").unwrap();

    // Set permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&file1, fs::Permissions::from_mode(0o644)).unwrap();
    }

    let input_files = vec![file1.clone()];
    let unique_links = UniqueLinks::new(&input_files, r"a\.txt", "b.txt", false).unwrap();
    let _ = unique_links.copy();

    let dest_path = base_path.join("b.txt");

    // Verify permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let expected_perms = fs::Permissions::from_mode(0o100644);
        let dest_perms = fs::metadata(&dest_path).unwrap().permissions();
        assert_eq!(dest_perms, expected_perms); // Unix: full mode check
    }
}

#[test]
fn test_copy_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file1 = temp_dir.path().join("dir_a");
    let file2 = temp_dir.path().join("a.txt");
    let _ = fs::create_dir(&file1);
    fs::write(&file2, "world").unwrap();

    let input_files = vec![file1, file2];
    let unique_links = UniqueLinks::new(
        &input_files,
        "a",
        "b", // dir_a → dir_b,  a.txt → b.txt
        false,
    )
    .unwrap();
    unique_links.print_graph(true);
    let status = unique_links.copy();
    println!("{:?}", status.status);

    let ecp_bool = status
        .files
        .iter()
        .zip(status.status)
        .filter_map(|((n1, _), s1)| {
            if n1.contains("dir_a") {
                Some(s1.0.contains("ECP_DIR"))
            } else {
                None
            }
        })
        .all(|x| x);
    assert!(temp_dir.path().join("a.txt").exists());
    assert!(temp_dir.path().join("b.txt").exists());
    assert!(temp_dir.path().join("dir_a").exists());
    assert!(!temp_dir.path().join("dir_b").exists());
    assert!(ecp_bool);
}
