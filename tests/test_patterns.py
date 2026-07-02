from __future__ import annotations

import shutil
import sys
from pathlib import Path
from subprocess import STDOUT, check_output


ROOT = Path(__file__).parents[1]


def test_inline_single_use_assignment(tmp_path: Path) -> None:
    """Delegate the Markdown input/output samples to Grit's native test runner."""
    environment_grit = Path(sys.executable).with_name("grit")
    grit = environment_grit if environment_grit.is_file() else shutil.which("grit")
    assert grit is not None, "install styleforce into the test environment first"
    shutil.copytree(ROOT / ".grit", tmp_path / ".grit")

    specification = ROOT / ".grit/patterns/inline_single_use_assignment.md"
    print("\nGritQL native test specification")
    print("Two Python blocks: input, then expected output.")
    print("One Python block: input expected to produce no rewrite.")
    print(specification.read_text())

    result = check_output(
        [
            grit,
            "patterns",
            "test",
            "--filter=inline_single_use_assignment",
            "--verbose",
        ],
        cwd=tmp_path,
        stderr=STDOUT,
        text=True,
    )

    print("GritQL native test results")
    print(result, end="")
