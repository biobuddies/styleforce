"""Apply each GritQL Markdown sample through the bundled native engine."""

from __future__ import annotations

from re import escape
from subprocess import PIPE, run

from pytest import raises

import styleforce


def _ruff(source: str) -> str:
    return run(['ruff', 'format', '-'], check=True, input=source, stdout=PIPE, text=True).stdout


def test_sample(sample: tuple[str, str, str, str]) -> None:
    pattern, before, after, filename = sample
    if after.startswith('RuntimeError: '):
        with raises(RuntimeError, match=escape(after.removeprefix('RuntimeError: ').strip())):
            styleforce.apply(pattern, before, filename)
    else:
        actual = styleforce.apply(pattern, before, filename)
        assert _ruff(actual) == _ruff(after), actual
