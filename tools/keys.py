"""Print manifest keys, statuses and tag lines for functions whose path starts with a prefix.
Usage: python tools/keys.py <path-prefix> [...]"""
import csv
import os
import sys

PORT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "port")
status = {r["key"]: r["status"] for r in csv.DictReader(open(os.path.join(PORT, "manifest.csv"), encoding="utf-8", newline=""))}
for r in csv.DictReader(open(os.path.join(PORT, "db_functions.csv"), encoding="utf-8", newline="")):
    if any(r["path"].startswith(p) for p in sys.argv[1:]):
        print(f"[{status.get(r['key'], '?')}] {r['path']}:{r['line_start']} {r['bin_address']}\n    // @port {r['key']} sha={r['sha256'][:12]}")
