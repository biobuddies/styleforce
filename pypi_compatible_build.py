"""
Rewrite maturin wheel metadata so PyPI accepts the hadolint-py git dependency.

PyPI rejects `Requires-Dist` with a URL but allows `Requires-External`:
https://packaging.python.org/specifications/core-metadata
"""

from base64 import urlsafe_b64encode
from hashlib import sha256
from sys import argv
from zipfile import ZIP_DEFLATED, ZipFile


def make_pypi_compatible(wheel: str) -> None:
    contents = {}
    metadata = record = ''
    with ZipFile(wheel) as archive:
        for name in archive.namelist():
            contents[name] = archive.read(name)
            if name.endswith('.dist-info/METADATA'):
                metadata = name
            elif name.endswith('.dist-info/RECORD'):
                record = name
    contents[metadata] = ''.join(
        line.replace('Requires-Dist:', 'Requires-External:', 1)
        if line.startswith('Requires-Dist:') and '://' in line
        else line
        for line in contents[metadata].decode().splitlines(keepends=True)
    ).encode()
    digest = urlsafe_b64encode(sha256(contents[metadata]).digest()).rstrip(b'=').decode()
    contents[record] = ''.join(
        f'{metadata},sha256={digest},{len(contents[metadata])}\n'
        if line.startswith(f'{metadata},')
        else line
        for line in contents[record].decode().splitlines(keepends=True)
    ).encode()
    with ZipFile(wheel, 'w', ZIP_DEFLATED) as archive:
        for name, data in contents.items():
            archive.writestr(name, data)


for wheel in argv[1:]:
    make_pypi_compatible(wheel)
