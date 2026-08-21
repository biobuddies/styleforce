"""Test bundled GritQL patterns."""

from __future__ import annotations


def test_patterns() -> None:
    """Run every pattern's Markdown samples through the native GritQL runner.

    The ``styleforce`` wheel bundles its ``.grit`` patterns as package data,
    so ``styleforce.test_patterns()`` resolves them from the installed
    location — no source-tree copy is needed.
    """
    import styleforce  # noqa: PLC0415

    result = styleforce.test_patterns()

    assert result['passed'], result.get('summary', 'pattern tests failed')
