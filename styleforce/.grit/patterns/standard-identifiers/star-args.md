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
    $function <: not contains keyword_argument(name=$name),
    $function <: contains bubble($name) identifier() as $reference where {
        $reference <: $name,
        $reference <: not within attribute(attribute=$name)
    } => `args`
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

From helicopyter `Block`; a keyword named like the parameter skips the function.

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


def tag(*labels: str) -> Block:
    return Block('tag', labels=labels)
```
