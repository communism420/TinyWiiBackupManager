// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result, bail};
use std::{
    ffi::OsStr,
    io::{self, Read, Seek},
    path::Path,
};
use zip::ZipArchive;

// Metadata discovery and extraction must select the same member. ZIP archives
// commonly place a directory or README before their game image.
pub(crate) fn game_index<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    mut check: impl FnMut() -> io::Result<()>,
) -> Result<usize> {
    let mut first_file = None;
    let mut image = None;
    for index in 0..archive.len() {
        check()?;
        let entry = archive.by_index_data(index)?;
        if !entry.is_file() {
            continue;
        }
        first_file.get_or_insert(index);
        let name = entry.name()?;
        let extension = Path::new(name.as_ref())
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        if ["wbf1", "wbf2", "wbf3"]
            .iter()
            .any(|supported| extension.eq_ignore_ascii_case(supported))
        {
            bail!("Extract split game images from the ZIP before importing them");
        }
        if ["iso", "gcm", "wia", "rvz", "wbfs", "ciso", "gcz", "tgc"]
            .iter()
            .any(|supported| extension.eq_ignore_ascii_case(supported))
            && image.replace(index).is_some()
        {
            bail!("ZIP contains multiple game images; extract them before importing");
        }
    }
    check()?;
    // Preserve support for a single image whose member has no usual extension;
    // ImportEntry validates the selected member's actual disc header below.
    image.or(first_file).context("ZIP contains no game image")
}
