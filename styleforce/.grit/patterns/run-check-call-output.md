---
title: check_call and check_output instead of checked run
---

`run(..., check=True)` spells out what `check_call` and `check_output` already mean. A statement
becomes `check_call`; reading `.stdout` from `stdout=PIPE` or `capture_output=True` becomes
`check_output`. `capture_output` also captured stderr, which `check_output` lets reach the terminal,
so a `try` that might read `CalledProcessError.stderr` stays put. Statements piping stdout or
stderr, or passing `input`, stay put: `check_call` would deadlock or reject them. Unchecked `run`,
such as measles tolerating a failing `mise tabr`, has no checked twin and stays put; so does
`Popen`, which never checks. Bare `run` changes only beside `from subprocess import run`, sparing
`asyncio.run`. Ruff F401 drops a `run` or `PIPE` left unused.

```grit
engine marzano(0.1)
language python

pattern checked_run($bare, $prefix, $arguments) {
    or {
        `run($arguments)` where { $bare <: true, $prefix = `` },
        `subprocess.run($arguments)` where { $prefix = `subprocess.` }
    } where {
        $arguments <: some `check=True`,
        $arguments <: not some or { list_splat(), dictionary_splat() }
    }
}

pattern call_statements($bare) {
    some bubble($bare) checked_run($bare, $prefix, $arguments) as $call where {
        $arguments <: not some or {
            `capture_output=$_`,
            `input=$_`,
            keyword_argument(value=or { `PIPE`, `subprocess.PIPE` })
        },
        $arguments <: some `check=True` => .,
        $call => `$[prefix]check_call($arguments)`
    }
}

pattern output_attribute($bare) {
    `$call.stdout` as $output where {
        $call <: checked_run($bare, $prefix, $arguments),
        $arguments <: or {
            some or { `stdout=PIPE`, `stdout=subprocess.PIPE` } as $keyword,
            some `capture_output=True` as $keyword where { $output <: not within try_statement() }
        },
        $arguments <: some `check=True` => .,
        $keyword => .,
        $output => `$[prefix]check_output($arguments)`
    }
}

module($statements) where {
    $bare = false,
    $statements <: or {
        contains `from subprocess import $names` where {
            $names <: contains `run`,
            $bare = true
        },
        contains `import subprocess`
    },
    $statements <: maybe contains `from subprocess import $names` as $import where {
        $names <: contains `run`,
        $added = [],
        if ($statements <: contains output_attribute($bare)) {
            if ($names <: not contains `check_output`) { $added += "check_output" }
        },
        if ($statements <: or {
            call_statements($bare),
            contains block(statements=$body) where { $body <: call_statements($bare) }
        }) {
            if ($names <: not contains `check_call`) { $added += "check_call" }
        },
        $joined = join(list=$added, separator=", "),
        $joined <: not "",
        $import => `from subprocess import $names, $joined`
    },
    $statements <: maybe call_statements($bare),
    $statements <: maybe contains bubble($bare) block(statements=$body) where {
        $body <: call_statements($bare)
    },
    $statements <: maybe contains bubble($bare) output_attribute($bare)
}
```

## Styleforce formats samples through ruff

From styleforce `test_patterns._ruff`.

```python
from subprocess import PIPE, run


def _ruff(source: str) -> str:
    return run(['ruff', 'format', '-'], check=True, input=source, stdout=PIPE, text=True).stdout
```

```python
from subprocess import PIPE, run, check_output


def _ruff(source: str) -> str:
    return check_output(['ruff', 'format', '-'], input=source, text=True)
```

## Measles TABR stays put while unchecked

From [measles `tabr`](https://github.com/biobuddies/measles/pull/122).

```python
from subprocess import DEVNULL, CalledProcessError, call, check_output, run


def tabr() -> str:
    tabr = (
        getenv('TABR')
        or run(
            ['mise', 'tabr'], capture_output=True, check=False, cwd=workdir, text=True
        ).stdout.strip()
    )
    return validate_name('TABR', tabr) if tabr else ''
```

## Checked measles TABR reuses the import

Synthetic: the measles `tabr` expression, checked.

```python
from subprocess import DEVNULL, CalledProcessError, call, check_output, run


def tabr() -> str:
    tabr = (
        getenv('TABR')
        or run(
            ['mise', 'tabr'], capture_output=True, check=True, cwd=workdir, text=True
        ).stdout.strip()
    )
    return validate_name('TABR', tabr) if tabr else ''
```

```python
from subprocess import DEVNULL, CalledProcessError, call, check_output, run


def tabr() -> str:
    tabr = getenv('TABR') or check_output(['mise', 'tabr'], cwd=workdir, text=True).strip()
    return validate_name('TABR', tabr) if tabr else ''
```

## Checked statements call check_call

Synthetic: helicopyter `test_djlint_jinja_html`, checked.

```python
from os import environ
from subprocess import STDOUT, run

try:
    run(['mise', 'djlint-jinja-html'], check=True, env={**environ, 'AUTOFORMAT_EXCLUDES': ''})
finally:
    run(['git', 'rm', '--force', str(test_path)], check=True, stderr=STDOUT)
```

```python
from os import environ
from subprocess import STDOUT, run, check_call

try:
    check_call(['mise', 'djlint-jinja-html'], env={**environ, 'AUTOFORMAT_EXCLUDES': ''})
finally:
    check_call(['git', 'rm', '--force', str(test_path)], stderr=STDOUT)
```

## Qualified calls stay qualified

Synthetic.

```python
import subprocess

subprocess.run(['git', 'fetch'], check=True)
head = subprocess.run(['git', 'rev-parse', 'HEAD'], check=True, stdout=subprocess.PIPE).stdout
```

```python
import subprocess

subprocess.check_call(['git', 'fetch'])
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'])
```

## Captured stderr inside try stays put

Synthetic.

```python
from subprocess import CalledProcessError, run

try:
    tabr = run(['mise', 'tabr'], capture_output=True, check=True, text=True).stdout
except CalledProcessError as error:
    raise ValueError(error.stderr) from error
```

## Piped and fed statements stay put

Synthetic.

```python
from subprocess import PIPE, run

run(['git', 'status'], check=True, stdout=PIPE)
run(['git', 'apply'], check=True, input=patch)
run(['git', 'diff'], capture_output=True, check=True)
```

## asyncio run stays put

Synthetic variant of spamhammer `peek_email`.

```python
import subprocess
from asyncio import run

message = run(fetch_email(sender, subject, check=True))
```
