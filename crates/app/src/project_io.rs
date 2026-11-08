//! Safe filesystem operations for native project documents.
//!
//! The core crate stays independent of the filesystem and operating system.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use fireeffects_core::{decode_project, encode_project, Project};

const MAX_PROJECT_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// Ensure a Save As path uses the native .firefx extension.
pub fn normalize_save_path(path: &Path) -> Result<PathBuf, String> {
    let mut path = path.to_path_buf();
    match path.extension().and_then(|extension| extension.to_str()) {
        None => {
            path.set_extension(fireeffects_core::PROJECT_FILE_EXTENSION);
        }
        Some(extension)
            if extension.eq_ignore_ascii_case(fireeffects_core::PROJECT_FILE_EXTENSION) => {}
        Some(extension) => {
            return Err(format!(
                "project files must use the .firefx extension, not .{extension}"
            ));
        }
    }
    Ok(path)
}

/// Read, decode, and validate a project without changing the source file.
pub fn read_project(path: &Path) -> Result<Project, String> {
    validate_extension(path)?;
    let metadata = fs::metadata(path).map_err(|error| format!("could not read file metadata: {error}"))?;
    if metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err("project file is larger than the 64 MiB safety limit".to_owned());
    }

    let bytes = fs::read(path).map_err(|error| format!("could not read project file: {error}"))?;
    decode_project(&bytes).map_err(|error| error.to_string())
}

/// Save a project. Existing files are replaced only when overwrite is true.
pub fn write_project(path: &Path, project: &Project, overwrite: bool) -> Result<(), String> {
    validate_extension(path)?;
    if path.exists() && !overwrite {
        return Err("that file already exists; open it or choose a different name".to_owned());
    }

    let bytes = encode_project(project).map_err(|error| error.to_string())?;
    atomic_write(path, &bytes).map_err(|error| format!("could not save project: {error}"))
}

fn validate_extension(path: &Path) -> Result<(), String> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension)
            if extension.eq_ignore_ascii_case(fireeffects_core::PROJECT_FILE_EXTENSION) =>
        {
            Ok(())
        }
        _ => Err("select a .firefx project file".to_owned()),
    }
}

/// Write beside the destination, then rename into place. When replacing an
/// existing project, keep a temporary backup until the new file is in place.
fn atomic_write(destination: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = destination
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing file name"))?
        .to_string_lossy();
    let suffix = std::process::id();
    let temporary = parent.join(format!(".{file_name}.{suffix}.tmp"));
    let backup = parent.join(format!(".{file_name}.{suffix}.bak"));

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    drop(file);

    if destination.exists() {
        if backup.exists() {
            let _ = fs::remove_file(&temporary);
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "a previous save backup needs attention",
            ));
        }
        if let Err(error) = fs::rename(destination, &backup) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        if let Err(error) = fs::rename(&temporary, destination) {
            let restore = fs::rename(&backup, destination);
            let _ = fs::remove_file(&temporary);
            if let Err(restore_error) = restore {
                return Err(std::io::Error::other(format!(
                    "{error}; restoring the previous project also failed: {restore_error}"
                )));
            }
            return Err(error);
        }
        let _ = fs::remove_file(backup);
    } else if let Err(error) = fs::rename(&temporary, destination) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_path() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("fireeffects-{stamp}.firefx"))
    }

    #[test]
    fn saves_and_reopens_a_project() {
        let path = test_path();
        let project = Project::new("Filesystem round trip");
        write_project(&path, &project, false).expect("first save should work");
        assert_eq!(
            read_project(&path).expect("saved project should reopen"),
            project
        );
        fs::remove_file(path).expect("test project should be removable");
    }

    #[test]
    fn refuses_to_overwrite_an_unrelated_existing_file() {
        let path = test_path();
        let project = Project::new("No overwrite");
        write_project(&path, &project, false).expect("first save should work");
        assert!(write_project(&path, &Project::new("Replacement"), false).is_err());
        assert_eq!(
            read_project(&path).expect("original should remain"),
            project
        );
        fs::remove_file(path).expect("test project should be removable");
    }

    #[test]
    fn adds_the_native_extension_when_missing() {
        assert_eq!(
            normalize_save_path(Path::new("sample")).unwrap(),
            PathBuf::from("sample.firefx")
        );
    }
}
