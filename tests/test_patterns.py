from __future__ import annotations

import json


def test_patterns() -> None:
    """Run every pattern's Markdown samples through the native GritQL runner.

    The ``styleforce`` wheel bundles its ``.grit`` patterns as package data,
    so ``styleforce.test_patterns()`` resolves them from the installed
    location — no source-tree copy is needed.
    """
    import styleforce  # noqa: PLC0415

    result = styleforce.test_patterns()

    # The native function returns a dict with: passed, total_patterns,
    # total_samples, failed_samples, patterns[], summary.
    print('GritQL native test results')
    print(json.dumps(result, indent=2))

    assert result['passed'], result.get('summary', 'pattern tests failed')
