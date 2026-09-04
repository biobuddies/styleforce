---
title: Inline a function called once
---

Inline a function called exactly once, binding each parameter to its argument, hoisting the body,
and assigning the returned expression to the caller's target. The `assignment-read-once` pattern
then collapses any parameter binding read only once.

The body must run straight through: a single `return` reached as the last statement, with no
`if`, `for`, `while`, `try`, `with`, or `match`. TODO translate control flow too, rewriting an
early `return` inside a loop as `continue`, inside a conditional as `if`/`else`, and folding
multiple returns into the surrounding context. TODO inline at non-assignment call sites, where the
result feeds another expression.

```grit
engine marzano(0.1)
language python

`$use_statement` as $use where {
  $use <: `$target = $name($arg)`,
  $use <: after `def $name($param):
    $fnbody
` as $definition,
  $fnbody <: contains `return $expr` as $ret,
  $fnbody <: not contains bubble or {
    if_statement(), for_statement(), while_statement(), try_statement(),
    with_statement(), match_statement()
  },
  $param <: not contains or {
    typed_parameter(), default_parameter(), typed_default_parameter(),
    list_splat_pattern(), dictionary_splat_pattern()
  },
  $arg <: not contains or { keyword_argument(), list_splat(), dictionary_splat() },
  $use <: within module(statements=$statements),
  $statements <: not some $usage where {
    $usage <: contains `$name`,
    $usage <: not $use,
    $usage <: not $definition
  },
  $ret => `$target = $expr`,
  $use => `$param = $arg
$fnbody`,
  $definition => .
}
```

## Inline a multi-statement function

```python
def origin_slug(remote):
    cleaned = remote.strip()
    return cleaned.rsplit('/', 1)[-1].removesuffix('.git')


codename = origin_slug(remote_url)
```

```python
remote = remote_url
cleaned = remote.strip()
codename = cleaned.rsplit('/', 1)[-1].removesuffix('.git')
```

## Inline a single-return function

```python
def offset_literal(offset):
    return f'{offset[:3]}:{offset[3:]}'


timezone_literal = offset_literal(raw_offset)
```

```python
offset = raw_offset
timezone_literal = f'{offset[:3]}:{offset[3:]}'
```

## Function called twice remains unchanged

```python
def origin_slug(remote):
    return remote.rsplit('/', 1)[-1]


first = origin_slug(one)
second = origin_slug(two)
```

## Early return remains unchanged

```python
def classify(value):
    if value < 0:
        return 'negative'
    return 'nonnegative'


label = classify(score)
```

## Function without a caller remains unchanged

```python
def origin_slug(remote):
    return remote.rsplit('/', 1)[-1]
```
