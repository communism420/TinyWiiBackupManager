// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::*;
use std::io::{Read, Write};
use std::{
    io::Cursor,
    sync::atomic::{AtomicBool, Ordering},
};
use tempfile::tempfile;
use zip::ZipArchive;

#[derive(Clone, Default)]
struct TestCancellation(Arc<AtomicBool>);

impl TestCancellation {
    fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

impl DiscCancellation for TestCancellation {
    fn check(&self) -> io::Result<()> {
        if self.0.load(Ordering::Acquire) {
            Err(io::Error::other(DiscCancelled("Test")))
        } else {
            Ok(())
        }
    }
}
use zip::{ZipWriter, write::SimpleFileOptions};

#[test]
fn large_part_lengths_do_not_truncate_small_reads() {
    let mut first = tempfile().unwrap();
    let mut second = tempfile().unwrap();
    first.write_all(b"first123").unwrap();
    second.write_all(b"second45").unwrap();
    for first_length in [1u64 << 32, (1u64 << 32) + 1] {
        let mut reader =
            SharedMultiFileReader::new([first.try_clone().unwrap(), second.try_clone().unwrap()])
                .unwrap();
        // Model a large recorded extent without allocating a 4 GiB fixture.
        // Only the real prefixes of each backing file are read below.
        reader.inner[0].1 = first_length;
        reader.stream_len = first_length + 8;
        assert_eq!(reader.stream_len().unwrap(), first_length + 8);

        let mut actual = [0u8; 8];
        reader.read_exact_at(&mut actual, 0).unwrap();
        assert_eq!(&actual, b"first123", "first part length {first_length}");
        reader.read_exact_at(&mut actual, first_length).unwrap();
        assert_eq!(&actual, b"second45");
    }
}

#[test]
fn split_iso_reads_two_and_three_parts() {
    for uppercase in [false, true] {
        for parts in 2..=3 {
            let dir = tempfile::tempdir().unwrap();
            let mut expected = Vec::new();
            let mut primary = None;
            for part in 0..parts {
                let mut name = format!("game.part{part}.iso");
                if uppercase {
                    name.make_ascii_uppercase();
                }
                let path = dir.path().join(name);
                let data = vec![part as u8 + 1; 17];
                std::fs::write(&path, &data).unwrap();
                expected.extend(data);
                primary.get_or_insert(path);
            }
            let files = retrieve_multi_disc_files(&primary.unwrap()).unwrap();
            assert_eq!(files.len(), parts);
            let mut reader = SharedMultiFileReader::new(files).unwrap();
            let mut actual = vec![0; expected.len()];
            reader.read_exact_at(&mut actual, 0).unwrap();
            assert_eq!(actual, expected);
        }
    }
}

fn zip_data(data: &[u8]) -> Vec<u8> {
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    archive
        .start_file("game.iso", SimpleFileOptions::default())
        .unwrap();
    archive.write_all(data).unwrap();
    archive.finish().unwrap().into_inner()
}

#[test]
fn cancellable_stream_reads_split_files_and_shares_cancellation_with_clones() {
    let mut first = tempfile().unwrap();
    let mut second = tempfile().unwrap();
    first.write_all(&vec![0x42; 600_000]).unwrap();
    second.write_all(&vec![0x91; 600_000]).unwrap();
    let cancel = TestCancellation::default();
    let mut stream = CancellableDiscStream::new(
        SharedMultiFileReader::new([first, second]).unwrap(),
        cancel.clone(),
    );
    assert_eq!(stream.stream_len().unwrap(), 1_200_000);
    let mut data = vec![0; 1_200_000];
    stream.read_exact_at(&mut data, 0).unwrap();
    assert!(data[..600_000].iter().all(|&byte| byte == 0x42));
    assert!(data[600_000..].iter().all(|&byte| byte == 0x91));

    let mut clone = stream.clone();
    cancel.cancel();
    assert_eq!(
        clone.read_exact_at(&mut [0; 4], 0).unwrap_err().kind(),
        io::ErrorKind::Other
    );
    assert_eq!(
        stream.stream_len().unwrap_err().kind(),
        io::ErrorKind::Other
    );
    assert!(stream.read_exact_at(&mut [], 0).is_err());
}

#[test]
fn cancelled_retrieval_stops_before_opening_source() {
    let cancel = TestCancellation::default();
    cancel.cancel();
    let error =
        retrieve_multi_disc_files_cancellable(Path::new("missing.zip"), &cancel).unwrap_err();
    assert_eq!(
        error.downcast_ref::<io::Error>().unwrap().kind(),
        io::ErrorKind::Other
    );
}

#[test]
fn cancellable_zip_extraction_preserves_source_and_contents() {
    let data = vec![0x65; 1_100_000];
    let archive = zip_data(&data);
    let mut source = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
    source.write_all(&archive).unwrap();
    let files =
        retrieve_multi_disc_files_cancellable(source.path(), &TestCancellation::default()).unwrap();
    let mut stream = SharedMultiFileReader::new(files).unwrap();
    assert_eq!(stream.stream_len().unwrap(), data.len() as u64);
    let mut extracted = vec![0; data.len()];
    stream.read_exact_at(&mut extracted, 0).unwrap();
    assert_eq!(extracted, data);
    assert_eq!(std::fs::read(source.path()).unwrap(), archive);
}

#[test]
fn cancellation_during_zip_read_stops_before_writing_extracted_chunk() {
    struct CancelAfterRead<R> {
        reader: R,
        cancel: TestCancellation,
    }
    impl<R: Read> Read for CancelAfterRead<R> {
        fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
            let read = self.reader.read(data)?;
            self.cancel.cancel();
            Ok(read)
        }
    }

    let mut archive = ZipArchive::new(Cursor::new(zip_data(&vec![7; 1_100_000]))).unwrap();
    let cancel = TestCancellation::default();
    let mut reader = CancelAfterRead {
        reader: archive.by_index(0).unwrap(),
        cancel: cancel.clone(),
    };
    let mut output = Vec::new();
    let error = retrieve::copy_with_check(&mut reader, &mut output, || cancel.check()).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert!(output.is_empty());
}
