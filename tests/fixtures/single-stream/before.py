def as_sqlite(self, compiler, connection):  # noqa: ANN001, ANN201  # pyrefly: ignore[bad-override]
    """Substitute %z with +00:00 in place, because SQLite formats the stored UTC value.

    STRFTIME returns NULL for %z, so %%z carries it through as a literal for REPLACE.
    Converting to an offset timezone would need the timezone database SQLite lacks, so
    %z demands a TIME_ZONE that stays on UTC year round, such as UTC or Africa/Freetown.
    """
    from django.conf import settings  # noqa: PLC0415

    escaped = self.format_string.replace('%z', '%%z').replace('%', '%%%%')
    template = f"%(function)s('{escaped}', %(expressions)s)"
    if '%z' not in self.format_string:
        return self.as_sql(compiler, connection, template=template)
    if settings.TIME_ZONE not in UTC_TIME_ZONES:
        raise NotSupportedError(
            f'%z on SQLite when TIME_ZONE={settings.TIME_ZONE} not supported yet'
        )
    return self.as_sql(compiler, connection, template=f"REPLACE({template}, '%%%%z', '+00:00')")
