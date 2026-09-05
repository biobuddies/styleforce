"""Apply each GritQL Markdown sample through the bundled native engine."""

from __future__ import annotations

from subprocess import PIPE, run

import styleforce


def _ruff(source: str) -> str:
    return run(['ruff', 'format', '-'], check=True, input=source, stdout=PIPE, text=True).stdout


def test_sample(sample: tuple[str, str, str]) -> None:
    pattern, before, after = sample
    actual = styleforce.apply(pattern, before)
    assert _ruff(actual) == _ruff(after), actual
