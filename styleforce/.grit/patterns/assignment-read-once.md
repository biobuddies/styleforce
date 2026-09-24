---
title: Inline single-use assignment
---

Inline Python assignments that are read exactly once within the nearest enclosing function, or
the module outside functions.

```grit
engine marzano(0.1)
language python

`$use_statement` as $use where {
  $use <: after `$variable = $value
` as $assignment,
  $use <: contains `$variable`,
  $use <: within or {
    function_definition(body=block(statements=$statements)),
    module(statements=$statements)
  },
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

## Avoid assignments used once

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

## Zero assignments remain unchanged

```python
print('A01')
print('B02')
```

## Assignment without same-scope use: expect no rewrite

```python
wells = ('A01', 'B02')
print('dispensing to plate')
```

## Assignments used twice remain unchanged

```python
# https://pmc.ncbi.nlm.nih.gov/articles/instance/6810757/bin/NIHMS1037790-supplement-supp_info.pdf
T8M_90964_c23CT = 'GGCCGAAGGAGACGCTGCAGT'
print(T8M_90964_c23CT)
log(T8M_90964_c23CT)
```

## Inline inside a function body

From helicopyter `HeliStack.provide`.

```python
def provide(self, name: str, **kwargs: Any) -> type[TerraformElement]:
    module = import_module(f'cdktf_cdktf_provider_{name}.provider')
    return getattr(module, f'{name.title()}Provider')(self, 'this', **kwargs)
```

```python
def provide(self, name: str, **kwargs: Any) -> type[TerraformElement]:
    return getattr(
        import_module(f'cdktf_cdktf_provider_{name}.provider'), f'{name.title()}Provider'
    )(self, 'this', **kwargs)  # module
```

## Read after the enclosing block remains unchanged

From nodeenv 1.9.1 `copy_node_from_prebuilt`. Scoping to the function body, not the nearest block,
keeps `dest` defined for `copytree`.

```python
def copy_node_from_prebuilt(env_dir, src_dir, node_version):
    if is_WIN:
        dest = join(env_dir, 'Scripts')
        mkdir(dest)
    elif is_CYGWIN:
        dest = join(env_dir, 'bin')
        mkdir(dest)
    else:
        dest = env_dir
    (src_folder,) = glob.glob(src_dir + to_utf8('/node-v%s*' % node_version))
    copytree(src_folder, dest, True)
```

## Closure read remains unchanged

Inlining would defer `perf_counter()` until `elapsed` runs.

```python
def start_timer() -> Callable[[], float]:
    started = perf_counter()

    def elapsed() -> float:
        return perf_counter() - started

    return elapsed
```

## Grit-ignore comment disables inlining: expect no rewrite

Inlining would evaluate `get_time()` after the `sleep`, so opt out with `grit-ignore`.

```python
before = get_time()
sleep(1)
print(get_time() - before)  # grit-ignore
```
