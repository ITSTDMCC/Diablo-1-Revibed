"""Normalise `// @port` tags in src/: fill in the sha, and complete keys written as
`path|qualname` or `path|qualname(...)` when they identify exactly one database function.

  python tools/fix_tags.py          rewrite tags in place, report unresolved ones (exit 1 if any)

Overloads must be written with their full parameter list (as in port/db_functions.csv).
"""
import csv
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
rows = list(csv.DictReader(open(os.path.join(ROOT, "port", "db_functions.csv"), encoding="utf-8", newline="")))
by_key = {r["key"]: r for r in rows}
by_q = {}
for r in rows:
    by_q.setdefault((r["path"], r["qualname"]), []).append(r)

TAG = re.compile(r"^(\s*// @port )(.*?)( sha=\S*)?\s*$")
bad = 0
for dp, _, fs in os.walk(os.path.join(ROOT, "src")):
    for fn in fs:
        if not fn.endswith(".rs"):
            continue
        p = os.path.join(dp, fn)
        lines = open(p, encoding="utf-8").read().split("\n")
        changed = False
        for i, line in enumerate(lines):
            m = TAG.match(line)
            if not m:
                continue
            key = m.group(2).strip()
            r = by_key.get(key)
            if r is None:
                path, _, q = key.partition("|")
                q = re.sub(r"\(.*$", "", q)
                cands = by_q.get((path, q), [])
                if len(cands) == 1:
                    r = cands[0]
                else:
                    print(f"{os.path.relpath(p, ROOT)}:{i + 1}: unresolved tag ({len(cands)} candidates): {key}")
                    for c in cands:
                        print("    candidate:", c["key"])
                    bad += 1
                    continue
            new = f"{m.group(1)}{r['key']} sha={r['sha256'][:12]}"
            if new != line:
                lines[i] = new
                changed = True
        if changed:
            open(p, "w", encoding="utf-8", newline="").write("\n".join(lines))
sys.exit(1 if bad else 0)
