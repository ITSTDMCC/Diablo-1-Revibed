//! The port of DevilutionX's `test/drlg_test.hpp`: generate a dungeon for a seed and compare it
//! with a level DevilutionX generated (`test/fixtures/<diablo|hellfire>/<level>-<seed>.dun`).
//!
//! The fixtures are read from the DevilutionX source (`$DEVILUTIONX_SOURCE`, default
//! `../Decomp/source_1.5.3` next to the crate); tests skip when it is missing.

use std::path::PathBuf;

use diablo1_rs::ctx::Ctx;
use diablo1_rs::enums::lvl_entry;
use diablo1_rs::levels::gendung::{DungeonType, MegaTile, DMAXX, DMAXY};
use diablo1_rs::platform::Platform;

pub fn source_dir() -> PathBuf {
    std::env::var_os("DEVILUTIONX_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("Decomp").join("source_1.5.3"))
}

/// `GetTileCount`
fn get_tile_count(level_type: DungeonType) -> usize {
    match level_type {
        DungeonType::Town => 376,
        DungeonType::Cathedral => 206,
        DungeonType::Catacombs => 160,
        DungeonType::Caves => 206,
        DungeonType::Hell => 137,
        DungeonType::Nest => 166,
        DungeonType::Crypt => 217,
        DungeonType::None => panic!("Invalid level type"),
    }
}

pub struct Harness {
    pub ctx: Ctx,
    dun_data: Vec<u16>,
}

impl Harness {
    /// `LoadExpectedLevelData`; None (test skipped) without the DevilutionX source.
    pub fn new(fixture: &str) -> Option<Harness> {
        let fixtures = source_dir().join("test").join("fixtures");
        let path = fixtures.join(fixture);
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("skipped: {} not found (set DEVILUTIONX_SOURCE)", path.display());
            return None;
        };
        let dun_data: Vec<u16> = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        assert_eq!((DMAXX, DMAXY), (dun_data[0] as usize, dun_data[1] as usize));
        let mut ctx = Ctx::new(Platform::headless_from_env());
        let base = format!("{}/", fixtures.display());
        ctx.paths.set_pref_path(&base);
        ctx.paths.set_assets_path(&base);
        Some(Harness { ctx, dun_data })
    }

    /// A further `LoadExpectedLevelData` in the same test.
    pub fn load_expected(&mut self, fixture: &str) -> bool {
        let path = source_dir().join("test").join("fixtures").join(fixture);
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("skipped: {} not found", path.display());
            return false;
        };
        self.dun_data = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        true
    }

    /// `paths::SetAssetsPath(paths::BasePath() + "/assets")`: the DevilutionX source's assets
    /// folder; false (test skipped) when it is missing.
    pub fn use_source_assets(&mut self) -> bool {
        let assets = source_dir().join("assets");
        if !assets.is_dir() {
            eprintln!("skipped: {} not found", assets.display());
            return false;
        }
        self.ctx.paths.set_assets_path(&format!("{}/", assets.display()));
        true
    }

    /// `TestInitGame`
    pub fn init_game(&mut self, full_quests: bool, original_cathedral: bool) {
        let ctx = &mut self.ctx;
        ctx.players.Players.truncate(1);
        while ctx.players.Players.is_empty() {
            ctx.players.Players.push(Default::default());
        }
        ctx.players.MyPlayer = Some(0);
        ctx.players.Players[0].pOriginalCathedral = original_cathedral;

        ctx.multi.sgGameInitInfo.fullQuests = full_quests as u8;
        ctx.init.gb_is_multiplayer = !full_quests;

        diablo1_rs::quests::init_quests(ctx);
    }

    /// `TestCreateDungeon`
    pub fn create_dungeon(&mut self, level: i32, seed: u32, entry: lvl_entry) {
        let ctx = &mut self.ctx;
        ctx.gendung.currlevel = level as u8;
        ctx.gendung.leveltype = diablo1_rs::levels::gendung::get_level_type(level);

        ctx.gendung.pMegaTiles = Some(vec![MegaTile::default(); get_tile_count(ctx.gendung.leveltype)]);

        diablo1_rs::levels::gendung::create_dungeon(ctx, seed, entry);
        diablo1_rs::levels::themes::create_theme_rooms(ctx);

        let tile_layer = &self.dun_data[2..];
        let mut mismatches = Vec::new();
        for y in 0..DMAXY {
            for x in 0..DMAXX {
                let tile_id = tile_layer[y * DMAXX + x] as u8;
                if ctx.gendung.dungeon[x][y] != tile_id {
                    mismatches.push(format!("({x},{y}) expected {tile_id} got {}", ctx.gendung.dungeon[x][y]));
                }
            }
        }
        assert!(mismatches.is_empty(), "Tiles don't match at {} positions (level {level}, seed {seed}, entry {entry}): {:?}", mismatches.len(), &mismatches[..mismatches.len().min(10)]);

        let transparent_layer = &self.dun_data[2 + DMAXX * DMAXY * 13..];
        let mut i = 0;
        for y in 16..16 + DMAXY * 2 {
            for x in 16..16 + DMAXX * 2 {
                let sector_id = transparent_layer[i] as u8;
                i += 1;
                if ctx.gendung.dTransVal[x][y] as u8 != sector_id {
                    mismatches.push(format!("({x},{y}) expected {sector_id} got {}", ctx.gendung.dTransVal[x][y]));
                }
            }
        }
        assert!(mismatches.is_empty(), "Room/region indexes don't match at {} positions (level {level}, seed {seed}, entry {entry}): {:?}", mismatches.len(), &mismatches[..mismatches.len().min(10)]);
    }
}
