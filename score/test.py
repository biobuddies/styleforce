"""Assignment scores for candidate rewrites."""

from pytest import mark, raises

from score import score_assignments


# These fixtures could evolve to GritQL-style .md files with input code, expected output
# code, and commentary.
@mark.parametrize(
    ('source', 'expected'),
    (
        (
            """record, metadata = (
    '.dist-info/'.join((findall(r'[^-]+-[^-]+', wheel.name)[0], name))
    for name in ('RECORD', 'METADATA')
)""",
            {'record': 2, 'metadata': 1, 'name': 4},
        ),
        ('first = 1', {'first': 2}),
        ('(first) = 1', {'first': 2}),
        ('first = second = third = 1', {'first': 2, 'second': 1, 'third': 1}),
        ('(first, second), third = data', {'first': 2, 'second': 1, 'third': 1}),
        ('first = first = 1', {'first': 3}),
        ('first = second, third = data', {'first': 2, 'second': 1, 'third': 1}),
        ('first = second = 1', {'first': 2, 'second': 1}),
        ('first, second = data', {'first': 2, 'second': 1}),
        ('first = call(one, two + three)', {'first': 2}),
        ('result = (saved := 1)', {'result': 2, 'saved': 2}),
        ('print(value)', {}),
        ('first, (second, *rest) = data', {'first': 2, 'second': 1, 'rest': 1}),
        (
            'result = [left + right for left in (1, 2) for right in (3, 4, 5)]',
            {'result': 2, 'left': 4, 'right': 12},
        ),
        ('result = [(saved := value) for value in (1, 2)]', {'result': 2, 'saved': 4, 'value': 4}),
        ('result = [value for value in ()]', {'result': 2}),
        ('result = {value for value in [1, 2]}', {'result': 2, 'value': 4}),
        (
            'result = {value: (saved := value) for value in "ab"}',
            {'result': 2, 'value': 4, 'saved': 4},
        ),
        (
            'result = [left for left, right in ((1, 2), (3, 4))]',
            {'result': 2, 'left': 4, 'right': 2},
        ),
        (
            'result = [[inner for inner in b"ab"] for outer in (1, 2)]',
            {'result': 2, 'inner': 8, 'outer': 4},
        ),
        ('value: int', {}),
        ('value: int = 1\nvalue += 1', {'value': 4}),
        ('record.name = 1\nrecord[0] = 2', {'record.name': 2, 'record[0]': 2}),
    ),
)
def test_score_assignments(source: str, expected: dict[str, int]):
    assert score_assignments(source) == expected


@mark.parametrize(
    'source',
    (
        'result = [value for value in values]',
        'result = [value for value in (1, 2) if value]',
        'result = [value for value in (*values,)]',
        'for value in values:\n    result = value',
        'if condition:\n    result = 1',
        'def function():\n    result = 1',
        'result = condition and (saved := 1)',
        'result = [(saved := 1) if condition else 2]',
        'result = lambda: (saved := 1)',
        'result = [value async for value in (1, 2)]',
        'import package',
        'del value',
    ),
)
def test_reject_unknown_scores(source: str):
    with raises(ValueError, match=r'Iteration count|Unsupported|unsupported'):
        score_assignments(source)
