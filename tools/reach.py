"""List the functions reachable from a root function that are still pending, grouped by file.

  python tools/reach.py "devilution::StartGame" [--depth N] [--list FILE]

Uses the call graph in working/portdb/diablo1.sqlite (name-resolved, so it over-approximates:
every overload/candidate of an ambiguous call is followed) and statuses from port/manifest.csv.
"""
import collections
import csv
import os
import sqlite3
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DB = os.path.join(ROOT, "..", "portdb", "diablo1.sqlite")

args = sys.argv[1:]
root = args[0]
depth = 99
list_file = None
if "--depth" in args:
    depth = int(args[args.index("--depth") + 1])
if "--list" in args:
    list_file = args[args.index("--list") + 1]

status = {r["key"]: r["status"] for r in csv.DictReader(open(os.path.join(ROOT, "port", "manifest.csv"), encoding="utf-8"))}
keys = {r["key"]: r for r in csv.DictReader(open(os.path.join(ROOT, "port", "db_functions.csv"), encoding="utf-8"))}

con = sqlite3.connect(DB)
fn = {}
for fid, qual, path, sig, ls, le in con.execute("SELECT id, qualname, path, signature, line_start, line_end FROM port_functions"):
    fn[fid] = (qual, path, sig, ls, le)
key_of = {}
for k, r in keys.items():
    key_of[(r["path"], r["qualname"], int(r["line_start"]))] = k
callees = collections.defaultdict(set)
for caller, callee in con.execute("SELECT caller_id, callee_id FROM port_calls WHERE callee_id IS NOT NULL"):
    callees[caller].add(callee)
# ambiguous calls: follow every function with that name
names = collections.defaultdict(set)
for fid, (qual, *_rest) in fn.items():
    names[qual.split("::")[-1]].add(fid)
for caller, text, res in con.execute("SELECT caller_id, callee_text, resolution FROM port_calls WHERE resolution IN ('ambiguous','overloaded')"):
    callees[caller] |= names.get(text.split("::")[-1], set())


def key(fid):
    q, p, s, ls, le = fn[fid]
    return key_of.get((p, q, ls))


start = [fid for fid, v in fn.items() if v[0] == root]
seen = {}
queue = collections.deque((s, 0) for s in start)
while queue:
    f, d = queue.popleft()
    if f in seen:
        continue
    seen[f] = d
    k = key(f)
    st = status.get(k, "?")
    if st == "replaced":
        continue
    if d >= depth:
        continue
    for c in callees[f]:
        if c not in seen:
            queue.append((c, d + 1))

by_file = collections.defaultdict(list)
for f in seen:
    k = key(f)
    if status.get(k) == "pending":
        q, p, s, ls, le = fn[f]
        by_file[p].append((ls, le, q, k))
total_lines = 0
rows = []
for p, fs in by_file.items():
    lines = sum(le - ls + 1 for ls, le, *_ in fs)
    total_lines += lines
    rows.append((lines, p, len(fs)))
for lines, p, n in sorted(rows, reverse=True):
    print(f"{lines:6d} lines {n:4d} fns  {p}")
print(f"total pending reachable: {sum(len(v) for v in by_file.values())} functions, {total_lines} lines")
if list_file:
    with open(list_file, "w", encoding="utf-8") as out:
        for p in sorted(by_file):
            for ls, le, q, k in sorted(by_file[p]):
                out.write(f"{k}\n")
