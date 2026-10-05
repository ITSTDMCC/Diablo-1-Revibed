"""Apply port/manual_tags.csv: hand-checked pairs of a database key and the Rust function that
ports it under a different name or with merged overloads.

  python tools/manual_tags.py

Each row gives the Rust file, the fn name and which occurrence of `fn <name>` in the file
(0 = first). Keys already tagged anywhere are skipped. Run tools/fix_tags.py afterwards.
"""
import csv
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def main():
    tagged = set()
    for d, _, fs in os.walk(os.path.join(ROOT, "src")):
        for f in fs:
            if f.endswith(".rs"):
                for m in re.finditer(r"// @port (.*?)(?: sha=\w+)?$", open(os.path.join(d, f), encoding="utf-8").read(), re.M):
                    tagged.add(m.group(1))
    n = 0
    for r in csv.DictReader(open(os.path.join(ROOT, "port", "manual_tags.csv"), encoding="utf-8")):
        if r["key"] in tagged:
            continue
        path = os.path.join(ROOT, r["rust_file"])
        text = open(path, encoding="utf-8").read()
        pat = re.compile(r"^([ \t]*)(pub(\([^)]*\))? )?(const )?(unsafe )?fn " + re.escape(r["fn_name"]) + r"\b", re.M)
        hits = list(pat.finditer(text))
        m = hits[int(r["occurrence"])]
        insert_at = m.start()
        lines_before = text[:insert_at].split("\n")
        i = len(lines_before) - 2
        while i >= 0 and lines_before[i].strip().startswith(("#[", "// @port")):
            insert_at -= len(lines_before[i]) + 1
            i -= 1
        text = text[:insert_at] + f"{m.group(1)}// @port {r['key']}\n" + text[insert_at:]
        open(path, "w", encoding="utf-8", newline="\n").write(text)
        n += 1
        print(f"{r['rust_file']}: {r['fn_name']} <- {r['key']}")
    print(f"{n} tags added")


if __name__ == "__main__":
    main()
