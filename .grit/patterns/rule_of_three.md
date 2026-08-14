# `rule_of_three`

## Inline a timezone-offset helper that has only one caller

```python
def _format_tz_offset(tz_name):
    """Return ISO 8601 offset like '-07:00' for a timezone name."""
    offset_str = datetime.now(ZoneInfo(tz_name)).strftime('%z')
    if not offset_str:
        return '+00:00'
    return f'{offset_str[:3]}:{offset_str[3:]}'


class StrFTime:
    def as_sqlite(self, compiler, connection):
        from django.conf import settings

        format_string = self.format_string
        tz_literal = ''
        if '%z' in format_string:
            format_string = format_string.replace('%z', '')
            tz_literal = _format_tz_offset(settings.TIME_ZONE)
        escaped = format_string.replace('%', '%%%%')
        template = f"%(function)s('{escaped}', %(expressions)s)"
        if tz_literal:
            template += f" || '{tz_literal}'"
        return self.as_sql(compiler, connection, template=template)
```

```python
class StrFTime:
    def as_sqlite(self, compiler, connection):  # noqa: ANN001, ANN201, D102  # pyrefly: ignore[bad-override]
        from django.conf import settings  # noqa: PLC0415

        format_string = self.format_string
        tz_literal = ''
        if '%z' in format_string:
            format_string = format_string.replace('%z', '')
            timezone_offset = datetime.now(ZoneInfo(settings.TIME_ZONE)).strftime('%z')
            tz_literal = (
                f'{timezone_offset[:3]}:{timezone_offset[3:]}' if timezone_offset else '+00:00'
            )
        escaped = format_string.replace('%', '%%%%')
        template = f"%(function)s('{escaped}', %(expressions)s)"
        if tz_literal:
            template += f" || '{tz_literal}'"
        return self.as_sql(compiler, connection, template=template)
```
