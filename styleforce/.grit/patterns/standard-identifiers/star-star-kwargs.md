---
title: Variadic keyword parameter named 'kwargs'
---

A function's `**` variadic keyword parameter must be named `kwargs`, renaming every reference.

```grit
engine marzano(0.1)
language python

function_definition() as $function where {
    $function <: contains dictionary_splat_pattern(dict=$name),
    $name <: not or { `kwargs`, `_kwargs` },
    if ($name <: r"_.*") { $standard = `_kwargs` } else { $standard = `kwargs` },
    $function <: not contains keyword_argument(name=$name, value=$value) where {
        $value <: not $name
    },
    $function <: contains bubble($name, $standard) or {
        keyword_argument(name=$key, value=$value) where {
            $key <: $name,
            $value <: $name
        } => `$key=$standard`,
        identifier() as $reference where {
            $reference <: $name,
            $reference <: not within attribute(attribute=$name),
            $reference <: not within keyword_argument(name=$name)
        } => `$standard`
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

## Unused variadic keeps its leading underscore

From spamhammer `list_email`.

```python
class Command(BaseCommand):
    def handle(self, *_args: object, **_options: object) -> None:
        print('done')
```

```python
class Command(BaseCommand):
    def handle(self, *_args: object, **_kwargs: object) -> None:
        print('done')
```

## Already named '_kwargs': unchanged

```python
def handle(**_kwargs):
    return None
```
