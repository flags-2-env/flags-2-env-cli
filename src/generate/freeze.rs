#![forbid(unsafe_code)]

use crate::error::CliError;
use std::fs;
use std::path::Path;

/// Write a generated artifact and mark it read-only (`chmod a-w` / 0444).
///
/// Git only stores the executable bit, not the Unix write bit, so clones come
/// back writable. The generator (and `scripts/freeze-generated.sh`) restore
/// 0444 after write/checkout. Callers must unfreeze before overwriting.
pub fn write_frozen(path: &Path, source: &str) -> Result<(), CliError> {
    match fs::read_to_string(path) {
        Ok(existing) if existing == source => {
            freeze(path)?;
            return Ok(());
        }
        _ => {}
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    unfreeze(path)?;
    fs::write(path, source)?;
    freeze(path)?;
    Ok(())
}

pub fn freeze(path: &Path) -> Result<(), CliError> {
    if !path.is_file() {
        return Ok(());
    }
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(path, perms)?;
    Ok(())
}

pub fn unfreeze(path: &Path) -> Result<(), CliError> {
    if !path.is_file() {
        return Ok(());
    }
    let mut perms = fs::metadata(path)?.permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{freeze, unfreeze, write_frozen};
    use std::fs;
    use std::path::PathBuf;

    fn temp_file(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("f2e-freeze-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir.join("artifact.txt")
    }

    #[test]
    fn write_frozen_makes_the_file_read_only_and_can_overwrite() {
        let path = temp_file("rw");
        write_frozen(&path, "one\n").unwrap();
        assert!(fs::metadata(&path).unwrap().permissions().readonly());
        assert_eq!(fs::read_to_string(&path).unwrap(), "one\n");
        write_frozen(&path, "two\n").unwrap();
        assert!(fs::metadata(&path).unwrap().permissions().readonly());
        assert_eq!(fs::read_to_string(&path).unwrap(), "two\n");
        unfreeze(&path).unwrap();
        assert!(!fs::metadata(&path).unwrap().permissions().readonly());
        freeze(&path).unwrap();
        assert!(fs::metadata(&path).unwrap().permissions().readonly());
        let _ = fs::remove_file(&path);
    }
}
