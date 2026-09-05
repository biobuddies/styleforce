# `rule_of_three` TODO

Aspirational fixtures copied from #5. Neither rewrites yet. Existing patterns cover the
straight-line skeleton; the steps below note what each still needs, marked (easy) or (hard).

Covered today:
* `assignment-read-once` -- inline an assignment read exactly once.
* `function-called-once` -- inline a single-use function whose body runs straight through to one
  trailing `return`, called as `target = name(arg)`, with positional parameters.
* `standard-identifiers/*` -- unrelated here.

## Inline a timezone-offset helper that has only one caller

```python
def _format_tz_offset(tz_name):
    """Return ISO 8601 offset like '-07:00' for a timezone name."""
    offset_str = datetime.now(ZoneInfo(tz_name)).strftime('%z')
    if not offset_str:
        return '+00:00'
    return f'{offset_str[:3]}:{offset_str[3:]}'


class StrFTime:
    def as_sqlite(self, compiler, connection):
        from django.conf import settings

        format_string = self.format_string
        tz_literal = ''
        if '%z' in format_string:
            format_string = format_string.replace('%z', '')
            tz_literal = _format_tz_offset(settings.TIME_ZONE)
        escaped = format_string.replace('%', '%%%%')
        template = f"%(function)s('{escaped}', %(expressions)s)"
        if tz_literal:
            template += f" || '{tz_literal}'"
        return self.as_sql(compiler, connection, template=template)
```

```python
class StrFTime:
    def as_sqlite(self, compiler, connection):  # noqa: ANN001, ANN201, D102  # pyrefly: ignore[bad-override]
        from django.conf import settings  # noqa: PLC0415

        format_string = self.format_string
        tz_literal = ''
        if '%z' in format_string:
            format_string = format_string.replace('%z', '')
            timezone_offset = datetime.now(ZoneInfo(settings.TIME_ZONE)).strftime('%z')
            tz_literal = (
                f'{timezone_offset[:3]}:{timezone_offset[3:]}' if timezone_offset else '+00:00'
            )
        escaped = format_string.replace('%', '%%%%')
        template = f"%(function)s('{escaped}', %(expressions)s)"
        if tz_literal:
            template += f" || '{tz_literal}'"
        return self.as_sql(compiler, connection, template=template)
```

Steps:
* Inline the one call to `_format_tz_offset`: `function-called-once`, except the body early-returns
  (`if not offset_str: return ...; return ...`). Uncovered (hard): fold an early return into a
  ternary, `f'...' if offset_str else '+00:00'`.
* The call sits in a method inside an `if`, not at module top level. Uncovered (easy): widen
  `function-called-once` past `within module` to any statement scope.
* Collapse `offset_str`, read once: `assignment-read-once` (covered) once the call is inlined.
* Add the `# noqa` and `# pyrefly` suppressions the moved code needs. Uncovered (hard): demands
  linter feedback, beyond a structural rewrite.
* Rename `offset_str` to `timezone_offset`: cosmetic, not worth automating.

## Inline a git-remote helper that has only one caller

```python
def _git_repository_name() -> str | None:
    if not (Path.cwd() / '.git').exists():
        return None
    try:
        remote = check_output(['git', 'remote', 'get-url', 'origin']).decode().strip()
    except CalledProcessError:
        return None
    if repository := search(r'github.com[:/][^/]+/([^/]+)', remote):
        return repository.group(1).removesuffix('.git')
    raise ValueError(f'Unexpected origin URL: {remote!r}')


def cona() -> str:
    """COde NAme, a four-letter abbreviation."""
    if cona := getenv('CONA'):
        pass
    elif repository := getenv('GITHUB_REPOSITORY'):
        cona = repository.split('/')[-1]
    elif repository := _git_repository_name():
        cona = repository
    elif virtual_environment := getenv('VIRTUAL_ENV'):
        cona = Path(virtual_environment).parent.name
    else:
        cona = Path.cwd().name
    if fullmatch(r'[A-Za-z0-9._-]+', cona):
        return cona
    raise ValueError(f'Unexpected CONA characters: {cona!r}')
```

```python
def cona() -> str:
    """COde NAme, a four-letter abbreviation."""
    cona = getenv('CONA') or ''
    if not cona and (repository_slug := getenv('GITHUB_REPOSITORY')):
        cona = repository_slug.split('/')[-1]
    if not cona and (Path.cwd() / '.git').exists():
        try:
            remote = check_output(['git', 'remote', 'get-url', 'origin']).decode().strip()
        except CalledProcessError:
            remote = ''
        if remote and (repository := search(r'github.com[:/][^/]+/([^/]+)', remote)):
            cona = repository.group(1).removesuffix('.git')
    if not cona and (virtual_environment := getenv('VIRTUAL_ENV')):
        cona = Path(virtual_environment).parent.name
    cona = cona or Path.cwd().name
    if fullmatch(r'[A-Za-z0-9._-]+', cona):
        return cona
    raise ValueError(f'Unexpected CONA characters: {cona!r}')
```

Steps:
* Inline the one call to `_git_repository_name`: `function-called-once` reaches single-use
  functions, but not this zero-argument one, nor its `if`/`try`/`except`/`raise` body. Uncovered
  (hard): early returns and exception flow.
* Rewrite the `if`/`elif`/`else` walrus chain as sequential `if not cona and (... := ...)` guards.
  Uncovered (hard): context-sensitive control-flow restructuring, the largest leap here.
* Rename `repository` to `repository_slug`: cosmetic.
