"""The corked command line. Two commands, check and render."""

import json
import pathlib
import sys

from .checker import check_diff_text
from .render import render_defects, render_review
from .review import verdict_for

_EXIT_CODES = {"accepted": 0, "sent back": 1, "invalid": 2}


def _read_text(path):
    return pathlib.Path(path).read_text(encoding="utf-8")


def _check(path):
    try:
        text = _read_text(path)
    except (OSError, UnicodeDecodeError):
        print(render_defects(pathlib.Path(path).name,
                             ["the diff file could not be read"]), end="")
        return 2
    review = {
        "diff_label": pathlib.Path(path).name,
        "objections": check_diff_text(text),
    }
    print(render_review(review), end="")
    return _EXIT_CODES[verdict_for(review)]


def _render(path):
    try:
        text = _read_text(path)
    except (OSError, UnicodeDecodeError):
        print(render_defects("unnamed review",
                             ["the review file could not be read"]), end="")
        return 2
    try:
        review = json.loads(text)
    except ValueError:
        print(render_defects("unnamed review",
                             ["the file does not hold a JSON review"]), end="")
        return 2
    print(render_review(review), end="")
    return _EXIT_CODES[verdict_for(review)]


def main(argv=None):
    """Run the command line and return the exit code."""
    args = list(sys.argv[1:] if argv is None else argv)
    if len(args) == 2 and args[0] == "check":
        return _check(args[1])
    if len(args) == 2 and args[0] == "render":
        return _render(args[1])
    print("Usage, corked check DIFF_FILE", file=sys.stderr)
    print("Usage, corked render REVIEW_JSON", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
