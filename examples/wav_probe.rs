//! Developer probe: summarise the WAV formats used by the game's sounds and music.
//! cargo run --example wav_probe -- <data dir>
use std::collections::BTreeMap;

use diablo1_rs::effects_data::SFX_DATA;
use diablo1_rs::mpq::MpqArchive;

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("data dir"));
    let mut archives: Vec<MpqArchive> = ["DIABDAT.MPQ", "hellfire.mpq", "hfvoice.mpq", "hfmusic.mpq", "hfmonk.mpq"]
        .iter()
        .filter_map(|n| MpqArchive::open(dir.join(n)).ok().flatten())
        .collect();
    let mut names: Vec<String> = SFX_DATA.iter().map(|(_, n)| n.to_string()).collect();
    for m in ["dtowne", "dlvla", "dlvlb", "dlvlc", "dlvld", "dlvlf", "dlvle", "dintro"] {
        names.push(format!("music\\{m}.wav"));
    }
    let mut hist: BTreeMap<(u16, u16, u32, u16), u32> = BTreeMap::new();
    let mut missing = 0;
    for name in &names {
        let Some(data) = archives.iter_mut().find_map(|a| a.read_file(name).ok()) else {
            missing += 1;
            continue;
        };
        let mut p = 12;
        while p + 8 <= data.len() {
            let id = &data[p..p + 4];
            let len = u32::from_le_bytes(data[p + 4..p + 8].try_into().unwrap()) as usize;
            if id == b"fmt " {
                let f = &data[p + 8..];
                let tag = u16::from_le_bytes([f[0], f[1]]);
                let ch = u16::from_le_bytes([f[2], f[3]]);
                let rate = u32::from_le_bytes([f[4], f[5], f[6], f[7]]);
                let bits = u16::from_le_bytes([f[14], f[15]]);
                *hist.entry((tag, ch, rate, bits)).or_default() += 1;
                break;
            }
            p += 8 + len + (len & 1);
        }
    }
    println!("missing {missing}");
    for ((tag, ch, rate, bits), n) in hist {
        println!("format {tag:#06x} channels {ch} rate {rate} bits {bits}: {n} files");
    }
}
