---
title: Inline single-use assignment
---

Inline Python assignments that are read exactly once within the nearest enclosing function, or
the module outside functions, by the very next statement. Skip reads that inlining would defer,
repeat, or leave unwrappable: inside compound statements, lambdas, comprehensions, and f-string
placeholders. Parenthesize operators; keep keyword names and string keys.

```grit
engine marzano(0.1)
language python

`$use_statement` as $use where {
  $use <: after `$variable = $value
` as $assignment,
  $variable <: identifier(),
  $use <: not or {
    class_definition(),
    decorated_definition(),
    for_statement(),
    function_definition(),
    if_statement(),
    match_statement(),
    try_statement(),
    while_statement(),
    with_statement()
  },
  $use <: within or {
    function_definition(body=block(statements=$statements)),
    module(statements=$statements)
  },
  $statements <: not some $other where {
    $other <: contains `$variable`,
    $other <: not $use,
    $other <: not `$variable = $value`
  },
  $reads = [],
  $use <: contains bubble($variable, $reads) or {
    keyword_argument(name=$key, value=$read) where {
      $key <: $variable,
      $read <: $variable,
      $reads += $read
    },
    identifier() as $read where {
      $read <: $variable,
      $read <: not within keyword_argument(name=$variable),
      $reads += $read
    }
  },
  $reads <: [$read],
  $read <: not within or {
    dictionary_comprehension(),
    generator_expression(),
    interpolation(),
    lambda(),
    list_comprehension(),
    set_comprehension()
  },
  $use_statement <: contains bubble($variable, $value) or {
    keyword_argument(name=$key, value=$read) where {
      $key <: $variable,
      $read <: $variable
    } => `$key=$value`,
    identifier() as $read where {
      $read <: $variable,
      $read <: not within keyword_argument(name=$variable),
      $value <: or {
        await(),
        binary_operator(),
        boolean_operator(),
        comparison_operator(),
        conditional_expression(),
        lambda(),
        named_expression(),
        not_operator(),
        unary_operator()
      },
      $read <: within or {
        attribute(object=$variable),
        await(),
        binary_operator(),
        call(function=$variable),
        comparison_operator(),
        not_operator(),
        subscript(value=$variable),
        unary_operator()
      }
    } => `($value)`,
    identifier() as $read where {
      $read <: $variable,
      $read <: not within keyword_argument(name=$variable)
    } => $value
  },
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

## Read twice in the next statement remains unchanged

From helicopyter `look_up_staff`; inlining would also corrupt the `'domain'` key.

```python
def look_up_staff(account_id: str, apex: str) -> Block:
    domain = f'*.{apex}'
    return data.cloudflare_zero_trust_access_application.staff(
        account_id=account_id,
        filter={'domain': domain, 'exact': True},
        error_message=f'Access application {domain} needs an allow policy',
    )
```

## String keys and keyword names keep their text

From helicopyter `look_up_staff` and measles `test_fqdn`.

```python
def look_up_tunnel(account_id: str, apex: str) -> Block:
    name = apex.removeprefix('www.')
    return data.cloudflare_zero_trust_tunnel_cloudflared.staff(
        account_id=account_id, filter={'is_deleted': False, 'name': name}
    )


def fqdn(tmp_path: Path, event: str) -> str:
    env = {'GITHUB_EVENT_NAME': event, 'PATH': environ['PATH']}
    return check_output(['mise', 'fqdn'], cwd=tmp_path, env=env).decode().strip()
```

```python
def look_up_tunnel(account_id: str, apex: str) -> Block:
    return data.cloudflare_zero_trust_tunnel_cloudflared.staff(
        account_id=account_id, filter={'is_deleted': False, 'name': apex.removeprefix('www.')}
    )  # name


def fqdn(tmp_path: Path, event: str) -> str:
    return (
        check_output(
            ['mise', 'fqdn'],
            cwd=tmp_path,
            env={'GITHUB_EVENT_NAME': event, 'PATH': environ['PATH']},
        )
        .decode()
        .strip()
    )  # env
```

## Operators stay grouped

From measles `gitignore`.

```python
def read_gitignore() -> list[str]:
    gitignore_path = Path(environ['PWD']) / '.gitignore'
    return gitignore_path.read_text().splitlines()[2:]
```

```python
def read_gitignore() -> list[str]:
    return (Path(environ['PWD']) / '.gitignore').read_text().splitlines()[2:]  # gitignore_path
```

## Lambda read remains unchanged

From measles `Measles.__init__`; inlining would make `from_string` call itself.

```python
def strip_j2(environment: Environment) -> None:
    render = environment.from_string
    environment.from_string = lambda source, *args, **kwargs: render(
        source.replace('.j2', '') if '\n' not in source else source, *args, **kwargs
    )
```

## Reads inside branches and with blocks remain unchanged

From helicopyter `test_cli` and measles `test_check_branch_rejects_weird_head_references`.

```python
def test_cli(tmp_path: Path, arguments: list[str]) -> None:
    output = check_output([executable, '-m', 'helicopyter', *arguments], cwd=tmp_path, text=True)
    if arguments == ['--help']:
        assert 'usage:' in output
    else:
        assert 'Generating deploys/example/terraform/main.tf' in output


def test_check_branch(tmp_path: Path, head_ref: str) -> None:
    environment = {'GITHUB_HEAD_REF': head_ref, 'PATH': environ['PATH']}
    with raises(CalledProcessError):
        check_output(['mise', 'check-branch'], cwd=tmp_path, env=environment, stderr=STDOUT)
```

## Reads inside comprehensions and f-string placeholders remain unchanged

From helicopyter `multisynth` and `Block.to_hcl`; the comprehension would rebuild `children`
per block.

```python
def render_top_level() -> str:
    children = {id(value) for block in registry for value in block.attributes.values()}
    return '\n\n'.join(block.to_hcl() for block in registry if id(block) not in children)


def to_hcl(self, depth: int = 0) -> str:
    tags = (' ' + ' '.join(f'"{tag}"' for tag in self.labels)) if self.labels else ''
    return f'{"  " * depth}{self.kind}{tags} {{'
```

## Module constant read by a definition remains unchanged

From measles `precise_environment` and helicopyter `HeliStack.push`; each call would create a new
sentinel or TypeVar.

```python
MISSING = object()


@fixture
def precise_environment(monkeypatch: MonkeyPatch) -> Callable[..., None]:
    def inner(**kwargs: str) -> None:
        for key, value in (dict.fromkeys(('HOME', 'PWD'), MISSING) | kwargs).items():
            if value is MISSING:
                monkeypatch.delitem(environ, key, raising=False)

    return inner


class HeliStack(TerraformStack):
    E = TypeVar('E', bound=TerraformElement)

    def push(self, Element: type[E], id_: str) -> E:
        return Element(self, id_)
```

## Grit-ignore comment disables inlining: expect no rewrite

Inlining would evaluate `get_time()` after the `sleep`, so opt out with `grit-ignore`.

```python
before = get_time()
sleep(1)
print(get_time() - before)  # grit-ignore
```
