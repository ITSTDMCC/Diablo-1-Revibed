"""Regenerate port/db_functions.csv from the reference database and merge port/manifest.csv.

  python tools/gen_manifest.py

db_functions.csv  every source function in the database: key, qualname, path, lines, sha256, binary address.
manifest.csv      every key from db_functions.csv with a status: ported | replaced | pending.
                  Existing statuses are kept. New keys start as `pending`, or `replaced` when a rule in
                  port/replace_rules.csv matches their path. Keys that vanished from the database are
                  kept with status `removed` so nothing disappears silently; the parity test fails on them.
Safe to re-run.
"""
import csv
import os
import sqlite3
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORT = os.path.join(ROOT, "port")
DB = os.path.join(os.path.dirname(ROOT), "portdb", "diablo1.sqlite")


def key_of(path, qualname, params):
    return f"{path}|{qualname}({' '.join(params.split())})"


def main():
    if not os.path.exists(DB):
        sys.exit(f"missing {DB}; run portdb/build_port_db.py")
    con = sqlite3.connect(f"file:{DB}?mode=ro", uri=True)
    addr = {}
    for a, fid in con.execute("SELECT address, function_id FROM port_bin_map ORDER BY address"):
        addr.setdefault(fid, []).append("0x" + a)
    rows, seen = [], {}
    for fid, q, path, ls, le, params, h in con.execute(
            "SELECT id, qualname, path, line_start, line_end, params, sha256 FROM port_functions ORDER BY path, start"):
        k = key_of(path, q, params)
        seen[k] = seen.get(k, 0) + 1
        if seen[k] > 1:  # identical declarations (e.g. both branches of an #if) get an ordinal
            k += f"#{seen[k]}"
        rows.append(dict(key=k, qualname=q, path=path, line_start=ls, line_end=le, sha256=h, bin_address=";".join(addr.get(fid, []))))
    with open(os.path.join(PORT, "db_functions.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0]), quoting=csv.QUOTE_MINIMAL, lineterminator="\n")
        w.writeheader()
        w.writerows(rows)

    rules = list(csv.DictReader(open(os.path.join(PORT, "replace_rules.csv"), encoding="utf-8", newline="")))
    man_path = os.path.join(PORT, "manifest.csv")
    old = {}
    if os.path.exists(man_path):
        old = {r["key"]: r for r in csv.DictReader(open(man_path, encoding="utf-8", newline=""))}
    out, counts = [], {}
    for r in rows:
        k = r["key"]
        if k in old and old[k]["status"] != "removed":
            m = old[k]
        else:
            rule = next((x for x in rules if r["path"].startswith(x["path_prefix"])), None)
            m = dict(key=k, status="replaced" if rule else "pending",
                     replaced_by=rule["replaced_by"] if rule else "", reason=rule["reason"] if rule else "", rust="")
        out.append(m)
        counts[m["status"]] = counts.get(m["status"], 0) + 1
    current = {r["key"] for r in rows}
    for k, m in old.items():
        if k not in current:
            m = dict(m, status="removed")
            out.append(m)
            counts["removed"] = counts.get("removed", 0) + 1
    with open(man_path, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=["key", "status", "replaced_by", "reason", "rust"], lineterminator="\n")
        w.writeheader()
        w.writerows(out)
    print(f"{len(rows)} functions; " + ", ".join(f"{k} {v}" for k, v in sorted(counts.items())))


if __name__ == "__main__":
    main()
