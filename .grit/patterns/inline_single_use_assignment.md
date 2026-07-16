---
title: Inline single-use assignment
---

Inline Python assignments that are read exactly once.

```grit
engine marzano(0.1)
language python

`$use_statement` as $use where {
  $use <: after `$variable = $value` as $assignment,
  $use <: contains `$variable`,
  $use <: within module(statements=$statements),
  $statements <: not some $other where {
    $other <: contains `$variable`,
    $other <: not $use,
    $other <: not `$variable = $value`
  },
  $use_statement <: contains bubble($variable, $value) `$variable` => $value,
  $assignment <: `$name = $value`,
  $use => `$use_statement  # $name`,
  $assignment => .
}
```

## Inline two assignments — expect values substituted with the name kept as a comment

```python
wells = ('A01', 'B02')
print(wells)

T8M_90964_c23CT = 'GGCCGAAGGAGACGCTGCAGT'
print(T8M_90964_c23CT)
```

```python

print(('A01', 'B02'))  # wells

print('GGCCGAAGGAGACGCTGCAGT')  # T8M_90964_c23CT
```

## No assignment to inline — expect no rewrite

```python
print('A01')
print('B02')
```

## Assignment without same-scope use — expect no rewrite

```python
wells = ('A01', 'B02')
print('dispensing to plate')
```

## Assignment used twice — expect no rewrite

```python
# https://pmc.ncbi.nlm.nih.gov/articles/instance/6810757/bin/NIHMS1037790-supplement-supp_info.pdf
T8M_90964_c23CT = 'GGCCGAAGGAGACGCTGCAGT'
print(T8M_90964_c23CT)
log(T8M_90964_c23CT)
```

## Grit-ignore comment disables inlining — expect no rewrite

Inlining would evaluate `get_time()` after the `sleep`, so opt out with `grit-ignore`.

```python
before = get_time()
sleep(1)
print(get_time() - before)  # grit-ignore
```
