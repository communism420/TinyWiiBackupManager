# Local nod patches: lossless conversion and cancellation cleanup

This directory contains the `nod` crate based on
`https://github.com/wiidev/nod`, branch `fix-block-handling`, commit
`9a7a6b2ef0a1036ff687f8e8ce0d0238064d4aaf` (`2.0.0-alpha.10`).
The crate sources and tests match the local integration snapshot
`6590d85b67f9a32aaecebca8ca34f1f6b5b83df9`, which applies the patches listed
below to that base. This snapshot is recorded for provenance; it is not a
published revision or an external build dependency. The upstream README and
MIT/Apache-2.0 licenses are preserved.
`Cargo.toml` expands inherited workspace package/dependency values without
changing their versions or features; its README path is local to this directory.
The Python package files and the other upstream workspace members are not needed
by TinyWiiBackupManagerPlus and are not vendored.

## Patch provenance

The following local history commits, in order, are included after the upstream
base. Their complete source changes are present in this directory, so these
commit identifiers do not need to be available in a remote repository:

- `4b515c5a16c540119a0d92cd46c80037d6eefcb2`: retain WIA hash exceptions in sector 63.
- `53ee9250ee57d55dbcb7c564cbdcadfb5cb2f0d1`: disconnect and join preloader workers on shutdown.
- `f388177ee70e03e229ec2ba0cdf5f204e6b4dc1a`: join all digest workers on finish and drop.
- `78ebad4d595052fbc4799eab0fa161c143422906`: expose split image component paths.
- `420af0f8d4ff5dd5eeda5f398de748244661c0a4`: preserve original Wii hash areas in WIA and RVZ output.
- `b7afec9cab32ca322e5949398fc94136dbd1b43f`: report WIA progress when groups write no output.
- `6590d85b67f9a32aaecebca8ca34f1f6b5b83df9`: format the sector boundary regression.

The application uses this repository-relative crate directly, so a clean clone
does not need a developer-specific dependency override. Build artifacts, private
disc images, and other upstream workspace members are not included.

## Why the patch is needed

The pinned WIA/RVZ writer contained a TODO that wrote an empty hash-exception
list for every Wii partition chunk. WIA/RVZ removes Wii partition hash areas and
reconstructs them when reading. Without exceptions, a source containing hashes
or padding that differ from the reconstructed values silently changes during
conversion, despite successful export and an embedded original SHA1.

This was reproduced by reading back real exported images and comparing their
decoded SHA1 with the source SHA1. The differences were limited to partition
hash areas; changing a dump to match Redump is not part of this patch. An export
must preserve the decoded input bytes, including nonstandard or invalid hashes.

## Changes

- `src/io/wia.rs`: Generate actual exception lists by comparing original,
  decrypted hash areas with freshly calculated hashes for a full,
  partition-aligned group of 64 sectors. Existing H0 hashes are not trusted when
  computing the reference. Cache these group hashes between adjacent chunks,
  keyed by the exact partition-relative group start so short neighboring
  partitions cannot share cached hashes.
- Preserve hash-area padding as well as H0/H1/H2. The 32-byte padding areas use
  two overlapping 20-byte exceptions, limiting each sector to 52 exceptions.
- Encode offsets relative to the RVZ chunk for chunks below 2 MiB, or relative
  to each 2 MiB exception list for larger chunks. Emit empty trailing lists for
  short final chunks, and use the reader's zero-filled final sector group.
- Size WIA compression buffers for the actual payload and exception data.
  Worst-case exceptions can exceed the space saved by removing hash areas.
- `src/io/block.rs`: Widen the exclusive end offset when selecting exceptions
  for sector 63. Its value is 65536 and cannot be represented by `u16`; the old
  addition overflowed in debug builds and discarded that sector's exceptions
  in release builds.
- Add focused unit and public-API round-trip tests for preservation of unusual
  hashes and padding, including small RVZ chunks, the last sector of a group,
  partial final groups, encryption and compressed output.
- The sector-63 unit regression verifies exception range selection and rebasing
  at the final sector boundary; `tests/wia_lossless.rs` uses synthetic discs.

These changes are kept in source control instead of modifying Cargo's shared
cache. The vendored dependency can be removed after a suitable upstream fix is
available and equivalent round-trip coverage passes against it.

## Source protection for manual conversion

- `src/read.rs`: Expose `DiscReader::split_file_paths()` using the existing
  `SplitFileReader` discovery and file order. TinyWiiBackupManager checks every
  component before replacing a confirmed conversion destination, preventing an
  output such as `game.part1.iso` from replacing part of its own input.
- A focused test checks that the accessor and reader use the same files for
  `.iso.1`, `.partN.iso`, and `.wbfN` naming. Discovery and reading are unchanged.

## Cancellation cleanup and progress checkpoints

- `src/disc/preloader.rs`: On drop, disconnect the original request and
  statistics channels before joining every preloader thread. Idle workers stop
  waiting for work; active workers stop after their current load. This ensures
  their source readers are released before an export reports cancellation.
  `shutdown(self)` uses the same drop path; its previous implementation could
  wait indefinitely with the request sender still alive.
- `src/util/digest.rs`: Both normal `finish()` and early drop close all digest
  input channels before joining all digest workers. Errors and cancellation no
  longer detach workers. Normal completion still returns every digest; drop
  joins workers without propagating panics during error cleanup.
- `src/io/wia.rs`: Invoke the existing data callback with empty data before
  processing each completed group and at finalization checkpoints. This lets
  callers observe progress and cancellation even when a group is empty or
  reuses an earlier group's data and therefore writes no bytes.
- Focused lifecycle tests check channel-disconnection order, joining all
  workers, and preservation of normal digest results. Export cancellation is
  cooperative: an already executing OS read or codec call finishes before the
  next checkpoint.
- `tests/wia_progress.rs` uses synthetic empty and repeated groups to check
  progress callbacks and cancellation errors at group and finalization
  checkpoints. `DataCallback` documents that these callbacks can contain no
  output bytes.
