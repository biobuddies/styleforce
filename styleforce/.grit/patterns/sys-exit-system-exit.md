---
title: Raise SystemExit instead of calling sys.exit
---

`sys.exit(status)` raises `SystemExit(status)`. Raise it directly without an import or shadowing
`exit`, [Python's interactive helper](https://docs.python.org/3.12/library/constants.html#constants-added-by-the-site-module). Ruff F401 removes unused imports.

```grit
engine marzano(0.1)
language python

pattern shadows_exit_name($name) {
    or {
        assignment(left=$target) where { $target <: contains $name },
        function_definition(name=$name),
        class_definition(name=$name),
        function_definition(parameters=$parameters) where { $parameters <: contains $name }
    }
}

module($statements) where {
    $statements <: not contains or {
        shadows_exit_name(name=`SystemExit`),
        `from $_ import $names` where { $names <: contains `SystemExit` },
        aliased_import(alias=`SystemExit`)
    },
    $statements <: contains bubble($statements) or {
        `import sys` where { $bound = `sys`, $qualified = true },
        `import sys as $bound` where { $qualified = true },
        `from sys import $names` where {
            $qualified = false,
            $names <: contains or {
                aliased_import(name=$name, alias=$bound) where { $name <: "exit" },
                `exit` as $name where {
                    $name <: not within aliased_import(),
                    $bound = `exit`
                }
            }
        }
    } where {
        $statements <: not contains shadows_exit_name(name=$bound),
        $statements <: contains bubble($bound, $qualified) call(
            function=$function, arguments=argument_list($arguments)
        ) as $statement where {
            $function <: or {
                attribute(object=$bound, attribute=`exit`) where { $qualified <: true },
                $bound where { $qualified <: false }
            },
            $statement <: within or {
                block(statements=$siblings),
                module(statements=$siblings)
            },
            $siblings <: some $statement,
            $arguments <: or {
                [] where { $status = "" },
                [$status] where {
                    $status <: not or { dictionary_splat(), keyword_argument(), list_splat() }
                }
            },
            $statement => `raise SystemExit($status)`
        }
    }
}
```

## Django autoreload exit status

Excerpted from [Django `trigger_reload`](https://github.com/django/django/blob/030c63d329c4814da221528e823a4aaaaa40e4f1/django/utils/autoreload.py#L264).

```python
import sys


def trigger_reload(filename):
    logger.info('%s changed, reloading.', filename)
    sys.exit(3)
```

```python
import sys


def trigger_reload(filename):
    logger.info('%s changed, reloading.', filename)
    raise SystemExit(3)
```

## Exit arguments

Synthetic. Preserve successful exits, numeric statuses, messages, and evaluated arguments.

```python
import sys

sys.exit()
sys.exit(None)
sys.exit(0)
sys.exit('failed')
sys.exit(main())
```

```python
import sys

raise SystemExit()
raise SystemExit(None)
raise SystemExit(0)
raise SystemExit('failed')
raise SystemExit(main())
```

## Imported aliases

Synthetic.

```python
import sys as system
from sys import exit as sysexit

system.exit(1)
sysexit(2)
```

```python
import sys as system
from sys import exit as sysexit

raise SystemExit(1)
raise SystemExit(2)
```

## Direct exit import

Synthetic.

```python
from sys import argv, exit

if len(argv) == 1:
    exit('missing argument')
```

```python
from sys import argv, exit

if len(argv) == 1:
    raise SystemExit('missing argument')
```

## Django signal handler remains an expression

Excerpted from [Django `run_with_reloader`](https://github.com/django/django/blob/030c63d329c4814da221528e823a4aaaaa40e4f1/django/utils/autoreload.py#L676).
Concession: raising inside a lambda requires restructuring the handler.

```python
import sys

signal.signal(signal.SIGTERM, lambda *args: sys.exit(0))
```

## Expression calls and callbacks stay put

Synthetic. Concession: a raise statement cannot replace an expression or a function reference.

```python
import sys

callback = sys.exit
result = sys.exit(1)

def stop():
    return sys.exit(2)
```

## Interactive and unrelated exits stay put

Synthetic. Recognize the import rather than guessing from the function name.

```python
exit(1)
quit(1)
connection.exit(1)
```

## Parameter shadows stay put

Synthetic. Concession: resolving parameter shadows requires distinguishing imported references.

```python
import sys
from sys import exit as sysexit

sys.exit(1)
sysexit(1)

def stop(sys, sysexit):
    sys.exit(2)
    sysexit(2)
```

## Reassigned exit bindings stay put

Synthetic. Concession: after reassignment, calling the replacement need not raise SystemExit.

```python
from sys import exit

exit = custom_exit
exit(1)
```

## Shadowed SystemExit stays put

Synthetic. Concession: the destination must refer to Python's exception.

```python
import sys

SystemExit = custom_exception
sys.exit(1)
```

## Invalid and unpacked calls stay put

Synthetic. Preserve TypeError for invalid calls; unpacked arguments need their arity resolved.

```python
import sys

sys.exit(1, 2)
sys.exit(status=1)
sys.exit(*statuses)
sys.exit(**options)
```
