---
title: Strip a single leading underscore from module-level names
---

Strip a single leading underscore from a module-level function or variable, renaming every
reference.

Reserving variables and functions for internal purposes is a highly speculative library concern
which assumes 1. external use will arise and then 2. internal use will need to permanently diverge
from external use. Instead of peppering lots of code that will never even get to 1. with
`_leading_underscores`, just [expand and contract](https://martinfowler.com/bliki/ParallelChange.html)
the few times you end up all the way at 2. In functions, loops, and unpacking a leading underscore
has an entirely different meaning: unused.

```grit
engine marzano(0.1)
language python

module(statements=$statements) where {
    $statements <: some bubble($statements) or {
        function_definition(name=$name),
        `$name = $value`
    } where {
        $name <: identifier(),
        $name <: r"_([^_].*)"($stripped),
        $statements <: contains `$name` => `$stripped`
    }
}
```

## Strips a module-level variable and function, references and all

```python
_REGISTRY = {}


def _register(name):
    _REGISTRY[name] = True


_register('measles')
```

```python
REGISTRY = {}


def register(name):
    REGISTRY[name] = True


register('measles')
```

## Unused function parameter remains intact

```python
def render(context, _request):
    return context
```

## Loop variable remains intact

```python
for _index in range(3):
    print('tick')
```

## Unpack variable remains intact

```python
first, _second = fetch_pair()
```

## Double leading underscores remain intact

```python
__cache = {}
__all__ = ['Measles']


def __getattr__(name):
    raise AttributeError(name)
```
