"""Pins for SETUP-3 as amended by FIX-CK-7. The scan runs as a
subprocess against a planted tree, so each case can fail for a real
reason in either direction."""

import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
EM_DASH = "\u2014"
EN_DASH = "\u2013"


def planted_tree(tmp_path, files):
    scripts = tmp_path / "scripts"
    scripts.mkdir(parents=True)
    shutil.copy(ROOT / "scripts" / "codepoint_scan.py",
                scripts / "codepoint_scan.py")
    for relative, text in files.items():
        path = tmp_path / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    return subprocess.run(
        [sys.executable, str(scripts / "codepoint_scan.py")],
        capture_output=True, text=True)


def test_a_dash_in_an_extensionless_file_fails_the_scan(tmp_path):
    result = planted_tree(tmp_path, {"NOTES": f"a note {EM_DASH} held back\n"})
    assert result.returncode == 1
    assert "U+2014=1" in result.stdout


def test_a_dash_in_a_python_file_fails_the_scan(tmp_path):
    result = planted_tree(tmp_path, {"app.py": f"# a note {EN_DASH} held\n"})
    assert result.returncode == 1
    assert "U+2013=1" in result.stdout


def test_dashes_only_under_an_ignored_directory_pass_the_scan(tmp_path):
    result = planted_tree(
        tmp_path, {".venv/notes.md": f"a note {EM_DASH} and {EN_DASH}\n"})
    assert result.returncode == 0
    assert "U+2014=0" in result.stdout
    assert "U+2013=0" in result.stdout


def test_a_clean_tree_passes_the_scan(tmp_path):
    result = planted_tree(tmp_path, {"app.py": "total = 1\n",
                                     "README.md": "plain words\n"})
    assert result.returncode == 0
    assert "U+2014=0" in result.stdout
    assert "U+2013=0" in result.stdout


def test_a_dash_in_a_rust_file_fails_the_scan(tmp_path):
    result = planted_tree(tmp_path, {"lib.rs": f"// a note {EM_DASH} held\n"})
    assert result.returncode == 1
    assert "U+2014=1" in result.stdout
