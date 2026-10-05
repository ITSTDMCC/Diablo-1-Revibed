"""Print manifest keys for C function names.

  python tools/keyfind.py Name1 Name2 ...

Matches the last qualified-name component exactly (e.g. `CheckCursMove`, `Monster::tag`).
"""
import csv
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
rows = list(csv.DictReader(open(os.path.join(ROOT, "port", "db_functions.csv"), encoding="utf-8")))
status = {r["key"]: r["status"] for r in csv.DictReader(open(os.path.join(ROOT, "port", "manifest.csv"), encoding="utf-8"))}
for name in sys.argv[1:]:
    hits = []
    for r in rows:
        key = r["key"]
        qual = key.split("|", 1)[1].split("(", 1)[0]
        if qual.endswith("::" + name) or qual == name:
            hits.append(key)
    if not hits:
        print(f"{name}: NOT FOUND")
    for k in hits:
        print(f"{name}: {k}  [{status.get(k, '?')}]")
