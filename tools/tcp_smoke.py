"""Two-player TCP smoke test: a host creates a game, a guest joins over 127.0.0.1, walks, and
leaves. Screenshots from both games go to OUT_DIR (they are game data: do not commit them).

  python tools/tcp_smoke.py DATA_DIR OUT_DIR [--password]

DATA_DIR is the folder with DIABDAT.MPQ. Builds nothing: run `cargo build --release` first.
Each game gets a fresh save/config folder under OUT_DIR with the server bound to 127.0.0.1, so
no firewall prompt appears. Uses DIABLO_FIXED_STEP=pace so both games keep to the real clock
(the network timeouts are real-time). Takes about two minutes.
"""
import os
import shutil
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXE = os.path.join(ROOT, "target", "release", "diablo1_rs" + (".exe" if os.name == "nt" else ""))
SCRIPTS = os.path.join(ROOT, "tools", "input_scripts")


def game(name, data_dir, out_dir, script, frames, max_frames):
    home = os.path.join(out_dir, name)
    shutil.rmtree(home, ignore_errors=True)
    os.makedirs(home)
    with open(os.path.join(home, "diablo.ini"), "w") as f:
        f.write("[Network]\nBind Address=127.0.0.1\n")
    env = dict(os.environ)
    env.update({
        "DIABLO_HEADLESS": "1",
        "DIABLO_FIXED_STEP": "pace",
        "DIABLO_NO_AUDIO": "1",
        "DIABLO_TIME": "1700000000",
        "DIABLO_MAX_FRAMES": str(max_frames),
        "DIABLO_INPUT_SCRIPT": os.path.join(SCRIPTS, script),
        "DIABLO_SCREENSHOT_FRAMES": ",".join(map(str, frames)),
        "DIABLO_SCREENSHOT_DIR": os.path.join(out_dir, name + "_shots"),
    })
    log = open(os.path.join(out_dir, name + ".log"), "w")
    args = [EXE, "-n", "--data-dir", data_dir, "--save-dir", home, "--config-dir", home]
    return subprocess.Popen(args, env=env, stdout=log, stderr=subprocess.STDOUT)


def main():
    data_dir, out_dir = sys.argv[1], sys.argv[2]
    pw = "--password" in sys.argv
    suffix = "_password" if pw else ""
    os.makedirs(out_dir, exist_ok=True)
    host = game("host", data_dir, out_dir, f"tcp_host{suffix}.txt", [4500, 5000, 5500, 6000], 7000)
    time.sleep(45)  # the host creates the game first
    guest = game("guest", data_dir, out_dir, f"tcp_guest{suffix}.txt", [2100, 2500, 2990], 3000)
    guest_rc = guest.wait()
    host_rc = host.wait()
    print(f"guest exit {guest_rc}, host exit {host_rc}; screenshots in {out_dir}")
    sys.exit(0 if guest_rc == 0 and host_rc == 0 else 1)


if __name__ == "__main__":
    main()
