"""
Rewrite maturin wheel metadata so PyPI accepts the hadolint-py git dependency.

PyPI rejects `Requires-Dist` with a URL but allows `Requires-External`:
https://packaging.python.org/specifications/core-metadata
"""

from base64 import urlsafe_b64encode
from hashlib import sha256
from pathlib import Path
from re import findall
from shutil import copyfileobj
from zipfile import ZIP_DEFLATED, ZipFile

for wheel in Path('dist').glob('styleforce-*.whl'):
    record, metadata = (
        '.dist-info/'.join((findall(r'[^-]+-[^-]+', wheel.name)[0], name))
        for name in ('RECORD', 'METADATA')
    )
    repaired = wheel.with_name(f'{wheel.name}.tmp')
    with ZipFile(wheel) as source, ZipFile(repaired, 'w', ZIP_DEFLATED) as target:
        compatible = ''.join(
            line.replace('Requires-Dist:', 'Requires-External:', 1)
            if line.startswith('Requires-Dist:') and '://' in line
            else line
            for line in source.read(metadata).decode().splitlines(keepends=True)
        ).encode()
        digest = urlsafe_b64encode(sha256(compatible).digest()).rstrip(b'=').decode()
        for info in source.infolist():
            if info.filename == metadata:
                target.writestr(info, compatible)
            elif info.filename == record:
                target.writestr(
                    info,
                    ''.join(
                        f'{metadata},sha256={digest},{len(compatible)}\n'
                        if line.startswith(f'{metadata},')
                        else line
                        for line in source.read(record).decode().splitlines(keepends=True)
                    ).encode(),
                )
            else:
                with source.open(info) as auxiliary, target.open(info, 'w') as copy:
                    copyfileobj(auxiliary, copy)
    repaired.replace(wheel)
