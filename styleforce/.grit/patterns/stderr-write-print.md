---
title: Print to stderr instead of writing to it
---

The stdout rule's twin: `print(text, file=stderr)`, imported one way: `from sys import stderr`,
beside any existing names or after the docstring. Ruff isort then places a new import, and F401
drops a `sys` left unused. Django's
`self.stderr.write` also styles text red, which `print` drops; failures should
`raise CommandError` instead, which Django prints in red and exits nonzero.

```grit
engine marzano(0.1)
language python

// TODO support Django's ending= and style_func= once a real sample shows them
module($statements) where {
    $statements <: contains or { `self.stderr.write($text)`, `sys.stderr.write($text)` } where {
        $text <: not contains keyword_argument()
    },
    $statements <: or {
        contains `from sys import $names` as $import where {
            $names <: not contains `stderr`,
            $import => `from sys import $names, stderr`
        },
        contains `from sys import $names` where { $names <: contains `stderr` },
        [r"(?s)^\"\"\".*", $first, ...] where { $first => `from sys import stderr\n$first` },
        [$first, ...] where { $first => `from sys import stderr\n$first` }
    },
    $statements <: contains bubble or {
        `sys.stderr.write($text)` => `print($text, end='', file=stderr)`,
        `self.stderr.write($text)` where {
            $text <: or {
                r"(?s).*\\n['\"]" => `$text, end=''`,
                $_
            }
        } => `print($text, file=stderr)`
    } where {
        $text <: not contains keyword_argument()
    }
}
```

## Django management command warns

Synthetic; a variant of lynch-library `transcribe_podcast_episode` with no real stderr sample yet.

```python
"""Transcribe episodes from the podcast archive."""

from typing import Any

from django.core.management.base import BaseCommand


class Command(BaseCommand):
    """Placeholder for transcribing archived audio."""

    def handle(self, *_args: Any, **_options: Any) -> None:
        """Warn until a transcriber is chosen."""
        self.stderr.write('Transcription not yet implemented')
```

```python
"""Transcribe episodes from the podcast archive."""

from sys import stderr
from typing import Any

from django.core.management.base import BaseCommand


class Command(BaseCommand):
    """Placeholder for transcribing archived audio."""

    def handle(self, *_args: Any, **_options: Any) -> None:
        """Warn until a transcriber is chosen."""
        print('Transcription not yet implemented', file=stderr)
```

## Existing sys import gains stderr

Synthetic; no real sample yet.

```python
from sys import argv


class Command(BaseCommand):
    def handle(self, *_args: object, **_options: object) -> None:
        self.stderr.write(f'Ignoring {argv[1:]}\n')
```

```python
from sys import argv, stderr


class Command(BaseCommand):
    def handle(self, *_args: object, **_options: object) -> None:
        print(f'Ignoring {argv[1:]}\n', end='', file=stderr)
```

## sys.stderr.write keeps its missing newline

Synthetic; no real sample yet.

```python
import sys

sys.stderr.write(f'{path}: unchanged\n')
```

```python
from sys import stderr
import sys

print(f'{path}: unchanged\n', end='', file=stderr)
```

## Style keyword stays put

Synthetic; no real sample yet.

```python
self.stderr.write(message, style_func=self.style.WARNING)
```
