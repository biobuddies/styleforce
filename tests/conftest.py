"""Turn each GritQL Markdown sample into its own pytest case, judged by the grit CLI.

Every ``styleforce/.grit/patterns/*.md`` pairs a pattern with ``## `` sample
sections: two fenced blocks for a before/after rewrite, one for an input that
must stay unchanged. The upstream ``grit patterns test`` runner already does
the matching and language formatting, so here pytest only reads the sections to
name a case apiece and asserts that section's verdict -- no bundled Rust tester.
"""

from __future__ import annotations

from functools import cache
from json import loads
from pathlib import Path
from subprocess import PIPE, STDOUT, run

from pytest import Metafunc, fixture

_GRIT_ROOT = Path(__file__).resolve().parent.parent / 'styleforce'
_PATTERNS = _GRIT_ROOT / '.grit' / 'patterns'


def _sample_titles(markdown: str) -> list[str]:
    titles = []
    fenced = False
    for line in markdown.splitlines():
        if line.startswith('```'):
            fenced = not fenced
        elif not fenced and line.startswith('## '):
            titles.append(line[3:].strip())
    return titles


@cache
def _sample_states(pattern: str) -> tuple[str, ...]:
    completed = run(
        ['grit', 'patterns', 'test', '--filter', f'^{pattern}$', '--json'],
        check=False,
        cwd=_GRIT_ROOT,
        stderr=STDOUT,
        stdout=PIPE,
        text=True,
    )
    results = loads(completed.stdout[completed.stdout.index('[') :])
    return tuple(sample['state'] for sample in results[0]['samples'])


def pytest_generate_tests(metafunc: Metafunc) -> None:
    if 'sample' not in metafunc.fixturenames:
        return
    arguments, identifiers = [], []
    for pattern in sorted(_PATTERNS.glob('*.md')):
        for index, title in enumerate(_sample_titles(pattern.read_text())):
            arguments.append((pattern.stem, index))
            identifiers.append(f'{pattern.stem}::{title}')
    metafunc.parametrize('sample', arguments, ids=identifiers)


@fixture
def sample_state(sample: tuple[str, int]) -> str:
    return _sample_states(sample[0])[sample[1]]
