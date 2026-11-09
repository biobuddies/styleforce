# styleforce

Autoformat the Rule of Three and more: GritQL [patterns](styleforce/.grit/patterns) rewrite Python
in place.

```bash
uvx styleforce file.py
```

See [TODO.md](TODO.md) for rewrites not yet covered.

## Imports

Prefer direct imports: `from module import name`. Use these aliases for common collisions:

| Import | Meaning |
| --- | --- |
| `from asyncio import sleep as asleep` | Async sleep that yields to the event loop |
| `from time import sleep as bsleep` | Blocking sleep |
| `from re import compile as recompile` | Regex compilation without shadowing built-in `compile` |

Ruff isort merges and sorts imports; F401 removes unused module imports after rewriting.
