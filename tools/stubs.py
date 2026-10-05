"""Add pending (not yet ported) function stubs to module files.

  python tools/stubs.py spec.txt

Each spec line:  <module path, e.g. qol/xpbar> | <rust signature without body> | <manifest key>
Creates src/<module>.rs (and parent mod.rs declarations) if missing, and appends a
`pending_fn!` for each signature whose function name is not yet defined in that file.
Lines starting with '#' are ignored. Also lists, with --report, every pending_fn in src/.
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "src")


def ensure_module(mod):
    parts = mod.split("/")
    for depth in range(1, len(parts) + 1):
        sub = parts[:depth]
        parent_dir = os.path.join(SRC, *sub[:-1])
        parent_file = os.path.join(SRC, "lib.rs") if depth == 1 else (
            os.path.join(parent_dir, "mod.rs") if os.path.exists(os.path.join(parent_dir, "mod.rs"))
            else os.path.join(SRC, *sub[:-2], sub[-2] + ".rs"))
        name = sub[-1]
        is_leaf = depth == len(parts)
        target = os.path.join(parent_dir, name + ".rs") if is_leaf else os.path.join(parent_dir, name, "mod.rs")
        if not is_leaf and os.path.exists(os.path.join(parent_dir, name + ".rs")):
            target = os.path.join(parent_dir, name + ".rs")
        if not os.path.exists(target) and not (is_leaf and os.path.exists(os.path.join(parent_dir, name, "mod.rs"))):
            os.makedirs(os.path.dirname(target), exist_ok=True)
            src_name = "/".join(sub)
            open(target, "w", encoding="utf-8").write(f"//! `Source/{src_name}` (pending functions are declared with `pending_fn!`).\n\n#[allow(unused_imports)]\nuse crate::ctx::Ctx;\n")
        if not os.path.exists(parent_file):
            os.makedirs(os.path.dirname(parent_file), exist_ok=True)
            open(parent_file, "w", encoding="utf-8").write("")
        ptext = open(parent_file, encoding="utf-8").read()
        if not re.search(rf"^\s*pub mod {re.escape(name)};", ptext, re.M):
            ptext = ptext.rstrip("\n") + f"\npub mod {name};\n"
            open(parent_file, "w", encoding="utf-8").write(ptext)
    leaf = os.path.join(SRC, *parts[:-1], parts[-1] + ".rs")
    return leaf if os.path.exists(leaf) else os.path.join(SRC, *parts, "mod.rs")


def main():
    if sys.argv[1] == "--report":
        n = 0
        for dp, _, fs in os.walk(SRC):
            for fn in fs:
                if fn.endswith(".rs"):
                    n += len(re.findall(r"pending_fn!\(", open(os.path.join(dp, fn), encoding="utf-8").read()))
        print(f"{n} pending stubs")
        return
    for line in open(sys.argv[1], encoding="utf-8"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        mod, sig, key = [x.strip() for x in line.split("|", 2)]
        path = ensure_module(mod)
        text = open(path, encoding="utf-8").read()
        name = re.search(r"fn\s+(\w+)", sig).group(1)
        if re.search(rf"\bfn\s+{name}\b", text):
            continue
        text = text.rstrip("\n") + f"\n\ncrate::pending_fn!({sig}, \"{key}\");\n"
        open(path, "w", encoding="utf-8").write(text)


if __name__ == "__main__":
    main()
