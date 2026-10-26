---
title: Variadic positional parameter named 'args'
---

A function's `*` variadic positional parameter must be named `args`, renaming every reference.

```grit
engine marzano(0.1)
language python

function_definition() as $function where {
    $function <: contains list_splat_pattern(list=$name),
    $name <: not or { `args`, `_args` },
    if ($name <: r"_.*") { $standard = `_args` } else { $standard = `args` },
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

## Renames the variadic positional parameter to 'args'

```python
def log(*records):
    return len(records)
```

```python
def log(*args):
    return len(args)
```

## Already named 'args': unchanged

```python
def log(*args):
    return len(args)
```

## Keyword-only marker: unchanged

```python
def log(first, *, second):
    return first + second
```

## Attributes and keyword names keep their text

From helicopyter `Block`.

```python
class Block:
    def __call__(self, *labels: str) -> Block:
        return Block(self.kind, *self.labels, *labels)


def tag(*labels: str) -> Block:
    return Block('tag', labels=labels)
```

```python
class Block:
    def __call__(self, *args: str) -> Block:
        return Block(self.kind, *self.labels, *args)


def tag(*args: str) -> Block:
    return Block('tag', labels=args)
```

## Unused variadic keeps its leading underscore

From spamhammer `list_email`.

```python
class Command(BaseCommand):
    def handle(self, *_arguments: object, **_options: object) -> None:
        print('done')
```

```python
class Command(BaseCommand):
    def handle(self, *_args: object, **_options: object) -> None:
        print('done')
```

## Already named '_args': unchanged

```python
def handle(*_args):
    return None
```
