"""Update port/manifest.csv statuses.

  python tools/mark.py ported              mark every function tagged `// @port` in src/ as ported
                                           (records the Rust file in the `rust` column)
  python tools/mark.py replaced "<key>" "<replaced_by>" "<reason>"
  python tools/mark.py pending "<key>"     undo

Ported rows whose tag disappeared are reported, not silently reset (cargo test fails on them).
"""
import csv
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MAN = os.path.join(ROOT, "port", "manifest.csv")
FIELDS = ["key", "status", "replaced_by", "reason", "rust"]


def load():
    return list(csv.DictReader(open(MAN, encoding="utf-8", newline="")))


def save(rows):
    with open(MAN, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=FIELDS, lineterminator="\n")
        w.writeheader()
        w.writerows(rows)


def tags():
    found = {}
    for dp, _, fs in os.walk(os.path.join(ROOT, "src")):
        for fn in fs:
            if fn.endswith(".rs"):
                p = os.path.join(dp, fn)
                for line in open(p, encoding="utf-8"):
                    m = re.match(r"\s*// @port (.*) sha=\w+\s*$", line)
                    if m:
                        found[m.group(1)] = os.path.relpath(p, ROOT).replace("\\", "/")
    return found


def main():
    rows = load()
    by_key = {r["key"]: r for r in rows}
    cmd = sys.argv[1]
    if cmd == "ported":
        t = tags()
        changed = 0
        for k, path in t.items():
            r = by_key.get(k)
            if r is None:
                print("unknown key in tag:", k)
                continue
            if r["status"] != "ported" or r["rust"] != path:
                r.update(status="ported", rust=path, replaced_by="", reason="")
                changed += 1
        for r in rows:
            if r["status"] == "ported" and r["key"] not in t:
                print("ported but no tag:", r["key"])
        print(f"{changed} rows marked ported")
    elif cmd == "replaced":
        key, by, reason = sys.argv[2:5]
        by_key[key].update(status="replaced", replaced_by=by, reason=reason, rust="")
    elif cmd == "pending":
        by_key[sys.argv[2]].update(status="pending", replaced_by="", reason="", rust="")
    else:
        sys.exit(__doc__)
    save(rows)
    counts = {}
    for r in rows:
        counts[r["status"]] = counts.get(r["status"], 0) + 1
    print(", ".join(f"{k} {v}" for k, v in sorted(counts.items())))


if __name__ == "__main__":
    main()
