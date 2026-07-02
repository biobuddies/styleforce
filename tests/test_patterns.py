from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).parents[1]


def test_inline_single_use_assignment(tmp_path: Path) -> None:
    """Delegate the Markdown input/output samples to Grit's native test runner."""
    grit_name = "grit.exe" if sys.platform == "win32" else "grit"
    environment_grit = Path(sys.executable).with_name(grit_name)
    grit = environment_grit if environment_grit.is_file() else shutil.which(grit_name)
    assert grit is not None, "install styleforce into the test environment first"
    shutil.copytree(ROOT / ".grit", tmp_path / ".grit")

    result = subprocess.run(
        [
            grit,
            "patterns",
            "test",
            "--filter=inline_single_use_assignment",
            "--verbose",
        ],
        cwd=tmp_path,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, result.stdout + result.stderr
