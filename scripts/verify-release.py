#!/usr/bin/env python3
"""Check release file headers and the complete asset set using only the standard library."""

import argparse
import hashlib
from pathlib import Path
import struct


NAME = "TinyWiiBackupManagerPlus"


def verify_binary(path, platform, arch):
    with path.open("rb") as stream:
        header = stream.read(64)
        if platform == "windows":
            if header[:2] != b"MZ":
                raise ValueError(f"{path}: missing DOS header")
            stream.seek(struct.unpack_from("<I", header, 60)[0])
            pe = stream.read(6)
            expected = {"x86": 0x014C, "x64": 0x8664, "arm64": 0xAA64}[arch]
            if pe[:4] != b"PE\0\0" or struct.unpack_from("<H", pe, 4)[0] != expected:
                raise ValueError(f"{path}: expected Windows {arch} executable")
        elif platform == "linux":
            expected = {"i686": (1, 3), "x86_64": (2, 62), "aarch64": (2, 183)}[arch]
            if (
                header[:4] != b"\x7fELF"
                or header[5] != 1
                or (header[4], struct.unpack_from("<H", header, 18)[0]) != expected
            ):
                raise ValueError(f"{path}: expected Linux {arch} executable")
        else:
            raise ValueError(f"Unsupported platform: {platform}")
    print(f"Verified {platform}/{arch}: {path}")


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verify_assets(directory, version, tag, write_checksums):
    if tag != f"v{version}":
        raise ValueError(f"Tag {tag!r} does not match version {version!r}")
    expected = {"version.txt", f"{NAME}-{tag}-macos-universal.dmg"}
    for arch in ("x86", "x64", "arm64"):
        expected.add(f"{NAME}-{tag}-windows-{arch}.exe")
    for arch in ("i686", "x86_64", "aarch64"):
        filename = f"{NAME}-{tag}-linux-{arch}.AppImage"
        expected.update((filename, f"{filename}.zsync"))
    actual = {path.name for path in directory.iterdir()}
    if actual - {"SHA256SUMS"} != expected:
        raise ValueError(
            f"Unexpected asset set: missing={sorted(expected - actual)}, "
            f"extra={sorted(actual - expected - {'SHA256SUMS'})}"
        )
    for filename in expected:
        path = directory / filename
        if not path.is_file() or path.stat().st_size == 0:
            raise ValueError(f"Empty or non-file asset: {path}")
    if (directory / "version.txt").read_text(encoding="utf-8").strip() != version:
        raise ValueError("version.txt does not match Cargo version")
    for arch in ("x86", "x64", "arm64"):
        verify_binary(directory / f"{NAME}-{tag}-windows-{arch}.exe", "windows", arch)
    for arch in ("i686", "x86_64", "aarch64"):
        filename = f"{NAME}-{tag}-linux-{arch}.AppImage"
        path = directory / filename
        verify_binary(path, "linux", arch)
        with path.open("rb") as stream:
            stream.seek(8)
            if stream.read(3) != b"AI\x02":
                raise ValueError(f"{filename}: expected a type 2 AppImage")
        with (directory / f"{filename}.zsync").open("rb") as stream:
            zsync_header = stream.read(8192).split(b"\n\n", 1)[0].decode("utf-8")
        if f"Filename: {filename}" not in zsync_header.splitlines():
            raise ValueError(f"{filename}.zsync names an unexpected AppImage")
    with (directory / f"{NAME}-{tag}-macos-universal.dmg").open("rb") as stream:
        stream.seek(-512, 2)
        if stream.read(4) != b"koly":
            raise ValueError("DMG does not have a UDIF trailer")
    checksums = "".join(f"{sha256(directory / name)}  {name}\n" for name in sorted(expected))
    checksum_path = directory / "SHA256SUMS"
    if write_checksums:
        checksum_path.write_text(checksums, encoding="utf-8", newline="\n")
    elif checksum_path.read_text(encoding="utf-8") != checksums:
        raise ValueError("SHA256SUMS does not match the release assets")
    print(f"Verified {len(expected)} release assets and SHA256SUMS")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    binary = commands.add_parser("binary")
    binary.add_argument("path", type=Path)
    binary.add_argument("platform", choices=("windows", "linux"))
    binary.add_argument("arch")
    assets = commands.add_parser("assets")
    assets.add_argument("directory", type=Path)
    assets.add_argument("--version", required=True)
    assets.add_argument("--tag", required=True)
    assets.add_argument("--write-checksums", action="store_true")
    args = parser.parse_args()
    if args.command == "binary":
        verify_binary(args.path, args.platform, args.arch)
    else:
        verify_assets(args.directory, args.version, args.tag, args.write_checksums)


if __name__ == "__main__":
    main()
