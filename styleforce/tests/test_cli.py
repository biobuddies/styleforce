"""Exercise the command-line interface."""

from pathlib import Path

from pytest import CaptureFixture, MonkeyPatch

from styleforce import cli


def test_main_runs_every_pattern_on_every_file(
    capsys: CaptureFixture[str], monkeypatch: MonkeyPatch, tmp_path: Path
) -> None:
    filenames = [tmp_path / 'first.py', tmp_path / 'second.py']
    for filename in filenames:
        filename.write_text('original')
    calls = []

    def append_pattern(pattern: str, source: str, filename: str) -> str:
        calls.append((pattern, filename))
        return f'{source}\nchanged'

    monkeypatch.setattr(cli, 'apply', append_pattern)

    cli.main([str(filename) for filename in filenames])

    assert len(calls) == 16
    assert [filename for _, filename in calls] == [str(filenames[0])] * 8 + [str(filenames[1])] * 8
    assert all(pattern.startswith('engine marzano') for pattern, _ in calls)
    assert [filename.read_text() for filename in filenames] == [
        'original' + '\nchanged' * 8,
        'original' + '\nchanged' * 8,
    ]
    assert capsys.readouterr().out == (f'Reformatted 2 files:\n{filenames[0]}\n{filenames[1]}\n')
