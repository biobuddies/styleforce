---
title: Import sleep as asleep or bsleep
---

Use `from asyncio import sleep as asleep` for yielding sleep and
`from time import sleep as bsleep` for blocking sleep. Rewrite qualified uses and direct imports,
including existing aliases. Preserve parameter bindings and their uses. Ruff isort merges and
sorts imports; F401 removes unused module imports. Existing destination names raise an error.

```grit
engine marzano(0.1)
language python

pattern sleep_parameter($name) {
    or {
        $name,
        default_parameter(name=$name),
        typed_parameter(name=$name),
        typed_default_parameter(name=$name),
        list_splat_pattern(list=$name),
        dictionary_splat_pattern(dict=$name)
    }
}

pattern shadowing_sleep_scope($name) {
    or {
        function_definition(parameters=$parameters),
        lambda(parameters=$parameters)
    } where {
        $parameters <: or {
            some sleep_parameter(name=$name),
            lambda_parameters(parameters=$items) where {
                $items <: some sleep_parameter(name=$name)
            }
        }
    }
}

pattern rename_sleep_reference($bound, $alias) {
    or {
        keyword_argument(name=$key, value=$bound) => `$key=$alias`,
        identifier() as $reference where {
            $reference <: $bound,
            $reference <: not within import_from_statement(),
            $reference <: not within attribute(attribute=$bound),
            $reference <: not within keyword_argument(name=$bound)
        } => `$alias`
    }
}

pattern rename_sleep_references($bound, $alias) {
    contains bubble($bound, $alias) or {
        shadowing_sleep_scope(name=$bound) as $scope where {
            $scope <: or {
                function_definition(parameters=$parameters),
                lambda(parameters=$parameters)
            },
            $parameters <: maybe contains bubble($bound, $alias) or {
                default_parameter(value=$value),
                typed_default_parameter(value=$value)
            } where {
                $value <: contains rename_sleep_reference(bound=$bound, alias=$alias)
                    until shadowing_sleep_scope(name=$bound)
            },
            $parameters <: maybe contains bubble($bound, $alias) or {
                typed_parameter(type=$type),
                typed_default_parameter(type=$type)
            } where {
                $type <: contains rename_sleep_reference(bound=$bound, alias=$alias)
                    until shadowing_sleep_scope(name=$bound)
            },
            $scope <: maybe function_definition(return_type=$type) where {
                $type <: contains rename_sleep_reference(bound=$bound, alias=$alias)
                    until shadowing_sleep_scope(name=$bound)
            }
        },
        rename_sleep_reference(bound=$bound, alias=$alias)
    } until shadowing_sleep_scope(name=$bound)
}

module($statements) where {
    $statements <: contains bubble($statements) or {
        `import $module` as $import where {
            $module <: or {
                "asyncio" where { $alias = "asleep" },
                "time" where { $alias = "bsleep" }
            },
            $statements <: contains attribute(object=$module, attribute=`sleep`),
            $statements <: not contains identifier() as $collision where {
                $collision <: $alias,
                $message = join(list=["ERROR: sleep-imports: ", $alias, " already exists"],
                    separator=""),
                log(message=$message)
            },
            $statements <: not contains or {
                assignment(left=$target) where { $target <: contains $module },
                function_definition(parameters=$parameters) where {
                    $parameters <: contains $module
                }
            },
            $statements <: contains attribute(object=$module, attribute=`sleep`) => `$alias`,
            $import => `import $module
from $module import sleep as $alias`
        },
        `from $module import $names` where {
            $module <: or {
                "asyncio" where { $alias = "asleep" },
                "time" where { $alias = "bsleep" }
            },
            $names <: contains or {
                aliased_import(name=$name, alias=$bound) as $import where {
                    $name <: "sleep",
                    $bound <: not $alias,
                    $import => `sleep as $alias`
                },
                `sleep` as $import where {
                    $import <: not within aliased_import(),
                    $bound = `sleep`,
                    $import => `sleep as $alias`
                }
            },
            $statements <: not contains identifier() as $collision where {
                $collision <: $alias,
                $message = join(list=["ERROR: sleep-imports: ", $alias, " already exists"],
                    separator=""),
                log(message=$message)
            },
            $statements <: maybe rename_sleep_references(bound=$bound, alias=$alias)
        }
    }
}
```

## Django blocking and yielding sleep

