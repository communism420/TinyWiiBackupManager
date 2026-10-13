// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::*;
use crate::games::disc_reader::{SharedMultiFileReader, retrieve_multi_disc_files};
use nod::read::DiscStream;
use std::io::{self, Cursor, Seek, Write};
use zip::{ZipWriter, write::SimpleFileOptions};

fn entry(dir: &Path, name: &str, disc: u8) -> ImportEntry {
    let path = dir.join(name);
    let mut data = vec![0; 0x8000];
    data[..6].copy_from_slice(b"GZZE01");
    data[6] = disc;
    data[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
    data[0x20..0x24].copy_from_slice(b"Test");
    std::fs::write(&path, data).unwrap();
    smol::block_on(ImportEntry::new(path)).unwrap()
}

fn import_zip_with_leading_entry(directory_first: bool) {
    let temp = tempfile::tempdir().unwrap();
    let original = entry(temp.path(), "original.iso", 0);
    let image = std::fs::read(original.path()).unwrap();
    let source = temp.path().join("source.zip");
    let mut zip = ZipWriter::new(std::fs::File::create(&source).unwrap());
    let options = SimpleFileOptions::default();
    if directory_first {
        zip.add_directory("images/", options).unwrap();
    } else {
        zip.start_file("README.txt", options).unwrap();
        zip.write_all(b"This archive contains one game.").unwrap();
    }
    zip.start_file("images/actual.GCM", options).unwrap();
    zip.write_all(&image).unwrap();
    zip.finish().unwrap();
    let source_before = std::fs::read(&source).unwrap();
    let imported = smol::block_on(ImportEntry::new(source.clone())).unwrap();
    assert_eq!(imported.meta().game_id(), original.meta().game_id());
    let mut files = retrieve_multi_disc_files(imported.path()).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].stream_position().unwrap(), 0);
    let mut reader = SharedMultiFileReader::new(files).unwrap();
    assert_eq!(reader.stream_len().unwrap(), image.len() as u64);
    let mut extracted = vec![0; image.len()];
    reader.read_exact_at(&mut extracted, 0).unwrap();
    assert_eq!(extracted, image);
    assert_eq!(std::fs::read(source).unwrap(), source_before);
}

#[test]
fn game_zip_with_leading_directory_extracts_the_image_completely() {
    import_zip_with_leading_entry(true);
}

#[test]
fn game_zip_with_leading_readme_extracts_the_image_completely() {
    import_zip_with_leading_entry(false);
}

#[test]
fn game_zip_with_multiple_images_is_rejected_without_changes() {
    let temp = tempfile::tempdir().unwrap();
    let original = entry(temp.path(), "original.iso", 0);
    let image = std::fs::read(original.path()).unwrap();
    let source = temp.path().join("multiple.zip");
    let mut zip = ZipWriter::new(std::fs::File::create(&source).unwrap());
    for filename in ["one.iso", "two.iso"] {
        zip.start_file(filename, SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&image).unwrap();
    }
    zip.finish().unwrap();
    let before = std::fs::read(&source).unwrap();
    assert!(smol::block_on(ImportEntry::new(source.clone())).is_err());
    assert_eq!(std::fs::read(source).unwrap(), before);
}

#[test]
fn zip_selection_checks_cancellation_before_each_entry_and_after_scanning() {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    zip.add_directory("images/", options).unwrap();
    zip.start_file("README.txt", options).unwrap();
    zip.write_all(b"One game follows").unwrap();
    zip.start_file("images/game.iso", options).unwrap();
    zip.write_all(b"selection does not extract the image")
        .unwrap();
    let archive_bytes = zip.finish().unwrap().into_inner();

    // Checkpoints cover directories, non-image entries, the image itself,
    // and cancellation arriving immediately after the last entry.
    for cancel_at in 0..=3 {
        let mut archive = ZipArchive::new(Cursor::new(&archive_bytes)).unwrap();
        let mut calls = 0;
        let error = zip_image::game_index(&mut archive, || {
            let cancelled = calls == cancel_at;
            calls += 1;
            if cancelled {
                Err(io::Error::other("selection cancelled"))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(calls, cancel_at + 1);
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::Other
        );
        assert_eq!(error.to_string(), "selection cancelled");
    }

    let mut archive = ZipArchive::new(Cursor::new(&archive_bytes)).unwrap();
    let mut calls = 0;
    let index = zip_image::game_index(&mut archive, || {
        calls += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(index, 2);
    assert_eq!(calls, 4);
}
