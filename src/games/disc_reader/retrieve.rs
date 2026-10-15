// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::zip_image::game_index;
use anyhow::{Context, Result};
use arrayvec::ArrayVec;
use std::{
    ffi::OsStr,
    fs::File,
    io::{self, Read, Seek, Write},
    path::Path,
};
use tempfile::tempfile;
use zip::ZipArchive;

pub fn retrieve_multi_disc_files(path: &Path) -> Result<ArrayVec<File, 4>> {
    retrieve_multi_disc_files_with_check(path, || Ok(()))
}

pub(super) fn copy_with_check(
    reader: &mut impl Read,
    writer: &mut impl Write,
    mut check: impl FnMut() -> io::Result<()>,
) -> io::Result<()> {
    let mut buffer = vec![0; 1024 * 1024];
    loop {
        check()?;
        let read = reader.read(&mut buffer)?;
        check()?;
        if read == 0 {
            return Ok(());
        }
        writer.write_all(&buffer[..read])?;
        check()?;
    }
}

pub(super) fn retrieve_multi_disc_files_with_check(
    path: &Path,
    mut check: impl FnMut() -> io::Result<()>,
) -> Result<ArrayVec<File, 4>> {
    fn open(path: &Path, mut check: impl FnMut() -> io::Result<()>) -> io::Result<File> {
        check()?;
        let file = File::open(path)?;
        check()?;
        Ok(file)
    }

    check()?;
    let mut files = ArrayVec::new();
    let ext = path.extension().context("invalid extension")?;

    if ext.eq_ignore_ascii_case("zip") {
        let mut zip = ZipArchive::new(open(path, &mut check)?)?;
        check()?;
        let index = game_index(&mut zip, &mut check)?;
        let mut entry = zip.by_index(index)?;
        check()?;

        let mut tmp = tempfile()?;
        check()?;
        copy_with_check(&mut entry, &mut tmp, &mut check)?;
        check()?;
        tmp.flush()?;
        check()?;
        tmp.rewind()?;
        check()?;
        files.push(tmp);
    } else {
        let filename = path
            .file_name()
            .and_then(OsStr::to_str)
            .context("invalid filename")?;
        files.push(open(path, &mut check)?);
        if let Some(filename) = filename.strip_suffix(".part0.iso") {
            for i in 1..=3 {
                check()?;
                let part_path = path.with_file_name(format!("{filename}.part{i}.iso"));
                if i > 1 && !part_path.exists() {
                    break;
                }
                files.push(open(&part_path, &mut check)?);
            }
        } else if let Some(filename) = filename.strip_suffix(".PART0.ISO") {
            for i in 1..=3 {
                check()?;
                let part_path = path.with_file_name(format!("{filename}.PART{i}.ISO"));
                if i > 1 && !part_path.exists() {
                    break;
                }
                files.push(open(&part_path, &mut check)?);
            }
        } else if let Some(filename) = filename.strip_suffix(".wbfs") {
            for i in 1..=3 {
                check()?;
                let wbfx_path = path.with_file_name(format!("{filename}.wbf{i}"));
                if !wbfx_path.exists() {
                    break;
                }
                files.push(open(&wbfx_path, &mut check)?);
            }
        } else if let Some(filename) = filename.strip_suffix(".WBFS") {
            for i in 1..=3 {
                check()?;
                let wbfx_path = path.with_file_name(format!("{filename}.WBF{i}"));
                if !wbfx_path.exists() {
                    break;
                }
                files.push(open(&wbfx_path, &mut check)?);
            }
        }
    }

    check()?;
    Ok(files)
}
