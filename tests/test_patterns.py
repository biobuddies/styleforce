from __future__ import annotations

import shutil
import sys
from pathlib import Path
from subprocess import PIPE, STDOUT, run


ROOT = Path(__file__).parents[1]


def test_patterns(tmp_path: Path) -> None:
    """Delegate every pattern's Markdown input/output samples to Grit's native test runner."""
    environment_grit = Path(sys.executable).with_name('grit')
    grit = environment_grit if environment_grit.is_file() else shutil.which('grit')
    assert grit is not None, 'install styleforce into the test environment first'
    shutil.copytree(ROOT / '.grit', tmp_path / '.grit')

    result = run(
        [grit, 'patterns', 'test', '--verbose'],
        cwd=tmp_path,
        stdout=PIPE,
        stderr=STDOUT,
        text=True,
    )

    print('GritQL native test results')
    print(result.stdout, end='')
    assert result.returncode == 0, result.stdout
