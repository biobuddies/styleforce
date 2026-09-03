"""styleforce -- shared GritQL rules for enforcing source-code style.

The ``.grit`` patterns and a native GritQL engine (``styleforce._native``, a
PyO3 build of the vendored marzano crates) ship inside this package, so the
wheel applies patterns out of the box. :func:`apply` rewrites one snippet by a
pattern; the repository's pytest suite drives it over each pattern's Markdown
samples.
"""

from __future__ import annotations

__all__ = ['apply']


def apply(pattern: str, source: str, filename: str = 'snippet.py') -> str:
    """Rewrite *source* by the GritQL *pattern* body, returning the new source.

    *source* is returned unchanged when the pattern matches nothing. *filename*
    names the snippet for the engine; the pattern's own ``language`` line, not
    the extension, selects the grammar.
    """
    import styleforce._native as native  # noqa: PLC0415

    return native.apply(pattern, source, filename)
