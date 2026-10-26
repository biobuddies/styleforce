---
title: getenv instead of environ.get
---

`getenv` says the same in fewer characters. Imports gain `getenv`; ruff F401 later drops an
`environ` left unused.

```grit
engine marzano(0.1)
language python

or {
    `os.environ.get($arguments)` => `os.getenv($arguments)`,
    module($statements) where {
        $statements <: contains `environ.get($arguments)`,
        $statements <: maybe contains `from os import $names` as $import where {
            $names <: contains `environ`,
            $names <: not contains `getenv`,
            $import => `from os import $names, getenv`
        },
        $statements <: contains bubble `environ.get($arguments)` => `getenv($arguments)`
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
