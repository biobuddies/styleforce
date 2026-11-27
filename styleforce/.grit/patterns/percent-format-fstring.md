---
title: F-strings instead of percent formatting
---

Rewrite `%s` and `%r` placeholders in logger calls and `%` expressions as f-strings. Percent
formatting stays as the fallback when literal curly braces would need escaping.
Logging f-strings trip ruff G004 and format eagerly, even when the level is disabled.
`'%s' % value` assumes `value` is not a tuple. `''.format()` stays permitted, not forced.

```grit
engine marzano(0.1)
language python

pattern interpolate($done, $rest) {
    $value where {
        $value <: not or { list_splat(), dictionary_splat(), keyword_argument() },
        $rest <: r"(?s)([^%]*)%([rs])(.*)"($before, $conversion, $after),
        if ($conversion <: "r") { $suffix = "!r" } else { $suffix = "" },
        $done = join(list=[$done, $before, "{", text($value), $suffix, "}"], separator=""),
        $rest = $after
    }
}

or {
    `$logger.$level($arguments)` as $call where {
        $level <: or { `critical`, `debug`, `error`, `exception`, `info`, `warning` },
        $arguments <: [$format, $_, ...],
        $format <: string(),
        $format <: r"(?s)['\"][^{}]*",
        $done = "",
        $rest = text($format),
        $keywords = "",
        $started = false,
        $arguments <: every bubble($done, $rest, $keywords, $started) or {
            $_ where { $started <: false, $started = true },
            keyword_argument() as $keyword where { $keywords = `$keywords, $keyword` },
            interpolate($done, $rest)
        }
    } => `$logger.$level(f$done$rest$keywords)`,
    binary_operator(left=$format, operator="%", right=$values) where {
        $format <: string(),
        $format <: r"(?s)['\"][^{}]*",
        $done = "",
        $rest = text($format),
        $values <: or {
            tuple($elements) where {
                $elements <: every bubble($done, $rest) interpolate($done, $rest)
            },
            interpolate($done, $rest) where { $values <: not tuple() }
        }
    } => `f$done$rest`
}
```

## Logger call

From spamhammer.

```python
logger.info('%s LOGIN OK', parsed.hostname)
```

```python
logger.info(f'{parsed.hostname} LOGIN OK')
```

## Curly braces fall back to percent

From [airdjang `dags/hello.py`](https://github.com/biobuddies/airdjang/blob/acce53305260cd44459c42c2b32ba848170bedde/dags/hello.py#L28).

```python
class Template:
    def __repr__(self) -> str:  # noqa: D105 self-documenting
        return '{{%s}}' % self.source
```

## Representations and keywords

Synthetic.

```python
logger.warning('%s rejected %r', sender, message, exc_info=True)
logger.info('%s', headers['Host'])
message = 'Hello %s and %r' % (first, second)
single = '%s' % (one,)
```

```python
logger.warning(f'{sender} rejected {message!r}', exc_info=True)
logger.info(f'{headers["Host"]}')
message = f'Hello {first} and {second!r}'
single = f'{one}'
```

## Modulo stays put

Synthetic.

```python
remainder = count % 7
```
