<img alt="TinyWiiBackupManagerPlus Logo" width="128" src="assets/TinyWiiBackupManager-256x256.png" align="left">

### `TinyWiiBackupManagerPlus`<br><sub><sup>:star: A tiny game backup and homebrew app manager for the Wii</sup></sub>

[![license: GPL-3.0](https://img.shields.io/github/license/communism420/TinyWiiBackupManagerPlus)](https://github.com/communism420/TinyWiiBackupManagerPlus/blob/main/COPYING)

TinyWiiBackupManagerPlus is a fork of [TinyWiiBackupManager](https://github.com/mq1/TinyWiiBackupManager), originally developed by Manuel Quarneti and its contributors. We welcome useful, safe changes that preserve existing functionality, whether made with AI or without it. What matters is the benefit and quality of the result. See our [contribution policy](AI_POLICY.md).

<br>

> [!CAUTION]
> TinyWiiBackupManagerPlus is intended strictly for legal homebrew use and is not affiliated with or endorsed by Nintendo.
> Use of TinyWiiBackupManagerPlus for pirated or unauthorized copies of games is strictly prohibited.

<img align="center" alt="App Screenshot" src="assets/screenshot.png">

## :sparkles: Features

- **Lightweight & Fast**: Native app written in Rust and Iced, no Electron!
- **Cross-Platform**:
  - :window: Windows | x86: XP+, x64: Vista+, ARM64: Windows 10+
  - :apple: macOS | Intel: 10.12+, Apple Silicon: 11+
  - :penguin: Linux ~2020+ | x86 (32-bit), x86_64 (64-bit), arm64 (Raspberry PIs etc.)

#### :video_game: Game Management

- **Games view**: Manage your Wii and GameCube games
- **Fast transfer**: An optional shared setting for imports and exports, available in Settings, the import queue, batch export, and game details. It uses extra memory and may improve speed depending on the drive, while keeping the chosen format and game data unchanged. It is off by default and does not change the number of simultaneous games.
- **Batch and parallel import**: Add game files or a folder, select individual games or **Select all** in the import queue, and choose **Import selected**. Choose 1–4 **Simultaneous games** (2 by default); 1 imports sequentially. Each batch uses the destination and settings captured when it starts. Newly added games wait for the next batch, and failed games remain in the queue for retry.
- **Cancel import**: Use **Cancel import** in the import queue or its progress notification, including after navigating to another page. Cancellation stops new jobs and waits for running work and temporary-file cleanup. Completed games stay on the drive; cancelled and pending entries remain selected for retry. A current disk operation or compression block may need to finish first.
- **Parallel export**: Use the Games toolbar's **Export games** button to select individual games or **Select all**, choose an output format, and export to one folder. Choose 1–4 **Simultaneous games** (2 by default); more jobs may be slower on a USB drive. Each export writes to its own temporary file before publishing the completed result. Existing files are kept; failures are reported without stopping the remaining exports.
- **Individual export**: On Windows, choose RVZ, ISO, WBFS, CISO, GCZ or WIA from **Save as type** when exporting a single game. The filename extension follows the selected format.
- **Cancel export**: Cancel a single, sequential or parallel export from its progress notification. Cancellation stops new jobs and waits for running work and temporary-file cleanup before allowing another export. Completed files are kept; unfinished files are removed. A current disk operation or compression block may need to finish first.
- **Format Support**: .iso, .rvz and major formats thanks to [NOD](https://github.com/encounter/nod)
- **Automatic Splitting**: .wbfs file splitting when needed
- **Partition Stripping**: Remove the update partition to save space
- **Game Archiving**: Archive games using RVZ+zstd-19
- **Integrity Checks**: Use **Check SHA1** in the Games toolbar to select individual games or **Select all** and compare their disc hashes with the bundled Redump database. Results stay available after closing the window; an unknown hash does not necessarily mean corruption.
- **GameTDB**: Fetch covers and `wiitdb.xml` from GameTDB
- **TxtCodes**: Download cheat codes from geckocodes.org (web archive), codes.rc24.xyz and gamehacking.org

#### :toolbox: Wii Homebrew Management

- **Apps view**: Manage Wii homebrew applications
- **OSC view**: Download apps from the Open Shop Channel
- **Wiiload**: Send apps directly to Wii via network

<br>

## :arrow_down: Downloads

Download **TinyWiiBackupManagerPlus 1.0.0** from the [releases page](https://github.com/communism420/TinyWiiBackupManagerPlus/releases/latest).

| System | Download |
| --- | --- |
| Windows, most Intel/AMD PCs | `TinyWiiBackupManagerPlus-v1.0.0-windows-x64.exe` |
| Windows, 32-bit / legacy systems | `TinyWiiBackupManagerPlus-v1.0.0-windows-x86.exe` |
| Windows on ARM | `TinyWiiBackupManagerPlus-v1.0.0-windows-arm64.exe` |
| Linux, most Intel/AMD PCs | `TinyWiiBackupManagerPlus-v1.0.0-linux-x86_64.AppImage` |
| Linux, 32-bit Intel/AMD | `TinyWiiBackupManagerPlus-v1.0.0-linux-i686.AppImage` |
| Linux, ARM64 | `TinyWiiBackupManagerPlus-v1.0.0-linux-aarch64.AppImage` |
| macOS, Intel and Apple Silicon | `TinyWiiBackupManagerPlus-v1.0.0-macos-universal.dmg` |

Windows executables run without an installer. On Linux, make the AppImage executable before launching it (`chmod +x TinyWiiBackupManagerPlus-*.AppImage`). On macOS, drag TinyWiiBackupManagerPlus into Applications; the app is ad-hoc signed and is not notarized. See the release notes for installation details and verification limits. `SHA256SUMS` accompanies the downloadable files.

**Moving from an earlier 6.x build:** download 1.0.0 manually. This fork starts its own version sequence, so an older 6.x executable will not offer 1.0.0 as a newer version. Plus uses a separate data directory and copies the previous `config.json` only when its own configuration does not exist. The original configuration is preserved; caches are rebuilt as needed. Portable mode uses `TinyWiiBackupManagerPlus-data` beside the executable and can import settings from a sibling `TinyWiiBackupManager-data` directory.

The original project's [Flathub package](https://flathub.org/apps/it.mq1.TinyWiiBackupManager) and `winget install -e --id mq1.TinyWiiBackupManager` install the original application and do not include this fork's changes.

<br>

## :page_facing_up: Additional Info

For useful tips, check out the [original project Wiki](https://github.com/mq1/TinyWiiBackupManager/wiki)

<br>
<br>

<p align="center"> Made with :white_heart::pink_heart::light_blue_heart::brown_heart::black_heart::heart::orange_heart::yellow_heart::green_heart::blue_heart::purple_heart: for the Wii homebrew community </p>
