"""
Rewrite maturin wheel metadata so PyPI accepts the hadolint-py git dependency.

PyPI rejects `Requires-Dist` with a URL but allows `Requires-External`:
https://packaging.python.org/specifications/core-metadata
"""

from base64 import urlsafe_b64encode
from hashlib import sha256
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile

for wheel in Path('dist').glob('styleforce-*.whl'):
    distribution, version = wheel.name.split('-')[:2]
    dist_info = f'{distribution}-{version}.dist-info'
    metadata = f'{dist_info}/METADATA'
    record = f'{dist_info}/RECORD'
    with ZipFile(wheel) as archive:
        contents = {member: archive.read(member) for member in archive.namelist()}
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
        for member, data in contents.items():
            archive.writestr(member, data)
