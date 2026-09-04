"""Auto-generate one pytest case per GritQL Markdown sample, run by the wheel.

Every ``styleforce/.grit/patterns/*.md`` pairs a ```grit`` pattern body with
``## `` sample sections: two fenced blocks for a before/after rewrite, one for
an input that must stay unchanged. This reads those sections and parametrizes a
``(pattern, before, after)`` case apiece; :mod:`tests.test_patterns` applies
each through the bundled native engine (``styleforce.apply``).
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import styleforce

if TYPE_CHECKING:
    from pytest import Metafunc

_PATTERNS = Path(styleforce.__file__).parent / '.grit' / 'patterns'


def _parse(markdown: str) -> list[tuple[str, tuple[str, str, str]]]:
    pattern, title, language, buffer, fences, sections = '', None, None, [], [], []
    for line in markdown.splitlines():
        if language is None and line.startswith('```'):
            language, buffer = line[3:].strip(), []
        elif language is not None and line.startswith('```'):
            code = '\n'.join(buffer) + '\n'
            if language == 'grit':
                pattern = code
            elif title is not None:
                fences.append(code)
            language = None
        elif language is not None:
            buffer.append(line)
        elif line.startswith('## '):
            title, fences = line[3:].strip(), []
            sections.append((title, fences))
    return [(title, (pattern, blocks[0], blocks[-1])) for title, blocks in sections if blocks]


def pytest_generate_tests(metafunc: Metafunc) -> None:
    if 'sample' not in metafunc.fixturenames:
        return
    arguments, identifiers = [], []
    for markdown in sorted(_PATTERNS.glob('*.md')):
        for title, sample in _parse(markdown.read_text()):
            arguments.append(sample)
            identifiers.append(f'{markdown.stem}::{title}')
    metafunc.parametrize('sample', arguments, ids=identifiers)
