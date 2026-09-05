---
title: Instance method receiver named 'self'
---

An instance method's first parameter must be named `self`, renaming every reference. Class and
static methods keep their own receivers, and nested functions are left alone.

```grit
engine marzano(0.1)
language python

function_definition(parameters=$params) as $method where {
    $method <: within class_definition(),
    $method <: not within function_definition() as $enclosing where {
        $enclosing <: not $method
    },
    $method <: not within decorated_definition(definition=$defined) as $decorated where {
        $defined <: $method,
        $decorated <: contains or { `@classmethod`, `@staticmethod` }
    },
    $params <: contains identifier() as $first,
    $first <: not `self`,
    $method <: contains `$first` => `self`
}
```

## Renames an instance method's first parameter to 'self'

```python
class Codename:
    def slug(this):
        return this.name.lower()
```

```python
class Codename:
    def slug(self):
        return self.name.lower()
```

## Renames a property receiver too

```python
class Codename:
    @property
    def slug(this):
        return this.name.lower()
```

```python
class Codename:
    @property
    def slug(self):
        return self.name.lower()
```

## Class method receiver — unchanged

```python
class Codename:
    @classmethod
    def from_environment(cls, name):
        return cls(name.upper())
```

## Static method — unchanged

```python
class Codename:
    @staticmethod
    def join(first, second):
        return f'{first}-{second}'
```

## Nested function — unchanged

```python
class Codename:
    def build(self):
        def helper(value):
            return value + 1

        return helper
```
