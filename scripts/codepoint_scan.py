#!/usr/bin/env python3
"""Count U+2014 and U+2013 in the text files this repo carries.
The scan covers files whose suffix is .md, .txt, .py, .json, .toml,
.yml, .yaml, or .rs, and extensionless text files such as LICENSE.
Zero of each is required. Exit 1 when any are found. Files inside
ignored environment and build directories are skipped."""
import pathlib, sys
root = pathlib.Path(__file__).resolve().parent.parent
counts = {"U+2014": 0, "U+2013": 0}
ignored_dirs = {".git", ".venv", "venv", "dist", "build", "node_modules", "__pycache__"}
text_suffixes = {".md", ".txt", ".py", ".json", ".toml", ".yml", ".yaml", ".rs", ""}
for p in sorted(root.rglob("*")):
    if not p.is_file():
        continue
    rel = p.relative_to(root)
    if any(part in ignored_dirs or part.endswith(".egg-info") for part in rel.parts[:-1]):
        continue
    if p.suffix not in text_suffixes:
        continue
    text = p.read_text(encoding="utf-8", errors="replace")
    counts["U+2014"] += text.count("\u2014")
    counts["U+2013"] += text.count("\u2013")
print(f"codepoint scan U+2014={counts['U+2014']} U+2013={counts['U+2013']}")
sys.exit(1 if any(counts.values()) else 0)
