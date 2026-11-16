use anyhow::Context;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

fn is_case_only_rename(src: &Path, dst: &Path) -> bool {
    match (src.parent(), dst.parent(), src.file_name(), dst.file_name()) {
        (Some(p1), Some(p2), Some(n1), Some(n2)) => {
            p1 == p2 && n1 != n2 && n1.eq_ignore_ascii_case(n2)
        }
        _ => false,
    }
}
fn is_same_file(src: &Path, dst: &Path) -> anyhow::Result<bool> {
    let source_metadata = fs::symlink_metadata(src)
        .with_context(|| anyhow::anyhow!("ECP001 for the file : {:?}", src))?;
    let dest_metadata = fs::symlink_metadata(dst)
        .with_context(|| anyhow::anyhow!("ECP001 for the file : {:?}", dst))?;
    Ok(
        source_metadata.dev() == dest_metadata.dev()
            && source_metadata.ino() == dest_metadata.ino(),
    )
}

pub(crate) fn atomic_copy(source_path: &Path, dest_path: &Path) -> Result<(), anyhow::Error> {
    if source_path.is_dir() {
        anyhow::bail!(
            "ECP_DIR : Source {:?} is a directory and copying directories is not yet handled",
            source_path,
        );
    }

    let mut source_file = fs::File::open(source_path)?;

    let source_metadata = fs::metadata(source_path)
        .with_context(|| anyhow::anyhow!("ECP001 for the file : {:?}", source_path))?;

    let abs_dest_path = std::path::absolute(dest_path)
        .with_context(|| anyhow::anyhow!("ECP002 for the file : {:?}", source_path))?;

    let parent_path = abs_dest_path
        .parent()
        .with_context(|| format!("ECP003 : No parent found for the file : {:?}", dest_path))?;

    let dest_dir_metadata = fs::metadata(parent_path).with_context(|| {
        anyhow::anyhow!(
            "ECP004 Metadata for parent {:?} denied for the file {:?}",
            parent_path,
            source_path
        )
    })?;

    if source_metadata.dev() != dest_dir_metadata.dev() {
        anyhow::bail!(
            "ECP005 Source and destination are on different devices. Aborting for safety."
        );
    }

    // 2. Create the destination file atomically
    let mut dest_file = match fs::File::create_new(dest_path) {
        Ok(file) => file,
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => {
                anyhow::bail!(
                    "ECP006 TOCTOU Blocked: Target file {:?} already exists or was created mid-copy-operation.",
                    dest_path,
                );
            }
            _ => {
                anyhow::bail!("ECP007 : Some error during FileCopy: {}", err);
            }
        },
    };

    // 3. Copy data
    io::copy(&mut source_file, &mut dest_file)?;

    // 4. Sync to disk
    dest_file.sync_all()?;

    // 5. Copy permissions from source to destination
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Extract the raw full Unix mode bits (e.g., 0o755, 0o644)
        let source_mode = source_metadata.permissions().mode();

        // Construct and force assign the absolute mode to the target path
        let target_permissions = fs::Permissions::from_mode(source_mode);
        fs::set_permissions(dest_path, target_permissions)?;
    }
    Ok(())
}

// This is not truly atomic yet. The filecheck and rename decoupled non-atomic operations.
// todo: use libc or rustix to do TOCTOU-safe renames.
pub(crate) fn atomic_rename(source_path: &Path, dest_path: &Path) -> Result<(), anyhow::Error> {
    match fs::exists(dest_path) {
        Ok(x) => {
            if x {
                let is_it_the_same_file = match is_same_file(source_path, dest_path) {
                    Ok(b) => b,
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "ERN001 : Error with the malformed or prexisting target file {:?} : {e}",
                            dest_path
                        ));
                    }
                };
                // Only passes when is_case_only_rename and is_it_the_same_file are both true
                // If not do not rename, report an error and move on
                if !is_case_only_rename(source_path, dest_path) || !is_it_the_same_file {
                    return Err(anyhow::anyhow!(
                        "ERN001 : Target file {:?} already exists or was created mid-rename-operation.",
                        dest_path
                    ));
                }
            }
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "ERN002 : Some error during Filerename.  Details: {}",
                e
            ));
        }
    }
    match fs::rename(source_path, dest_path) {
        Ok(_) => Ok(()),
        Err(x) => match x.kind() {
            io::ErrorKind::IsADirectory => Err(anyhow::anyhow!(
                "A directory with the target file name {:?} exists or was created mid-rename-operation.",
                dest_path
            )),
            io::ErrorKind::AlreadyExists => Err(anyhow::anyhow!(
                "TOCTOU Blocked: Target file {:?} already exists or was created mid-rename-operation.",
                dest_path
            )),
            io::ErrorKind::CrossesDevices => Err(anyhow::anyhow!(
                "Source and destination are on different devices. Aborting for safety."
            )),
            other_kind => Err(anyhow::anyhow!(
                "ERN003 : Some error during file moving. Kind: {:?}, Details: {}",
                other_kind,
                x
            )),
        },
    }
}

#[cfg(test)]
#[path = "./tests/rename_tests.rs"]
mod rename_tests;
