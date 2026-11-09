---
title: Import re.compile as recompile
---

`from re import compile as recompile` keeps calls concise without shadowing Python's built-in
`compile`. Import every other `re` attribute by its bound name too. Ruff isort merges and sorts
imports. Keep `import re` only when the module itself is used as a value. Built-in `compile` without
a matching import stays put.

```grit
engine marzano(0.1)
language python

module($statements) where {
    $statements <: or {
        contains `import re`,
        contains `from re import $names` where { $names <: contains `compile` }
    },
    $statements <: maybe contains `from re import $names` where {
        $names <: contains `compile` as $name where {
            $name <: not within aliased_import()
        },
        $name => `compile as recompile`,
        $statements <: maybe contains bubble or {
            keyword_argument(name=$key, value=$value) where {
                $value <: `compile`
            } => `$key=recompile`,
            identifier() as $reference where {
                $reference <: `compile`,
                $reference <: not within import_from_statement(),
                $reference <: not within attribute(attribute=`compile`),
                $reference <: not within keyword_argument(name=`compile`)
            } => `recompile`
        }
    },
    $statements <: maybe contains `import re` as $import where {
        $imports = [],
        $statements <: contains bubble($imports) attribute(object=`re`, attribute=$name) as $use where {
            $name <: or {
                `compile` where { $bound = "recompile", $imports += "compile as recompile" },
                $_ where { $bound = $name, $imports += $name }
            },
            $use => `$bound`
        },
        $names = join(list=distinct(list=$imports), separator=", "),
        $statements <: or {
            contains identifier() as $reference where {
                $reference <: `re`,
                $reference <: not within import_statement(),
                $reference <: not within import_from_statement(),
                $reference <: not within attribute(object=`re`),
                $import => `from re import $names
import re`
            },
            $_ where { $import => `from re import $names` }
        }
    }
}
```

## Qualified Map My Merchants expression

From [Map My Merchants `_platform_regex`](https://github.com/biobuddies/mapmymerchants/pull/8).
The qualified import and call are synthesized from the real expression.

```python
import re
from re import IGNORECASE, escape

pattern = re.compile('^(' + '|'.join(map(escape, patterns)) + ')[ *]+', IGNORECASE)
```

```python
from re import compile as recompile
from re import IGNORECASE, escape

pattern = recompile('^(' + '|'.join(map(escape, patterns)) + ')[ *]+', IGNORECASE)
```

## Unaliased Map My Merchants expression

Synthetic import alternative using the same real expression.

```python
from re import IGNORECASE, Pattern, compile, escape, search

pattern = compile('^(' + '|'.join(map(escape, patterns)) + ')[ *]+', IGNORECASE)
```

```python
from re import IGNORECASE, Pattern, compile as recompile, escape, search

pattern = recompile('^(' + '|'.join(map(escape, patterns)) + ')[ *]+', IGNORECASE)
```

## Preferred Map My Merchants expression stays put

```python
from re import IGNORECASE, escape
from re import compile as recompile

pattern = recompile('^(' + '|'.join(map(escape, patterns)) + ')[ *]+', IGNORECASE)
```

## Built-in compile stays put

Synthetic.

```python
code = compile('result = 1', '<input>', 'exec')
```

## Other methods and keyword names stay put

Synthetic.

```python
from re import compile

factory = compile
pattern = parser.compile('expression')
configure(compile=compile)
```

```python
from re import compile as recompile

factory = recompile
pattern = parser.compile('expression')
configure(compile=recompile)
```

## Qualified operations use bound names

Synthetic.

```python
import re
from re import compile

first = compile('first')
second = re.compile('second')
matched = re.search('first', text)
```

```python
from re import compile as recompile, search
from re import compile as recompile

first = recompile('first')
second = recompile('second')
matched = search('first', text)
```

## Unused compile import gains the alias

Synthetic.

```python
from re import compile
```

```python
from re import compile as recompile
```

## Qualified calls keep built-in compilation and the module docstring

Synthetic.

```python
"""Build patterns and Python code."""

from __future__ import annotations

import re

pattern = re.compile('first')
code = compile('result = 1', '<input>', 'exec')
```

```python
"""Build patterns and Python code."""

from __future__ import annotations

from re import compile as recompile

pattern = recompile('first')
code = compile('result = 1', '<input>', 'exec')
```

## Qualified constants and functions use bound names

Synthetic variant of the Map My Merchants expression.

```python
import re

pattern = re.compile('^(' + '|'.join(map(re.escape, patterns)) + ')[ *]+', re.IGNORECASE)
matched = re.search('first', text, re.IGNORECASE)
```

```python
from re import compile as recompile, escape, IGNORECASE, search

pattern = recompile('^(' + '|'.join(map(escape, patterns)) + ')[ *]+', IGNORECASE)
matched = search('first', text, IGNORECASE)
```

## Keep the module when used as a value

Synthetic.

```python
import re

register(re)
pattern = re.compile('first')
```

```python
from re import compile as recompile

import re

register(re)
pattern = recompile('first')
```
