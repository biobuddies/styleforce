---
title: Sole inner function named 'inner'
---

A function whose body defines exactly one inner function must name it `inner`, renaming every
reference alongside the definition.

```grit
engine marzano(0.1)
language python

function_definition(body=$body) where {
    $body <: contains function_definition(name=$inner_name),
    $inner_name <: not `inner`,
    $body <: not contains function_definition(name=$other) where {
        $other <: not $inner_name
    },
    $body <: contains `$inner_name` => `inner`
}
```

## Renames the sole returned inner function to 'inner'

```python
def make_handler():
    def handler(event):
        return {'status': 'ok', 'event': event}

    return handler
```

```python
def make_handler():
    def inner(event):
        return {'status': 'ok', 'event': event}

    return inner
```

## Renames a sole inner function reached only by call

```python
def double_first(values):
    def scale(value):
        return value * 2

    return scale(values[0])
```

```python
def double_first(values):
    def inner(value):
        return value * 2

    return inner(values[0])
```

## Already named 'inner' — unchanged

```python
def make_handler():
    def inner(event):
        return {'status': 'ok', 'event': event}

    return inner
```

## Multiple inner functions — unchanged

```python
def make_handlers():
    def get():
        pass

    def post():
        pass

    return get, post
```
