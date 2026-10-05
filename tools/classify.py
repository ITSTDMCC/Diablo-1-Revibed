"""Classify pending manifest rows with port/classify_rules.csv.

  python tools/classify.py [--dry-run]

Rows still `pending` whose key matches a rule's regex get that rule's status ("replaced" with
replaced_by and reason, or "pending" with a reason saying why it is not ported). The first
matching rule wins; ported rows are never touched.
"""
import csv
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MAN = os.path.join(ROOT, "port", "manifest.csv")
FIELDS = ["key", "status", "replaced_by", "reason", "rust"]


def main():
    dry = "--dry-run" in sys.argv
    rules = list(csv.DictReader(open(os.path.join(ROOT, "port", "classify_rules.csv"), encoding="utf-8")))
    for r in rules:
        r["re"] = re.compile(r["key_regex"])
    rows = list(csv.DictReader(open(MAN, encoding="utf-8", newline="")))
    counts = {}
    for row in rows:
        if row["status"] != "pending":
            continue
        for r in rules:
            if r["re"].search(row["key"]):
                row.update(status=r["status"], replaced_by=r["replaced_by"], reason=r["reason"])
                counts[r["key_regex"]] = counts.get(r["key_regex"], 0) + 1
                break
    for k, v in counts.items():
        print(f"{v:4} {k}")
    if not dry:
        with open(MAN, "w", newline="", encoding="utf-8") as f:
            w = csv.DictWriter(f, fieldnames=FIELDS, lineterminator="\n")
            w.writeheader()
            w.writerows(rows)
    left = [r["key"] for r in rows if r["status"] == "pending" and not r["reason"]]
    print(f"{len(left)} pending rows without a reason")


if __name__ == "__main__":
    main()
