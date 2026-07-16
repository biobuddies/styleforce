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
  $assignment => .
}
```

## Inline two assignments — expect values substituted and assignments removed

```python
wells = ('A01', 'B02')
print(wells)

T8M_90964_c23CT = 'GGCCGAAGGAGACGCTGCAGT'
print(T8M_90964_c23CT)
```

```python

print(('A01', 'B02'))

print('GGCCGAAGGAGACGCTGCAGT')
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
