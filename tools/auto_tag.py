"""Add `// @port <key>` tags for pending manifest functions that already exist in Rust under
their snake_case name.

  python tools/auto_tag.py [--dry-run] [--crate-wide] [--names=a,b]

For each pending key `file|devilution::Class::Method(args)` the Rust module for `file` is
searched for `fn method(` (snake_case of the C++ name). A tag is added only when
  * exactly one such function exists in that module,
  * the C++ name is not overloaded among the database keys of that file, and
  * the Rust function does not already carry a tag for another function of the same name.
With --crate-wide, a name of two or more words that has no match in its module may match a
function anywhere in the crate, if exactly one exists (review the --dry-run output first).
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
    cands = [stem, stem.replace("DiabloUI/", "diablo_ui/")]
    out = []
    for c in cands:
        c = c.replace("diablo_ui/multi/", "diablo_ui/")
        p = os.path.join(SRC, *c.split("/")) + ".rs"
        if os.path.exists(p) and p not in out:
            out.append(p)
    return out


def all_rust_files():
    out = []
    for d, _, fs in os.walk(SRC):
        out.extend(os.path.join(d, f) for f in fs if f.endswith(".rs"))
    return sorted(out)


def fn_pattern(rname):
    return re.compile(r"^([ \t]*)(pub(\([^)]*\))? )?(const )?(unsafe )?fn " + re.escape(rname) + r"\b", re.M)


def main():
    dry = "--dry-run" in sys.argv
    crate_wide = "--crate-wide" in sys.argv
    # --names a,b: only these Rust names (to apply a reviewed subset of a --crate-wide dry run)
    only = next((a.split("=", 1)[1].split(",") for a in sys.argv if a.startswith("--names=")), None)
    manifest = list(csv.DictReader(open(os.path.join(ROOT, "port", "manifest.csv"), encoding="utf-8")))
    names_per_file = Counter()
    for r in manifest:
        f = r["key"].split("|")[0]
        n, _ = cpp_name(r["key"])
        names_per_file[(f, n)] += 1
    edits = {}

    def text_of(p):
        return edits.get(p) or open(p, encoding="utf-8").read()

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
        if only is not None and rname not in only:
            continue
        pat = fn_pattern(rname)
        paths = rust_files(f)
        if crate_wide:
            if "_" not in rname or any(pat.search(text_of(p)) for p in paths):
                continue
            found = [p for p in all_rust_files() if pat.search(text_of(p))]
            paths = found if len(found) == 1 else []
        for path in paths:
            text = text_of(path)
            hits = list(pat.finditer(text))
            if len(hits) != 1:
                continue
            m = hits[0]
            # Insert above attributes and tags directly preceding the fn.
            start = m.start()
            lines_before = text[:start].split("\n")
            insert_at = start
            i = len(lines_before) - 2
            while i >= 0 and lines_before[i].strip().startswith(("#[", "// @port")):
                insert_at -= len(lines_before[i]) + 1
                i -= 1
            tag = f"{m.group(1)}// @port {key}\n"
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
