"""Add `// @port <key>` tags for pending manifest functions that already exist in Rust under
their snake_case name.

  python tools/auto_tag.py [--dry-run]

For each pending key `file|devilution::Class::Method(args)` the Rust module for `file` is
searched for `fn method(` (snake_case of the C++ name). A tag is added only when
  * exactly one such function exists in that module,
  * the C++ name is not overloaded among the database keys of that file, and
  * the Rust function does not already carry a tag for another function of the same name.
Constructors, destructors and operators are skipped. Run tools/fix_tags.py and
tools/mark.py ported afterwards; cargo test then checks every tag against the database.
"""
import csv
import os
import re
import sys
from collections import Counter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "src")


def snake(name):
    s = re.sub(r"(?<=[a-z0-9])([A-Z])", r"_\1", name)
    s = re.sub(r"(?<=[A-Z])([A-Z][a-z])", r"_\1", s)
    return s.lower()


def cpp_name(key):
    sig = key.split("|", 1)[1]
    qual = sig.split("(", 1)[0]
    qual = re.sub(r"<[^>]*>", "", qual)
    parts = qual.split("::")
    return parts[-1], parts[-2] if len(parts) > 1 else ""


def rust_files(cpp_file):
    stem = os.path.splitext(cpp_file)[0]
    cands = [stem, stem.replace("DiabloUI/", "diablo_ui/").replace("DiabloUI/multi/", "diablo_ui/")]
    out = []
    for c in cands:
        c = c.replace("diablo_ui/multi/", "diablo_ui/")
        p = os.path.join(SRC, *c.split("/")) + ".rs"
        if os.path.exists(p) and p not in out:
            out.append(p)
    return out


def main():
    dry = "--dry-run" in sys.argv
    manifest = list(csv.DictReader(open(os.path.join(ROOT, "port", "manifest.csv"), encoding="utf-8")))
    names_per_file = Counter()
    for r in manifest:
        f = r["key"].split("|")[0]
        n, _ = cpp_name(r["key"])
        names_per_file[(f, n)] += 1
    edits = {}
    added = 0
    for r in manifest:
        if r["status"] != "pending":
            continue
        key = r["key"]
        f = key.split("|")[0]
        name, cls = cpp_name(key)
        if name.startswith("operator") or name.startswith("~") or name == cls or not re.fullmatch(r"\w+", name):
            continue
        if names_per_file[(f, name)] != 1:
            continue
        rname = snake(name)
        for path in rust_files(f):
            text = edits.get(path) or open(path, encoding="utf-8").read()
            hits = [m for m in re.finditer(r"^([ \t]*)(pub(\([^)]*\))? )?(const )?(unsafe )?fn " + re.escape(rname) + r"\b", text, re.M)]
            if len(hits) != 1:
                continue
            m = hits[0]
            # Insert above attributes directly preceding the fn.
            start = m.start()
            lines_before = text[:start].split("\n")
            insert_at = start
            i = len(lines_before) - 2
            while i >= 0 and lines_before[i].strip().startswith(("#[", "// @port")):
                if lines_before[i].strip().startswith("// @port") and lines_before[i].split("|", 1)[-1].split("(")[0].endswith("::" + name):
                    break
                insert_at -= len(lines_before[i]) + 1
                i -= 1
            else:
                pass
            indent = m.group(1)
            tag = f"{indent}// @port {key}\n"
            text = text[:insert_at] + tag + text[insert_at:]
            edits[path] = text
            added += 1
            print(f"{os.path.relpath(path, ROOT)}: {rname} <- {key}")
            break
    if not dry:
        for path, text in edits.items():
            open(path, "w", encoding="utf-8", newline="\n").write(text)
    print(f"{added} tags {'would be ' if dry else ''}added")


if __name__ == "__main__":
    main()
