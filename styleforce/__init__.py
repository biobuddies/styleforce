"""Autoformat the Rule of Three and more."""

from __future__ import annotations

import sys
from argparse import ArgumentParser
from functools import reduce
from pathlib import Path
from typing import TYPE_CHECKING

from styleforce._native import apply

if TYPE_CHECKING:
    from collections.abc import Sequence

__all__ = ['apply', 'main']


def main(arguments: Sequence[str] | None = None) -> None:
    parser = ArgumentParser(description=__doc__)
    parser.add_argument('filenames', nargs='+', type=Path)
    bundled_patterns = tuple(
        markdown.read_text().partition('```grit\n')[2].partition('```')[0]
        for markdown in sorted((Path(__file__).parent / '.grit' / 'patterns').rglob('*.md'))
    )
    reformatted = []
    for filename in parser.parse_args(arguments).filenames:
        source = filename.read_text()
        formatted = reduce(
            lambda current, pattern: apply(pattern, current, str(filename)),
            bundled_patterns,
            source,
        )
        if formatted != source:
            filename.write_text(formatted)
            reformatted.append(filename)
    if reformatted:
        sys.stdout.write(
            f'Reformatted {len(reformatted)} files:\n' + '\n'.join(map(str, reformatted)) + '\n'
        )
