---
title: Variadic keyword parameter named 'kwargs'
---

A function's `**` variadic keyword parameter must be named `kwargs`, renaming every reference.

```grit
engine marzano(0.1)
language python

function_definition() as $function where {
    $function <: contains dictionary_splat_pattern(dict=$name),
    $name <: not `kwargs`,
    $function <: not contains keyword_argument(name=$name, value=$value) where {
        $value <: not $name
    },
    $function <: contains bubble($name) or {
        keyword_argument(name=$key, value=$value) where {
            $key <: $name,
            $value <: $name
        } => `$key=kwargs`,
        identifier() as $reference where {
            $reference <: $name,
            $reference <: not within attribute(attribute=$name),
            $reference <: not within keyword_argument(name=$name)
        } => `kwargs`
    }
}
```

## Renames the variadic keyword parameter to 'kwargs'

```python
def build(**options):
    return dict(options)
```

```python
def build(**kwargs):
    return dict(kwargs)
```

## Already named 'kwargs': unchanged

```python
def build(**kwargs):
    return dict(kwargs)
```
