//! Developer tool: field-level diff of one record of two single-player saves, walking
//! DevilutionX's memory map files (test/fixtures/memory_map) like `CreateDetailDiffs`, but
//! printing every differing field with both values instead of counting.
//!
//!   cargo run --release --example save_diff -- <reference.sv> <actual.sv> <record> <memory_map_dir> [--hellfire] [--max N]
//! (spawn saves are recognised by "spawn_" in the file name and use the spawn password).
//!
//! <record> is "hero", "game", "additionalMissiles" or a level ("perml00", ...; its map is
//! "level"). Saves are read only.

use std::collections::HashMap;

use diablo1_rs::mpq::MpqArchive;

struct Info {
    data: Vec<u8>,
    pos: usize,
    town: bool,
    exists: bool,
}

struct Walker {
    dir: String,
    hellfire: bool,
    max: usize,
    printed: usize,
    diffs: usize,
}

fn read32(i: &Info, le: bool) -> i32 {
    if !i.exists || i.pos + 4 > i.data.len() {
        return 0;
    }
    let b: [u8; 4] = i.data[i.pos..i.pos + 4].try_into().unwrap();
    if le { i32::from_le_bytes(b) } else { i32::from_be_bytes(b) }
}

fn value(i: &Info, bytes: usize) -> String {
    if !i.exists {
        return "-".into();
    }
    let s = &i.data[i.pos.min(i.data.len())..(i.pos + bytes).min(i.data.len())];
    match bytes {
        1 => format!("{}", s[0]),
        2 => format!("{}", i16::from_le_bytes([s[0], s[1]])),
        4 => format!("{} (be {})", i32::from_le_bytes([s[0], s[1], s[2], s[3]]), i32::from_be_bytes([s[0], s[1], s[2], s[3]])),
        _ => s.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(""),
    }
}

impl Walker {
    fn compare(&mut self, path: &str, r: &mut Info, a: &mut Info, bytes: usize) {
        let differs = r.exists && a.exists && r.data.get(r.pos..r.pos + bytes) != a.data.get(a.pos..a.pos + bytes);
        let one_missing = r.exists != a.exists;
        if differs || one_missing {
            self.diffs += 1;
            if self.printed < self.max {
                self.printed += 1;
                println!("{path}: reference {} actual {}  (offsets {} / {})", value(r, bytes), value(a, bytes), r.pos, a.pos);
            }
        }
        if r.exists {
            r.pos += bytes;
        }
        if a.exists {
            a.pos += bytes;
        }
    }

    fn walk(&mut self, prefix: &str, map: &str, r: &mut Info, a: &mut Info) {
        let text = std::fs::read_to_string(format!("{}/{}.txt", self.dir, map)).unwrap_or_else(|_| panic!("memory map {map} missing"));
        let mut counter: HashMap<String, (i32, i32)> = HashMap::new();
        for line in text.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if line.is_empty() {
                continue;
            }
            let split: Vec<&str> = line.split(' ').collect();
            let tok = |i: usize| split.get(i).copied().unwrap_or("");
            let mut cmd = tok(0);
            let (re, ae) = (r.exists, a.exists);
            if let Some(c) = cmd.strip_suffix("_HF") {
                if !self.hellfire {
                    continue;
                }
                cmd = c;
            }
            if let Some(c) = cmd.strip_suffix("_DA") {
                if self.hellfire {
                    continue;
                }
                cmd = c;
            }
            if let Some(c) = cmd.strip_suffix("_DL") {
                if r.town && a.town {
                    continue;
                }
                if r.town {
                    r.exists = false;
                }
                if a.town {
                    a.exists = false;
                }
                cmd = c;
            }
            let count_of = |counter: &HashMap<String, (i32, i32)>, s: &str| counter.get(s).copied().unwrap_or_else(|| {
                let n = s.trim().parse().unwrap_or(0);
                (n, n)
            });
            match cmd {
                "R" | "LT" | "LC" | "LC_LE" => {
                    let bytes = tok(1).trim().parse::<usize>().unwrap_or(0) / 8;
                    let comment = tok(2).to_string();
                    if cmd == "LT" {
                        r.town = read32(r, false) == 0;
                        a.town = read32(a, false) == 0;
                    }
                    if cmd == "LC" || cmd == "LC_LE" {
                        counter.insert(comment.clone(), (read32(r, cmd == "LC_LE"), read32(a, cmd == "LC_LE")));
                    }
                    self.compare(&format!("{prefix}.{comment}"), r, a, bytes);
                }
                "M" => {
                    let (cr, ca) = count_of(&counter, tok(1));
                    let bytes = tok(2).trim().parse::<usize>().unwrap_or(0) / 8;
                    let comment = tok(3);
                    for i in 0..cr.max(ca) {
                        if cr == i {
                            r.exists = false;
                        }
                        if ca == i {
                            a.exists = false;
                        }
                        self.compare(&format!("{prefix}.{comment}[{i}]"), r, a, bytes);
                    }
                }
                "C" => {
                    let (cr, ca) = count_of(&counter, tok(1));
                    let sub = tok(2).to_string();
                    let comment = tok(3);
                    for i in 0..cr.max(ca) {
                        if cr == i {
                            r.exists = false;
                        }
                        if ca == i {
                            a.exists = false;
                        }
                        self.walk(&format!("{prefix}.{comment}[{i}]"), &sub, r, a);
                    }
                }
                _ => {}
            }
            r.exists = re;
            a.exists = ae;
        }
    }
}

fn read_record(path: &str, name: &str, password: &str) -> Vec<u8> {
    let mut archive = MpqArchive::open(path).expect("open save").expect("save exists");
    let mut data = archive.read_file(name).unwrap_or_default();
    if data.is_empty() {
        return data;
    }
    let n = diablo1_rs::codec::codec_decode(&mut data, password);
    data.truncate(n);
    data
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (reference, actual, record, dir) = (&args[1], &args[2], &args[3], &args[4]);
    let hellfire = args.iter().any(|a| a == "--hellfire");
    let max = args.iter().position(|a| a == "--max").map(|i| args[i + 1].parse().unwrap()).unwrap_or(60);
    let map = if record.starts_with("perm") || record.starts_with("temp") { "level" } else { record.as_str() };
    // PASSWORD_SINGLE, or PASSWORD_SPAWN_SINGLE for spawn saves
    let password = if reference.contains("spawn_") { "adslhfb1" } else { "xrgyrkj1" };
    let rdata = read_record(reference, record, password);
    let adata = read_record(actual, record, password);
    println!("{record}: reference {} bytes, actual {} bytes", rdata.len(), adata.len());
    let town = record == "perml00";
    let mut r = Info { exists: !rdata.is_empty(), data: rdata, pos: 0, town };
    let mut a = Info { exists: !adata.is_empty(), data: adata, pos: 0, town };
    let mut w = Walker { dir: dir.clone(), hellfire, max, printed: 0, diffs: 0 };
    w.walk(record, map, &mut r, &mut a);
    println!("{} differing fields; walked {} / {} bytes", w.diffs, r.pos, a.pos);
}
