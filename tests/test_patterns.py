"""Check every bundled GritQL pattern's Markdown samples through the grit CLI."""

from __future__ import annotations


def test_sample(sample_state: str) -> None:
    assert sample_state in {'pass', 'passWithFormat'}, sample_state
