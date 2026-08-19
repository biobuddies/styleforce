---
title: Inner function named 'inner'
---

A function that defines exactly one inner function and returns it must name that inner function `inner`.

```grit
engine marzano(0.1)
language python

function_definition(body=$body) as $outer where {
    $body <: contains function_definition(name=$inner_name),
    $inner_name <: not `inner`,
    $body <: contains `return $inner_name`,
    $body <: not contains function_definition(name=$other) where {
        $other <: not $inner_name
    },
    $inner_name => `inner`
}
```

## Renames sole inner function to 'inner'

```python
def make_handler():
    def handler(event):
        return {"status": "ok", "event": event}
    return handler
```

```python
def make_handler():
    def inner(event):
        return {"status": "ok", "event": event}
    return inner
```

## Already named 'inner' — unchanged

```python
def make_handler():
    def inner(event):
        return {"status": "ok", "event": event}
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

## Inner function not returned — unchanged

```python
def make_handler():
    def handler(event):
        return {"status": "ok", "event": event}
    return lambda event: handler(event)
```
