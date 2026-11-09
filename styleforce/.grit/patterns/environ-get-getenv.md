---
title: getenv instead of environ.get
---

`getenv` says the same in fewer characters, imported one way: `from os import getenv`, beside any
existing names or after the docstring. Ruff isort then places a new import, and F401 drops an
`environ` or `os` left unused. Bare `environ.get` changes only beside `from os import environ`,
sparing WSGI's `environ` dictionary.

```grit
engine marzano(0.1)
language python

module($statements) where {
    $statements <: or {
        contains or { `os.environ.get($_)`, `os.getenv($_)` },
        and {
            contains `environ.get($_)`,
            contains `from os import $names` where { $names <: contains `environ` }
        }
    },
    $statements <: or {
        contains `from os import $names` as $import where {
            $names <: not contains `getenv`,
            $import => `from os import $names, getenv`
        },
        contains `from os import $names` where { $names <: contains `getenv` },
        [r"(?s)^\"\"\".*", $first, ...] where { $first => `from os import getenv\n$first` },
        [$first, ...] where { $first => `from os import getenv\n$first` }
    },
    $statements <: contains bubble or {
        `os.environ.get($arguments)` => `getenv($arguments)`,
        `os.getenv($arguments)` => `getenv($arguments)`,
        `environ.get($arguments)` => `getenv($arguments)`
    }
}
```

## Rewrites the from-import and calls

From spamhammer `with_http`.

```python
from logging import getLogger
from os import environ
from urllib.request import Request, urlopen


def decorate(function: Callable[[Get], T]) -> Callable[[], T | None]:
    @wraps(function)
    def wrapper() -> T | None:
        key = environ.get(variable, 'off')
        if key.startswith('off'):
            logger.info('%s off', variable)
            return off
```

```python
from logging import getLogger
from os import environ, getenv
from urllib.request import Request, urlopen


def decorate(function: Callable[[Get], T]) -> Callable[[], T | None]:
    @wraps(function)
    def wrapper() -> T | None:
        key = getenv(variable, 'off')
        if key.startswith('off'):
            logger.info('%s off', variable)
            return off
```

## Qualified calls import getenv

Synthetic; no real `os.environ.get` sample yet.

```python
import os

key = os.environ.get('KEY', 'off')
home = os.getenv('HOME')
```

```python
from os import getenv
import os

key = getenv('KEY', 'off')
home = getenv('HOME')
```

## WSGI environ stays put

Synthetic; no real sample yet.

```python
def application(environ, start_response):
    return environ.get('PATH_INFO')
```
