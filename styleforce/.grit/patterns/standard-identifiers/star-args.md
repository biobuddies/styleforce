---
title: Variadic positional parameter named 'args'
---

A function's `*` variadic positional parameter must be named `args`, renaming every reference.

```grit
engine marzano(0.1)
language python

function_definition() as $function where {
    $function <: contains list_splat_pattern(list=$name),
    $name <: not `args`,
    $function <: contains `$name` => `args`
}
```

## Renames the variadic positional parameter to 'args'

```python
def log(*records):
    return len(records)
```

```python
def log(*args):
    return len(args)
```

## Already named 'args' — unchanged

```python
def log(*args):
    return len(args)
```

## Keyword-only marker — unchanged

```python
def log(first, *, second):
    return first + second
```
