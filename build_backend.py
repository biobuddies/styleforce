"""Build platform wheels containing the upstream GritQL CLI."""

from __future__ import annotations

import base64
import csv
import hashlib
import io
import os
import platform
import shutil
import stat
import sys
import tarfile
import tempfile
import urllib.request
import zipfile
from pathlib import Path


NAME = "styleforce"
VERSION = "0.1.0"
DIST_INFO = f"{NAME}-{VERSION}.dist-info"
GRIT_VERSION = "v0.1.0-alpha.1743007075"
RELEASE_URL = (
    "https://github.com/getgrit/gritql/releases/download/"
    f"{GRIT_VERSION}"
)
ARCHIVE_SHA256 = {
    "aarch64-apple-darwin": "7ab8c7eea90799ae35c86f2a9b7e48e56b91a62e6a459b910dde1a3daa066bf3",
    "x86_64-apple-darwin": "b502f031cfe72b58e193282faf53531a5aac01c7bfa779421fc52652b652010e",
    "aarch64-unknown-linux-gnu": "8e37415c45595716386d018f4d279a78f80261a7c7592c37632e7ce7d0934870",
    "x86_64-unknown-linux-gnu": "94b34641a538ca0e85a92aa7f0ac94077fc6d663c996d0556c781d3d4c163149",
}


def _target() -> tuple[str, str]:
    machine = platform.machine().lower()
    arch = "aarch64" if machine in {"arm64", "aarch64"} else "x86_64"

    if sys.platform == "darwin":
        tag = "macosx_11_0_arm64" if arch == "aarch64" else "macosx_10_9_x86_64"
        return f"{arch}-apple-darwin", tag
    if sys.platform == "linux":
        tag = (
            "manylinux_2_17_aarch64.manylinux2014_aarch64"
            if arch == "aarch64"
            else "manylinux_2_17_x86_64.manylinux2014_x86_64"
        )
        return f"{arch}-unknown-linux-gnu", tag
    raise RuntimeError(f"GritQL has no supported binary for {sys.platform}/{machine}")


def _download_grit(destination: Path) -> str:
    target, wheel_platform = _target()
    asset = f"grit-{target}.tar.gz"
    url = f"{RELEASE_URL}/{asset}"
    archive = destination / asset

    urllib.request.urlretrieve(url, archive)
    expected = ARCHIVE_SHA256[target]
    actual = hashlib.sha256(archive.read_bytes()).hexdigest()
    if actual != expected:
        raise RuntimeError(f"checksum mismatch for {asset}: {actual} != {expected}")

    unpacked = destination / "unpacked"
    unpacked.mkdir()
    with tarfile.open(archive, "r:gz") as bundle:
        if sys.version_info >= (3, 12):
            bundle.extractall(unpacked, filter="data")
        else:
            for member in bundle.getmembers():
                resolved = (unpacked / member.name).resolve()
                if unpacked.resolve() not in resolved.parents and resolved != unpacked.resolve():
                    raise RuntimeError(f"unsafe archive member: {member.name}")
            bundle.extractall(unpacked)

    matches = list(unpacked.rglob("grit"))
    if len(matches) != 1:
        raise RuntimeError(f"expected one 'grit' in {asset}, found {len(matches)}")
    shutil.copyfile(matches[0], destination / "grit")
    os.chmod(destination / "grit", 0o755)
    return wheel_platform


def _metadata() -> bytes:
    return (
        "Metadata-Version: 2.4\n"
        f"Name: {NAME}\n"
        f"Version: {VERSION}\n"
        "Summary: Shared GritQL rules for enforcing source-code style.\n"
        "Requires-Python: >=3.9\n"
    ).encode()


def _digest(data: bytes) -> str:
    value = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=")
    return f"sha256={value.decode()}"


def build_wheel(wheel_directory: str, config_settings=None, metadata_directory=None) -> str:
    del config_settings, metadata_directory
    with tempfile.TemporaryDirectory() as temporary:
        staging = Path(temporary)
        wheel_platform = _download_grit(staging)
        executable = "grit"
        filename = f"{NAME}-{VERSION}-py3-none-{wheel_platform}.whl"
        output = Path(wheel_directory) / filename
        output.parent.mkdir(parents=True, exist_ok=True)

        files: dict[str, bytes] = {
            f"{NAME}-{VERSION}.data/scripts/{executable}": (staging / executable).read_bytes(),
            f"{NAME}-{VERSION}.data/data/share/{NAME}/.grit/grit.yaml": Path(".grit/grit.yaml").read_bytes(),
            f"{NAME}-{VERSION}.data/data/share/{NAME}/.grit/patterns/inline_single_use_assignment.md": Path(
                ".grit/patterns/inline_single_use_assignment.md"
            ).read_bytes(),
            f"{DIST_INFO}/METADATA": _metadata(),
            f"{DIST_INFO}/WHEEL": (
                "Wheel-Version: 1.0\n"
                "Generator: styleforce.build_backend\n"
                "Root-Is-Purelib: false\n"
                f"Tag: py3-none-{wheel_platform}\n"
            ).encode(),
        }
        record = io.StringIO()
        writer = csv.writer(record, lineterminator="\n")
        for path, data in files.items():
            writer.writerow((path, _digest(data), len(data)))
        writer.writerow((f"{DIST_INFO}/RECORD", "", ""))
        files[f"{DIST_INFO}/RECORD"] = record.getvalue().encode()

        with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as wheel:
            for path, data in files.items():
                info = zipfile.ZipInfo(path)
                info.create_system = 3
                info.compress_type = zipfile.ZIP_DEFLATED
                mode = 0o755 if path.endswith(executable) else 0o644
                info.external_attr = (stat.S_IFREG | mode) << 16
                wheel.writestr(info, data)
        return filename


def prepare_metadata_for_build_wheel(metadata_directory: str, config_settings=None) -> str:
    del config_settings
    dist_info = Path(metadata_directory) / DIST_INFO
    dist_info.mkdir(parents=True, exist_ok=True)
    (dist_info / "METADATA").write_bytes(_metadata())
    return DIST_INFO
