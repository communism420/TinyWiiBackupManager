// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use arrayvec::ArrayVec;
use nod::read::DiscStream;
use std::{fs::File, io, sync::Arc};

mod retrieve;
pub use retrieve::retrieve_multi_disc_files;

#[cfg(unix)]
use std::os::unix::fs::FileExt;

#[cfg(windows)]
use std::os::windows::fs::FileExt;

#[cfg(windows)]
trait ReadExactAt {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> io::Result<()>;
}

#[cfg(windows)]
impl ReadExactAt for File {
    fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        while !buf.is_empty() {
            match self.seek_read(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &mut buf[n..];
                    offset += n as u64;
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        if !buf.is_empty() {
            Err(io::ErrorKind::UnexpectedEof.into())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct SharedMultiFileReader {
    inner: ArrayVec<(Arc<File>, u64), 4>,
    stream_len: u64,
}

use std::path::Path;

pub trait DiscCancellation: Clone + Send + Sync {
    fn check(&self) -> io::Result<()>;
}

#[derive(Debug)]
pub(crate) struct DiscCancelled(pub &'static str);

impl std::fmt::Display for DiscCancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} cancelled", self.0)
    }
}
impl std::error::Error for DiscCancelled {}

pub(crate) fn error_contains<E: std::error::Error + 'static>(
    error: &(dyn std::error::Error + 'static),
) -> bool {
    if error.is::<E>() {
        return true;
    }
    if let Some(inner) = error
        .downcast_ref::<io::Error>()
        .and_then(io::Error::get_ref)
    {
        return error_contains::<E>(inner);
    }
    error.source().is_some_and(error_contains::<E>)
}

/// Keeps cancellation active when nod clones a stream for its worker threads.
#[derive(Clone)]
pub struct CancellableDiscStream<C> {
    inner: SharedMultiFileReader,
    cancel: C,
}

impl<C: DiscCancellation> CancellableDiscStream<C> {
    pub fn new(inner: SharedMultiFileReader, cancel: C) -> Self {
        Self { inner, cancel }
    }
}

impl<C: DiscCancellation + 'static> DiscStream for CancellableDiscStream<C> {
    fn stream_len(&mut self) -> io::Result<u64> {
        self.cancel.check()?;
        let len = self.inner.stream_len()?;
        self.cancel.check()?;
        Ok(len)
    }

    fn read_exact_at(&mut self, buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        self.cancel.check()?;
        // A format may request a large metadata table or compressed block in
        // one call. Bound each source read so cancellation is checked in between.
        for chunk in buf.chunks_mut(1024 * 1024) {
            self.cancel.check()?;
            self.inner.read_exact_at(chunk, offset)?;
            self.cancel.check()?;
            offset = offset
                .checked_add(chunk.len() as u64)
                .ok_or_else(|| io::Error::other("disc read offset overflow"))?;
        }
        self.cancel.check()
    }
}

pub fn retrieve_multi_disc_files_cancellable<C: DiscCancellation>(
    path: &Path,
    cancel: &C,
) -> Result<ArrayVec<File, 4>> {
    retrieve::retrieve_multi_disc_files_with_check(path, || cancel.check())
}

impl SharedMultiFileReader {
    pub fn new(files: impl IntoIterator<Item = File>) -> Result<Self> {
        let mut inner = ArrayVec::new();
        let mut stream_len = 0;

        for file in files {
            let size = file.metadata()?.len();
            inner.push((Arc::new(file), size));
            stream_len += size;
        }

        Ok(Self { inner, stream_len })
    }
}

#[cfg(test)]
mod tests;

impl DiscStream for SharedMultiFileReader {
    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.stream_len)
    }

    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        let mut offset = offset;
        let mut buf = buf;

        for (file, len) in self.inner.iter() {
            // If the offset is past the end of the file, skip it
            if offset >= *len {
                offset -= len;
                continue;
            }

            // Limit in u64 before narrowing: a single ISO may exceed usize on x86.
            let n = (*len - offset).min(buf.len() as u64) as usize;
            file.read_exact_at(&mut buf[..n], offset)?;
            buf = &mut buf[n..];
            offset = 0;

            if buf.is_empty() {
                return Ok(());
            }
        }

        if buf.is_empty() {
            Ok(())
        } else {
            Err(io::ErrorKind::UnexpectedEof.into())
        }
    }
}
