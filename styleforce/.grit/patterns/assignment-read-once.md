---
title: Inline single-use assignment
---

Inline Python assignments that are read exactly once.

Considerations:
* Does the right hand side make system calls like:
    - `subprocess.check_call()`
    - `file.read()`
    - `file.seek(0, SEEK_END)`
    - `time.monotonic()`
    - `tempfile.TemporaryDirectory()`
* Are the reads static or dynamic?

The order and number of side effects must remain unchanged.

```grit
engine marzano(0.1)
language python

`$use_statement` as $use where {
  $use <: after `$variable = $value
` as $assignment,
  $use <: contains `$variable`,
  $use <: within module(statements=$statements),
  $statements <: not some $other where {
    $other <: contains `$variable`,
    $other <: not $use,
    $other <: not `$variable = $value`
  },
  $use_statement <: contains bubble($variable, $value) `$variable` => $value,
  $assignment <: `$name = $value`,
  $use => `$use_statement  # $name`,
  $assignment => .
}
```

## Two assignments read once

```python
wells = ('A01', 'B02')
print(wells)

T8M_90964_c23CT = 'GGCCGAAGGAGACGCTGCAGT'
print(T8M_90964_c23CT)
```

```python
print(('A01', 'B02'))  # wells

print('GGCCGAAGGAGACGCTGCAGT')  # T8M_90964_c23CT
```

## Read from loop
```python
from os import environ
from pathlib import Path
from subprocess import check_output
from sys import argv

from jinja2 import Environment

if len(argv) != 2:
    raise SystemExit('Usage: mise render ENVI')

ENVI = argv[1]
public = Path('public')
environment = Environment(autoescape=False, keep_trailing_newline=True)
table = check_output(
    ['mise', 'tabularize'], env={**environ, 'ENVI': ENVI, 'ROLE': 'flare'}, text=True
)
values = {
    columns[2]: columns[3]
    for line in table.splitlines()[2:]
    if len(columns := [column.strip() for column in line.split('|')]) == 5
}
for source in public.glob('*.j2.txt'):
    source.with_name(source.name.replace('.j2', '')).write_text(
        environment.from_string(source.read_text()).render(values, workspace=values['ENVI'])
    )
```

```python
from os import environ
from pathlib import Path
from subprocess import check_output
from sys import argv

from jinja2 import Environment

if len(argv) != 2:
    raise SystemExit('Usage: mise render ENVI')

values = {
    columns[2]: columns[3]
    for line in check_output(
        ['mise', 'tabularize'], env={**environ, 'ENVI': argv[1], 'ROLE': 'flare'}, text=True
    ).splitlines()[2:]
    if len(columns := [column.strip() for column in line.split('|')]) == 5
}
for source in Path('public').glob('*.j2.txt'):
    source.with_name(source.name.replace('.j2', '')).write_text(
        Environment(autoescape=False, keep_trailing_newline=True)
        .from_string(source.read_text())
        .render(values)
    )
```

## Single read from context manager

```python
from pathlib import Path
from os import SEEK_END

file_path = Path('example.txt')

with file_path.open('rb') as handle:
    file_size = handle.seek(0, SEEK_END)
    print(f'File size: {file_size} bytes')
```

```python
from pathlib import Path
from os import SEEK_END

with Path('example.txt').open('rb') as handle:
    print(f'File size: {handle.seek(0, SEEK_END)} bytes')
```

## Zero assignments remain unchanged

```python
print('A01')
print('B02')
```

## Assignment without same-scope use — expect no rewrite

```python
wells = ('A01', 'B02')
print('dispensing to plate')
```

## Assignments used twice remain unchanged

```python
# https://pmc.ncbi.nlm.nih.gov/articles/instance/6810757/bin/NIHMS1037790-supplement-supp_info.pdf
T8M_90964_c23CT = 'GGCCGAAGGAGACGCTGCAGT'
print(T8M_90964_c23CT)
log(T8M_90964_c23CT)
```

## Grit-ignore comment disables inlining — expect no rewrite

Inlining would evaluate `get_time()` after the `sleep`, so opt out with `grit-ignore`.

```python
before = get_time()
sleep(1)
print(get_time() - before)  # grit-ignore
```
