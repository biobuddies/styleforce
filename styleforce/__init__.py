"""styleforce — shared GritQL rules for enforcing source-code style.

The native pattern-testing engine lives in ``styleforce._native``, a PyO3
extension built from ``styleforce/rust``. The ``.grit`` pattern data files
ship inside this package so the wheel is usable out-of-the-box with no
checkout of the source tree required.
"""

from __future__ import annotations

from pathlib import Path

__all__ = ['test_patterns']

_PACKAGE_DIR = Path(__file__).resolve().parent


def test_patterns(cwd: str | None = None) -> dict:
    """Run every pattern's Markdown samples through the native GritQL runner.

    By default the patterns bundled in this package (``styleforce/.grit``)
    are tested. Pass *cwd* to point at a different working directory whose
    ``.grit`` tree should be used instead.
    """
    import styleforce._native as native  # noqa: PLC0415

    target = cwd if cwd is not None else str(_PACKAGE_DIR)
    return native.test_patterns(target)
