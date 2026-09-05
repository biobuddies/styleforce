---
title: Class method receiver named 'cls'
---

A `@classmethod`'s first parameter must be named `cls`, renaming every reference alongside the
parameter.

```grit
engine marzano(0.1)
language python

decorated_definition(definition=function_definition(parameters=$params)) as $method where {
    $method <: contains `@classmethod`,
    $params <: contains identifier() as $first,
    $first <: not `cls`,
    $method <: contains `$first` => `cls`
}
```

## Renames a class method's first parameter to 'cls'

```python
class Codename:
    @classmethod
    def from_environment(klass, name):
        return klass(name.upper())
```

```python
class Codename:
    @classmethod
    def from_environment(cls, name):
        return cls(name.upper())
```

## Already named 'cls' — unchanged

```python
class Codename:
    @classmethod
    def from_environment(cls, name):
        return cls(name.upper())
```

## Instance method receiver — unchanged

```python
class Codename:
    def slug(self):
        return self.name.lower()
```

## Static method — unchanged

```python
class Codename:
    @staticmethod
    def join(first, second):
        return f'{first}-{second}'
```
