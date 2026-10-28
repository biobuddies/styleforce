---
title: Print instead of writing to stdout
---

`print` reads plainest; Django's `self.stdout.write` adds the newline `print` adds, unless the text
already ends with one, while `sys.stdout.write` needs `end=''`. Literals ending in `\n` gain
`end=''`; other text ending in a newline would print a blank line. Calls with keyword arguments such
as Django's `ending` stay put until a real sample shows them. `print` bypasses
`call_command(stdout=...)`, so tests capture with pytest's `capsys` instead. Linters flagging
`print`, like ruff T201, then need a terse `# noqa` outside command line interfaces.

```grit
engine marzano(0.1)
language python

// TODO support Django's ending= once a real sample shows it, e.g. ending='' => end=''
or {
    `self.stdout.write($text)` => `print($text, end='')` where { $text <: r"(?s).*\\n['\"]" },
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

## Placeholder management command

From lynch-library `transcribe_podcast_episode`.

```python
class Command(BaseCommand):
    """Placeholder for transcribing archived audio."""

    help = 'Transcribe the oldest archived episode lacking a transcript; not yet implemented'

    def handle(self, *_args: Any, **_options: Any) -> None:
        """Do nothing until a transcriber is chosen."""
        self.stdout.write('Transcription not yet implemented')
```

```python
class Command(BaseCommand):
    """Placeholder for transcribing archived audio."""

    help = 'Transcribe the oldest archived episode lacking a transcript; not yet implemented'

    def handle(self, *_args: Any, **_options: Any) -> None:
        """Do nothing until a transcriber is chosen."""
        print('Transcription not yet implemented')
```

## Trailing newline literal keeps its single newline

Synthetic; no real sample yet.

```python
self.stdout.write(f'Archived {count} episodes\n')
```

```python
print(f'Archived {count} episodes\n', end='')
```

## Ending keyword stays put

Synthetic; no real sample yet.

```python
self.stdout.write(progress, ending='')
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
