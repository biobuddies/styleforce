"""Copy the pinned upstream GritQL assets into this repository's own releases.

Claude Code on the web blocks upstream getgrit downloads but allows these mirrored releases;
build_backend then fetches grit from here.
"""

from __future__ import annotations

import hashlib
import urllib.request
from pathlib import Path
from subprocess import DEVNULL, CalledProcessError, check_call, check_output

import build_backend as backend


def mirror() -> None:
    tag = f"grit-{backend.GRIT_VERSION}"
    try:
        check_output(["gh", "release", "view", tag], stderr=DEVNULL)
        print(f"{tag} already mirrored")
        return
    except CalledProcessError:
        pass

    assets = []
    for target, expected in backend.ARCHIVE_SHA256.items():
        asset = f"grit-{target}.tar.gz"
        urllib.request.urlretrieve(f"{backend.UPSTREAM_URL}/{asset}", asset)
        actual = hashlib.sha256(Path(asset).read_bytes()).hexdigest()
        if actual != expected:
            raise RuntimeError(f"checksum mismatch for {asset}: {actual} != {expected}")
        assets.append(asset)

    check_call(
        ["gh", "release", "create", tag, *assets,
         "--title", tag, "--notes", f"Mirror of getgrit/gritql {backend.GRIT_VERSION}."]
    )


if __name__ == "__main__":
    mirror()