Excerpted from [Django ASGI test views](https://github.com/django/django/blob/030c63d329c4814da221528e823a4aaaaa40e4f1/tests/asgi/urls.py).
Ruff F401 removes the leftover module imports.

```python
import asyncio
import time


def hello_with_delay(request):
    name = request.GET.get('name') or 'World'
    time.sleep(1)
    return HttpResponse(f'Hello {name}!')


async def streaming_inner(sleep_time):
    yield b'first\n'
    await asyncio.sleep(sleep_time)
    yield b'last\n'
```

```python
import asyncio

from asyncio import sleep as asleep
import time

from time import sleep as bsleep


def hello_with_delay(request):
    name = request.GET.get('name') or 'World'
    bsleep(1)
    return HttpResponse(f'Hello {name}!')


async def streaming_inner(sleep_time):
    yield b'first\n'
    await asleep(sleep_time)
    yield b'last\n'
```

## Direct blocking import

Synthetic import variant of the Django call.

```python
from time import monotonic, sleep

started = monotonic()
sleep(1)
```

```python
from time import monotonic, sleep as bsleep

started = monotonic()
bsleep(1)
```

## Direct yielding import

Synthetic import variant of the Django call.

```python
from asyncio import sleep


async def streaming_inner(sleep_time):
    await sleep(sleep_time)
```

```python
from asyncio import sleep as asleep


async def streaming_inner(sleep_time):
    await asleep(sleep_time)
```

## Existing aliases and function values

Synthetic.

```python
from asyncio import sleep as A_sleep
from time import sleep as B_sleep

blocking = B_sleep
configure(sleep=B_sleep, B_sleep=B_sleep)
scheduler.sleep(1)


async def pause():
    await A_sleep(0)
```

```python
from asyncio import sleep as asleep
from time import sleep as bsleep

blocking = bsleep
configure(sleep=bsleep, B_sleep=bsleep)
scheduler.sleep(1)


async def pause():
    await asleep(0)
```

## Preferred aliases stay put

Synthetic.

```python
from asyncio import sleep as asleep
from time import sleep as bsleep

bsleep(1)


async def pause():
    await asleep(0)
```

## Unrecognized sleep stays put

Synthetic. Are there other standard or popular library functions we should recognize and assign
unique names to?

```python
from scheduler import sleep

sleep(1)
scheduler.sleep(1)
text = 'time.sleep(1)'
```

## Other module uses remain qualified

Synthetic. Concession: retain the module import when existing code passes the module as an argument.
TODO: convert `time.monotonic` to a direct import.

```python
import time

started = time.monotonic()
time.sleep(1)
register(time)
```

```python
import time

from time import sleep as bsleep

started = time.monotonic()
bsleep(1)
register(time)
```

## Existing destination names raise an error

Synthetic. Report the collision rather than overwrite existing alias bindings.

```python
import asyncio
from time import sleep

asleep = custom_sleep
asyncio.sleep(0)
sleep(1)
```

```error
RuntimeError: sleep-imports: asleep already exists
```

## Keyword Argument Unchanged

Synthetic.

```python
from time import sleep

sleep(1)


def pause(sleep):
    sleep(2)
```

```python
from time import sleep as bsleep

bsleep(1)


def pause(sleep):
    sleep(2)
```

## Rename rebound direct imports and their references

Synthetic.

```python
from time import sleep

sleep = custom_sleep
sleep(1)
```

```python
from time import sleep as bsleep

bsleep = custom_sleep
bsleep(1)
```

## Shadowed modules stay put

Synthetic. The parameter shadows the module; its attributes must remain unchanged.

```python
import time


def pause(time):
    time.sleep(1)
```

## Parameter defaults use the imported sleep

Synthetic. Defaults and annotations evaluate outside the parameter's local scope.

```python
from time import sleep


def pause(sleep: sleep = sleep) -> sleep:
    configure(sleep=sleep)
    sleep(1)
```

```python
from time import sleep as bsleep


def pause(sleep: bsleep = bsleep) -> bsleep:
    configure(sleep=sleep)
    sleep(1)
```

## Lambda parameters stay unchanged

Synthetic.

```python
from time import sleep

sleep(1)
pause = lambda sleep: sleep(2)
delay = lambda seconds=sleep: sleep(seconds)
```

```python
from time import sleep as bsleep

bsleep(1)
pause = lambda sleep: sleep(2)
delay = lambda seconds=bsleep: bsleep(seconds)
```

## Direct import destination collision raises an error

Synthetic.

```python
from time import sleep

bsleep = custom_sleep
sleep(1)
```

```error
RuntimeError: sleep-imports: bsleep already exists
```
