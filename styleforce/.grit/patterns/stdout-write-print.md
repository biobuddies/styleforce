---
title: Print instead of writing to stdout
---

`print` reads plainest; Django's `self.stdout.write` adds the newline `print` adds, while
`sys.stdout.write` needs `end=''`. Calls with keyword arguments such as Django's `ending` stay put.
Linters flagging `print`, like ruff T201, then need a terse `# noqa`.

```grit
engine marzano(0.1)
language python

or {
    `self.stdout.write($text)` => `print($text)`,
    `sys.stdout.write($text)` => `print($text, end='')`
} where {
    $text <: not contains keyword_argument()
}
```

## Django management command output

From spamhammer `list_email`.

```python
class Command(BaseCommand):
    help = __doc__

    def handle(self, *_args: object, **_options: object) -> None:
        """Print one sender per line."""
        self.stdout.write('\n'.join(run(list_email(read_email_url()))))
```

```python
class Command(BaseCommand):
    help = __doc__

    def handle(self, *_args: object, **_options: object) -> None:
        """Print one sender per line."""
        print('\n'.join(run(list_email(read_email_url()))))
```

## sys.stdout.write keeps its missing newline

From styleforce `main`.

```python
if reformatted:
    sys.stdout.write(
        f'Reformatted {len(reformatted)} files:\n' + '\n'.join(map(str, reformatted)) + '\n'
    )
```

```python
if reformatted:
    print(
        f'Reformatted {len(reformatted)} files:\n' + '\n'.join(map(str, reformatted)) + '\n', end=''
    )
```
