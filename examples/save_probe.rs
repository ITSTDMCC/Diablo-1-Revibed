//! Checks the save file pipeline against real DevilutionX saves (read-only):
//!   cargo run --example save_probe -- <copy of single_N.hsv/.sv> [password]
//!
//! 1. Every file's raw sectors are re-compressed with our `implode` and compared byte for byte
//!    with what DevilutionX wrote (proves the compressor matches, not just round-trips).
//! 2. The "hero" record is decoded with the codec and unpacked into a player.
//!
//! Pass a copy of the save, never the original.

use diablo1_rs::encrypt::{decrypt, hash};
use std::io::{Read, Seek, SeekFrom};

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let password = args.get(2).map(|s| s.as_str()).unwrap_or("xrgyrkj1");
    let mut f = std::fs::File::open(path).unwrap();
    let mut all = Vec::new();
    f.read_to_end(&mut all).unwrap();
    let hash_pos = u32_at(&all, 16) as usize;
    let block_pos = u32_at(&all, 20) as usize;
    let hash_count = u32_at(&all, 24) as usize;
    let block_count = u32_at(&all, 28) as usize;
    let mut ht = all[hash_pos..hash_pos + hash_count * 16].to_vec();
    decrypt(&mut ht, hash(b"(hash table)", 3));
    let mut bt = all[block_pos..block_pos + block_count * 16].to_vec();
    decrypt(&mut bt, hash(b"(block table)", 3));

    let mut names: Vec<String> = vec!["hero".into(), "game".into(), "hotkeys".into(), "heroitems".into(), "additionalMissiles".into()];
    for p in ["perm", "temp"] {
        for s in ['l', 's'] {
            for i in 0..25 {
                names.push(format!("{p}{s}{i:02}"));
            }
        }
    }
    let (mut sectors_ok, mut sectors_bad, mut sectors_raw) = (0, 0, 0);
    for name in &names {
        let n = name.as_bytes();
        let (h0, h1, h2) = (hash(n, 0), hash(n, 1), hash(n, 2));
        let mut idx = (h0 as usize) & (hash_count - 1);
        let mut found = None;
        for _ in 0..hash_count {
            let e = &ht[idx * 16..idx * 16 + 16];
            let block = u32_at(e, 12);
            if block == 0xFFFF_FFFF {
                break;
            }
            if u32_at(e, 0) == h1 && u32_at(e, 4) == h2 && block != 0xFFFF_FFFE {
                found = Some(block as usize);
                break;
            }
            idx = (idx + 1) & (hash_count - 1);
        }
        let Some(bi) = found else { continue };
        let b = &bt[bi * 16..bi * 16 + 16];
        let (offset, packed, unpacked, flags) = (u32_at(b, 0) as usize, u32_at(b, 4) as usize, u32_at(b, 8) as usize, u32_at(b, 12));
        let num_sectors = unpacked.div_ceil(4096);
        let table: Vec<usize> = (0..=num_sectors).map(|i| u32_at(&all, offset + i * 4) as usize).collect();
        let mut file = Vec::new();
        let mut file_ok = true;
        for s in 0..num_sectors {
            let raw = &all[offset + table[s]..offset + table[s + 1]];
            let want = (unpacked - s * 4096).min(4096);
            let plain = if raw.len() < want {
                let mut out = Vec::new();
                diablo1_rs::pkware::explode(raw, &mut out).unwrap();
                out
            } else {
                sectors_raw += 1;
                raw.to_vec()
            };
            assert_eq!(plain.len(), want, "{name} sector {s}");
            let mut buf = plain.clone();
            let len = diablo1_rs::encrypt::pkware_compress(&mut buf, want);
            if buf[..len] == *raw {
                sectors_ok += 1;
            } else {
                sectors_bad += 1;
                file_ok = false;
                println!("MISMATCH {name} sector {s}: ours {len} bytes, theirs {}", raw.len());
            }
            file.extend_from_slice(&plain);
        }
        println!("{name}: block {bi} offset {offset} packed {packed} unpacked {unpacked} flags {flags:#x} sectors {num_sectors} {}", if file_ok { "ok" } else { "DIFF" });
        if name == "hero" {
            let mut data = file.clone();
            let n = diablo1_rs::codec::codec_decode(&mut data, password);
            println!("  hero decoded length {n} (PlayerPack is {})", diablo1_rs::pack::PLAYER_PACK_SIZE);
            if n == diablo1_rs::pack::PLAYER_PACK_SIZE {
                let pk = diablo1_rs::pack::PlayerPack::from_bytes(&data[..n]);
                let end = pk.pName.iter().position(|&c| c == 0).unwrap_or(32);
                println!(
                    "  name {:?} class {} level {} exp {} gold {} str {} hellfire {}",
                    String::from_utf8_lossy(&pk.pName[..end]),
                    pk.pClass,
                    pk.pLevel,
                    pk.pExperience,
                    pk.pGold,
                    pk.pBaseStr,
                    pk.bIsHellfire
                );
                assert_eq!(pk.to_bytes(), data[..n].to_vec(), "PlayerPack round trip");
                println!("  PlayerPack bytes round-trip ok");
            }
        }
    }
    let _ = f.seek(SeekFrom::Start(0));
    println!("sectors: {sectors_ok} identical, {sectors_bad} different, {sectors_raw} stored uncompressed");
}
