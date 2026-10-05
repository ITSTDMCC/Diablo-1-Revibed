//! `Source/levels/town.cpp`: building the town map.

use crate::ctx::Ctx;
use crate::engine::geometry::Point;
use crate::enums::*;
use crate::levels::gendung::{DungeonType, MAXDUNX, MAXDUNY};
use crate::levels::trigs::is_warp_open;

fn load_dun(ctx: &mut Ctx, path: &str) -> Vec<u16> {
    let bytes = crate::engine::load_file::load_file_in_mem(ctx, path).unwrap_or_else(|| panic!("missing {path}"));
    bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

/// Original: `FillSector` (levels/town.cpp): loads level data into `dPiece`.
// @port levels/town.cpp|devilution::FillSector(const char *path, int xi, int yy) sha=9c91a13dd7e2
fn fill_sector(ctx: &mut Ctx, path: &str, xi: i32, mut yy: i32) {
    let dun_data = load_dun(ctx, path);
    let width = dun_data[0] as i32;
    let height = dun_data[1] as i32;
    let tile_layer = &dun_data[2..];
    for j in 0..height {
        let mut xx = xi;
        for i in 0..width {
            let (mut v1, mut v2, mut v3, mut v4) = (218u16, 218u16, 218u16, 218u16);
            let tile_id = tile_layer[(j * width + i) as usize] as i32 - 1;
            if tile_id >= 0 {
                let mega = ctx.gendung.pMegaTiles.as_ref().expect("pMegaTiles")[tile_id as usize];
                v1 = mega.micro1;
                v2 = mega.micro2;
                v3 = mega.micro3;
                v4 = mega.micro4;
            }
            let d = &mut ctx.gendung.dPiece;
            d[xx as usize][yy as usize] = v1;
            d[(xx + 1) as usize][yy as usize] = v2;
            d[xx as usize][(yy + 1) as usize] = v3;
            d[(xx + 1) as usize][(yy + 1) as usize] = v4;
            xx += 2;
        }
        yy += 2;
    }
}

/// Original: `FillTile` (levels/town.cpp): loads a tile into `dPiece`.
// @port levels/town.cpp|devilution::FillTile(int xx, int yy, int t) sha=7c51478b8353
fn fill_tile(ctx: &mut Ctx, xx: i32, yy: i32, t: i32) {
    let mega = ctx.gendung.pMegaTiles.as_ref().expect("pMegaTiles")[(t - 1) as usize];
    let d = &mut ctx.gendung.dPiece;
    d[xx as usize][yy as usize] = mega.micro1;
    d[(xx + 1) as usize][yy as usize] = mega.micro2;
    d[xx as usize][(yy + 1) as usize] = mega.micro3;
    d[(xx + 1) as usize][(yy + 1) as usize] = mega.micro4;
}

fn set_pieces(ctx: &mut Ctx, pieces: &[(usize, usize, u16)]) {
    for &(x, y, v) in pieces {
        ctx.gendung.dPiece[x][y] = v;
    }
}

/// Original: `TownCloseHive` (levels/town.cpp): updates the map to show the closed hive.
// @port levels/town.cpp|devilution::TownCloseHive() sha=c7dcee098cc7
fn town_close_hive(ctx: &mut Ctx) {
    ctx.gendung.dungeon[35][27] = 18;
    ctx.gendung.dungeon[36][27] = 63;
    #[rustfmt::skip]
    set_pieces(ctx, &[
        (78, 60, 0x489), (79, 60, 0x4ea), (78, 61, 0x4eb), (79, 61, 0x4ec), (78, 62, 0x4ed), (79, 62, 0x4ee),
        (78, 63, 0x4ef), (79, 63, 0x4f0), (78, 64, 0x4f1), (79, 64, 0x4f2), (78, 65, 0x4f3), (80, 60, 0x4f4),
        (81, 60, 0x4f5), (80, 61, 0x4f6), (81, 61, 0x4f7), (82, 60, 0x4f8), (83, 60, 0x4f9), (82, 61, 0x4fa),
        (83, 61, 0x4fb), (80, 62, 0x4fc), (81, 62, 0x4fd), (80, 63, 0x4fe), (81, 63, 0x4ff), (80, 64, 0x500),
        (81, 64, 0x501), (80, 65, 0x502), (81, 65, 0x503), (82, 64, 0x508), (83, 64, 0x509), (82, 65, 0x50a),
        (83, 65, 0x50b), (82, 62, 0x504), (83, 62, 0x505), (82, 63, 0x506), (83, 63, 0x507), (84, 61, 0x117),
        (84, 62, 0x117), (84, 63, 0x117), (85, 60, 0x117), (85, 61, 0x117), (85, 63, 7), (85, 64, 7),
        (86, 60, 0xd8), (86, 61, 0x17), (85, 62, 0x12), (84, 64, 0x117),
    ]);
}

/// Original: `TownCloseGrave` (levels/town.cpp): updates the map to show the closed grave.
// @port levels/town.cpp|devilution::TownCloseGrave() sha=837561bca181
fn town_close_grave(ctx: &mut Ctx) {
    #[rustfmt::skip]
    set_pieces(ctx, &[
        (36, 21, 0x52a), (37, 21, 0x52b), (36, 22, 0x52c), (37, 22, 0x52d), (36, 23, 0x52e),
        (37, 23, 0x52f), (36, 24, 0x530), (37, 24, 0x531), (35, 21, 0x53a), (34, 21, 0x53b),
    ]);
}

/// Original: `InitTownPieces` (levels/town.cpp).
// @port levels/town.cpp|devilution::InitTownPieces() sha=669829743d07
fn init_town_pieces(ctx: &mut Ctx) {
    for y in 0..MAXDUNY {
        for x in 0..MAXDUNX {
            let special = match ctx.gendung.dPiece[x][y] {
                359 => 1,
                357 => 2,
                128 => 6,
                129 => 7,
                127 => 8,
                116 => 9,
                156 => 10,
                157 => 11,
                155 => 12,
                161 => 13,
                159 => 14,
                213 => 15,
                211 => 16,
                216 => 17,
                215 => 18,
                _ => continue,
            };
            ctx.gendung.dSpecial[x][y] = special;
        }
    }
}

/// Original: `DrlgTPass3` (levels/town.cpp): initializes all of the level's data.
// @port levels/town.cpp|devilution::DrlgTPass3() sha=fb00ddc3b92f
fn drlg_t_pass3(ctx: &mut Ctx) {
    let mut yy = 0;
    while yy < MAXDUNY {
        let mut xx = 0;
        while xx < MAXDUNX {
            let d = &mut ctx.gendung.dPiece;
            d[xx][yy] = 426;
            d[xx + 1][yy] = 426;
            d[xx][yy + 1] = 426;
            d[xx + 1][yy + 1] = 426;
            xx += 2;
        }
        yy += 2;
    }
    fill_sector(ctx, "levels\\towndata\\sector1s.dun", 46, 46);
    fill_sector(ctx, "levels\\towndata\\sector2s.dun", 46, 0);
    fill_sector(ctx, "levels\\towndata\\sector3s.dun", 0, 46);
    fill_sector(ctx, "levels\\towndata\\sector4s.dun", 0, 0);

    let dun_data = load_dun(ctx, "levels\\towndata\\automap.dun");
    crate::levels::gendung::place_dun_tiles(ctx, &dun_data, Point::new(0, 0), 0);

    if !is_warp_open(ctx, DungeonType::Catacombs) {
        ctx.gendung.dungeon[20][7] = 10;
        ctx.gendung.dungeon[20][6] = 8;
        fill_tile(ctx, 48, 20, 320);
    }
    if !is_warp_open(ctx, DungeonType::Caves) {
        ctx.gendung.dungeon[4][30] = 8;
        fill_tile(ctx, 16, 68, 332);
        fill_tile(ctx, 16, 70, 331);
    }
    if !is_warp_open(ctx, DungeonType::Hell) {
        ctx.gendung.dungeon[15][35] = 7;
        ctx.gendung.dungeon[16][35] = 7;
        ctx.gendung.dungeon[17][35] = 7;
        for x in 36..46 {
            let t = ctx.rng.generate_rnd(4) + 1;
            fill_tile(ctx, x, 78, t);
        }
    }
    if ctx.init.gb_is_hellfire {
        if is_warp_open(ctx, DungeonType::Nest) {
            town_open_hive(ctx);
        } else {
            town_close_hive(ctx);
        }
        if is_warp_open(ctx, DungeonType::Crypt) {
            town_open_grave(ctx);
        } else {
            town_close_grave(ctx);
        }
    }
    let pw = ctx.quests.Quests[Q_PWATER as usize]._qactive;
    if pw != QUEST_DONE && pw != QUEST_NOTAVAIL {
        fill_tile(ctx, 60, 70, 342);
    } else {
        fill_tile(ctx, 60, 70, 71);
    }
    init_town_pieces(ctx);
}

/// Original: `devilution::OpensHive` (levels/town.cpp).
// @port levels/town.cpp|devilution::OpensHive(Point position) sha=a0d6a501e6ec
pub fn opens_hive(position: Point) -> bool {
    let (xp, yp) = (position.x, position.y);
    (79..=82).contains(&xp) && (61..=64).contains(&yp)
}

/// Original: `devilution::OpensGrave` (levels/town.cpp).
// @port levels/town.cpp|devilution::OpensGrave(Point position) sha=d4b64476f771
pub fn opens_grave(position: Point) -> bool {
    let (xp, yp) = (position.x, position.y);
    (35..=38).contains(&xp) && (20..=24).contains(&yp)
}

/// Original: `devilution::OpenHive` (levels/town.cpp).
// @port levels/town.cpp|devilution::OpenHive() sha=02400d58c343
pub fn open_hive(ctx: &mut Ctx) {
    crate::msg::net_send_cmd(ctx, false, CMD_OPENHIVE);
    ctx.quests.Quests[Q_FARMER as usize]._qactive = QUEST_DONE;
    if ctx.init.gb_is_multiplayer {
        crate::msg::net_send_cmd_quest(ctx, true, Q_FARMER as usize);
    }
}

/// Original: `devilution::OpenGrave` (levels/town.cpp).
// @port levels/town.cpp|devilution::OpenGrave() sha=cf6f1393c907
pub fn open_grave(ctx: &mut Ctx) {
    crate::msg::net_send_cmd(ctx, false, CMD_OPENGRAVE);
    ctx.quests.Quests[Q_GRAVE as usize]._qactive = QUEST_DONE;
    if ctx.init.gb_is_multiplayer {
        crate::msg::net_send_cmd_quest(ctx, true, Q_GRAVE as usize);
    }
}

/// Original: `devilution::TownOpenHive` (levels/town.cpp).
// @port levels/town.cpp|devilution::TownOpenHive() sha=cad3cca4984d
pub fn town_open_hive(ctx: &mut Ctx) {
    ctx.gendung.dungeon[36][27] = 47;
    #[rustfmt::skip]
    set_pieces(ctx, &[
        (78, 60, 0x489), (79, 60, 0x48a), (78, 61, 0x48b), (79, 61, 0x50d), (78, 62, 0x4ed), (78, 63, 0x4ef),
        (79, 62, 0x50f), (79, 63, 0x510), (79, 64, 0x511), (78, 64, 0x119), (78, 65, 0x11b), (79, 65, 0x11c),
        (80, 60, 0x512), (80, 61, 0x514), (81, 61, 0x515), (82, 60, 0x516), (83, 60, 0x517), (82, 61, 0x518),
        (83, 61, 0x519), (80, 62, 0x51a), (81, 62, 0x51b), (80, 63, 0x51c), (81, 63, 0x51d), (80, 64, 0x51e),
        (81, 64, 0x51f), (80, 65, 0x520), (81, 65, 0x521), (82, 64, 0x526), (83, 64, 0x527), (82, 65, 0x528),
        (83, 65, 0x529), (82, 62, 0x522), (83, 62, 0x523), (82, 63, 0x524), (83, 63, 0x525), (84, 61, 0x117),
        (84, 62, 0x117), (84, 63, 0x117), (85, 60, 0x117), (85, 61, 0x117), (85, 63, 7), (85, 64, 7),
        (86, 60, 0xd8), (86, 61, 0x17), (85, 62, 0x12), (84, 64, 0x117),
    ]);
}

/// Original: `devilution::TownOpenGrave` (levels/town.cpp).
// @port levels/town.cpp|devilution::TownOpenGrave() sha=f4653d4e1836
pub fn town_open_grave(ctx: &mut Ctx) {
    ctx.gendung.dungeon[14][8] = 47;
    ctx.gendung.dungeon[14][7] = 47;
    #[rustfmt::skip]
    set_pieces(ctx, &[
        (36, 21, 0x532), (37, 21, 0x533), (36, 22, 0x534), (37, 22, 0x535), (36, 23, 0x536),
        (37, 23, 0x537), (36, 24, 0x538), (37, 24, 0x539), (35, 21, 0x53a), (34, 21, 0x53b),
    ]);
}

/// Original: `devilution::CleanTownFountain` (levels/town.cpp).
// @port levels/town.cpp|devilution::CleanTownFountain() sha=fa2942b79da2
pub fn clean_town_fountain(ctx: &mut Ctx) {
    if ctx.gendung.pMegaTiles.is_none() {
        return;
    }
    fill_tile(ctx, 60, 70, 71);
}

/// Original: `devilution::CreateTown` (levels/town.cpp).
// @port levels/town.cpp|devilution::CreateTown(lvl_entry entry) sha=6311b52ade65
pub fn create_town(ctx: &mut Ctx, entry: lvl_entry) {
    ctx.gendung.dminPosition = Point::new(10, 10);
    ctx.gendung.dmaxPosition = Point::new(84, 84);
    if entry == ENTRY_MAIN {
        // New game
        ctx.gendung.ViewPosition = Point::new(75, 68);
    } else if entry == ENTRY_PREV {
        // Cathedral
        ctx.gendung.ViewPosition = Point::new(25, 31);
    } else if entry == ENTRY_TWARPUP {
        match ctx.trigs.TWarpFrom {
            5 => ctx.gendung.ViewPosition = Point::new(49, 22),
            9 => ctx.gendung.ViewPosition = Point::new(18, 69),
            13 => ctx.gendung.ViewPosition = Point::new(41, 81),
            21 => ctx.gendung.ViewPosition = Point::new(36, 25),
            17 => ctx.gendung.ViewPosition = Point::new(79, 62),
            _ => {}
        }
    }
    drlg_t_pass3(ctx);
}
