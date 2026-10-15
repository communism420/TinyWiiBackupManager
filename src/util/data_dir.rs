// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use directories::ProjectDirs;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn get_data_dir() -> io::Result<PathBuf> {
    let (current, legacy) = match is_portable() {
        true => get_portable_dir(),
        false => get_user_dir().or_else(get_portable_dir),
    }
    .ok_or_else(|| io::Error::other("Unable to find a writable data directory"))?;

    // Keep each application's settings independent. Copy settings once, without
    // moving the original data or duplicating downloaded covers and other caches.
    migrate_config(&legacy, &current)?;
    Ok(current)
}

fn is_portable() -> bool {
    let Ok(current_exe) = std::env::current_exe() else {
        return false;
    };

    let Some(stem) = current_exe
        .file_stem()
        .map(|s| s.to_string_lossy().to_ascii_lowercase())
    else {
        return false;
    };

    if ["not-portable", "notportable", "no-portable", "noportable"]
        .into_iter()
        .any(|s| stem.contains(s))
    {
        return false;
    }

    if stem.contains("portable") {
        return true;
    }

    // if parent is a drive root
    current_exe.parent().and_then(|p| p.parent()).is_none()
}

fn get_user_dir() -> Option<(PathBuf, PathBuf)> {
    let current = ProjectDirs::from("io.github", "communism420", "TinyWiiBackupManagerPlus")?
        .data_dir()
        .to_path_buf();
    let legacy = ProjectDirs::from("it", "mq1", "TinyWiiBackupManager")?
        .data_dir()
        .to_path_buf();
    fs::create_dir_all(&current).ok()?;
    Some((current, legacy))
}

fn get_portable_dir() -> Option<(PathBuf, PathBuf)> {
    let executable = std::env::current_exe().ok()?;
    let current = executable.with_file_name("TinyWiiBackupManagerPlus-data");
    let legacy = executable.with_file_name("TinyWiiBackupManager-data");
    fs::create_dir_all(&current).ok()?;
    Some((current, legacy))
}

fn migrate_config(legacy: &Path, current: &Path) -> io::Result<()> {
    let destination = current.join("config.json");
    if destination.try_exists()? {
        return Ok(());
    }

    let mut source = match fs::File::open(legacy.join("config.json")) {
        Ok(source) => source,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    let mut temporary = tempfile::NamedTempFile::new_in(current)?;
    io::copy(&mut source, &mut temporary)?;
    temporary.as_file().sync_all()?;
    match temporary.persist_noclobber(destination) {
        Ok(_) => Ok(()),
        Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error.error),
    }
}

#[cfg(test)]
mod tests {
    use super::migrate_config;
    use std::fs;

    #[test]
    fn migrates_settings_without_changing_legacy_data() {
        let legacy = tempfile::tempdir().unwrap();
        let current = tempfile::tempdir().unwrap();
        let settings = br#"{"fast_transfer":true,"wii_ip":"192.168.1.42"}"#;
        fs::write(legacy.path().join("config.json"), settings).unwrap();
        fs::write(legacy.path().join("cached-cover.png"), b"cover").unwrap();

        migrate_config(legacy.path(), current.path()).unwrap();

        assert_eq!(
            fs::read(current.path().join("config.json")).unwrap(),
            settings
        );
        assert_eq!(
            fs::read(legacy.path().join("config.json")).unwrap(),
            settings
        );
        assert_eq!(
            fs::read(legacy.path().join("cached-cover.png")).unwrap(),
            b"cover"
        );
        assert!(!current.path().join("cached-cover.png").exists());
        assert_eq!(fs::read_dir(current.path()).unwrap().count(), 1);
    }

    #[test]
    fn preserves_existing_plus_settings() {
        let legacy = tempfile::tempdir().unwrap();
        let current = tempfile::tempdir().unwrap();
        fs::write(legacy.path().join("config.json"), b"legacy").unwrap();
        fs::write(current.path().join("config.json"), b"plus").unwrap();

        migrate_config(legacy.path(), current.path()).unwrap();

        assert_eq!(
            fs::read(current.path().join("config.json")).unwrap(),
            b"plus"
        );
        assert_eq!(
            fs::read(legacy.path().join("config.json")).unwrap(),
            b"legacy"
        );
    }

    #[test]
    fn missing_legacy_settings_leave_new_directory_empty() {
        let root = tempfile::tempdir().unwrap();
        let current = tempfile::tempdir().unwrap();

        migrate_config(&root.path().join("absent"), current.path()).unwrap();

        assert_eq!(fs::read_dir(current.path()).unwrap().count(), 0);
    }

    #[test]
    fn failed_migration_does_not_leave_partial_settings() {
        let legacy = tempfile::tempdir().unwrap();
        let current = tempfile::tempdir().unwrap();
        fs::create_dir(legacy.path().join("config.json")).unwrap();

        assert!(migrate_config(legacy.path(), current.path()).is_err());

        assert_eq!(fs::read_dir(current.path()).unwrap().count(), 0);
    }
}
