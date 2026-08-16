from __future__ import annotations

import json
import shutil
from pathlib import Path


ROOT = Path(__file__).parents[1]


def test_patterns(tmp_path: Path) -> None:
    """Run every pattern's Markdown samples through the native GritQL test runner.

    Instead of shelling out to a downloaded ``grit`` CLI binary, we call the
    ``styleforce._native`` PyO3 extension (built from ``rust/styleforce_py``)
    which compiles the GritQL crates directly and exposes a ``test_patterns``
    function mirroring ``grit patterns test --verbose``.
    """
    # TODO: once the wheel bundles the .grit patterns as package data, we can
    # point at the installed location instead of copying from the source tree.
    shutil.copytree(ROOT / '.grit', tmp_path / '.grit')

    import styleforce._native as native  # noqa: PLC0415

    result = native.test_patterns(str(tmp_path))

    # The native function returns a dict with: passed, total_patterns,
    # total_samples, failed_samples, patterns[], summary.
    print('GritQL native test results')
    print(json.dumps(result, indent=2))

    assert result['passed'], result.get('summary', 'pattern tests failed')
