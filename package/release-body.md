# TinyWiiBackupManagerPlus 1.0.0

The first release of this fork of [TinyWiiBackupManager](https://github.com/mq1/TinyWiiBackupManager), originally developed by Manuel Quarneti and its contributors. It includes the original 6.1.7 functionality, Wiiload 0.7.1, and our game-management improvements. Original authorship and licenses are preserved.

## What is included

- Select games individually or use Select all for import, export, and SHA1 verification.
- Import and export with 1–4 simultaneous games; cancel single, sequential, or parallel transfers while retaining completed files.
- Optional Fast transfer, off by default, with no additional low-level settings. Its benefit depends on the source, destination, and format.
- Choose RVZ, ISO, WBFS, CISO, GCZ, or WIA in the Windows single-game export/conversion save dialog.
- Staged writes, source-file protection, split-image handling, safer ZIP imports and path normalization, and bundled WIA/RVZ integrity fixes.
- Homebrew metadata, catalog-cache validation, HTTP timeout, and interface fixes.
- The TinyWiiBackupManagerPlus name and an independent version sequence starting at 1.0.0.

See the [changelog](https://github.com/communism420/TinyWiiBackupManagerPlus/blob/v1.0.0/CHANGELOG.md) for details and [vendored dependency notes](https://github.com/communism420/TinyWiiBackupManagerPlus/blob/v1.0.0/vendor/nod/PATCHES.md) for provenance.

## Choose your download

| System | File |
| --- | --- |
| Windows, most Intel/AMD PCs | `TinyWiiBackupManagerPlus-v1.0.0-windows-x64.exe` |
| Windows, 32-bit / legacy systems | `TinyWiiBackupManagerPlus-v1.0.0-windows-x86.exe` |
| Windows on ARM | `TinyWiiBackupManagerPlus-v1.0.0-windows-arm64.exe` |
| Linux, most Intel/AMD PCs | `TinyWiiBackupManagerPlus-v1.0.0-linux-x86_64.AppImage` |
| Linux, 32-bit Intel/AMD | `TinyWiiBackupManagerPlus-v1.0.0-linux-i686.AppImage` |
| Linux, ARM64 | `TinyWiiBackupManagerPlus-v1.0.0-linux-aarch64.AppImage` |
| macOS, Intel and Apple Silicon | `TinyWiiBackupManagerPlus-v1.0.0-macos-universal.dmg` |

Windows executables do not need an installer. On Linux, run `chmod +x TinyWiiBackupManagerPlus-*.AppImage` before opening the downloaded AppImage. The accompanying `.zsync` files are update metadata, not separate applications.

On macOS, drag the application into Applications. The app is ad-hoc signed, not notarized. If macOS blocks the verified download, remove its quarantine attribute in Terminal:

```sh
xattr -rd com.apple.quarantine /Applications/TinyWiiBackupManagerPlus.app
```

`SHA256SUMS` contains checksums for the assets. Compare your download with its corresponding entry before running it.

## Moving from an earlier build

**Download this release manually if you use an earlier 6.x build.** The version number restarts at 1.0.0 for this fork; an older executable's version comparison will not identify it as a newer release. Future Plus 1.x builds check this fork for updates.

Plus uses its own data directory. When no Plus configuration exists, it copies the previous application's `config.json` without changing the original. Existing Plus settings are not overwritten. Cached artwork/catalog data may be downloaded again. Portable mode uses `TinyWiiBackupManagerPlus-data` beside the executable and looks for previous settings in a sibling `TinyWiiBackupManager-data` directory.

## Validation scope

The release pipeline builds all eight architecture targets and packages the two macOS binaries into one Universal application. Tests and strict Clippy run on Windows modern/legacy, Linux, and macOS configurations. Packaging checks cover executable architectures, application metadata and checksums.

Build and automated test success do not establish runtime compatibility on every supported OS version or device. Older Windows versions, all ARM devices, and Wii console boot/real Wiiload transmission are not covered by a full hardware test matrix. Fast transfer is optional; no universal speedup is promised. Unknown Redump hashes are reported separately from read errors and do not by themselves prove a damaged dump.
