"""styleforce — shared GritQL rules for enforcing source-code style.

The native pattern-testing engine lives in ``styleforce._native``, a PyO3
extension built from ``rust/styleforce_py``. This package stub exists so that
maturin can place the extension module under the ``styleforce`` namespace and
so that the ``.grit`` pattern data files can be shipped as package data.

TODO: decide whether to bundle the ``.grit`` patterns as package data here
(e.g. via ``include_package_data`` or a ``[tool.maturin] data`` entry) or
keep them as a separate wheel data directory as the old ``build_backend.py``
did.
"""
