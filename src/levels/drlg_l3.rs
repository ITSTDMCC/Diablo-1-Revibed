//! `Source/levels/drlg_l3.cpp`: caves (and Hellfire nest) level generation.
//!
//! Unchecked one-past-the-edge `dungeon` accesses of the original go through
//! `gendung::dungeon_flat`/`set_dungeon_flat` (see there).

use crate::ctx::Ctx;
use crate::engine::geometry::{Displacement, Point, Rectangle, Size};
use crate::enums::*;
use crate::levels::drlg_l3_data::*;
use crate::levels::gendung::*;

/// File-scope globals of levels/drlg_l3.cpp.
#[derive(Default)]
pub struct DrlgL3State {
    lockoutcnt: i32,
}

const DX: i32 = DMAXX as i32;
const DY: i32 = DMAXY as i32;

fn rnd(ctx: &mut Ctx, v: i32) -> i32 {
    ctx.rng.generate_rnd(v)
}

fn flip(ctx: &mut Ctx) -> bool {
    ctx.rng.flip_coin(2)
}

fn d(ctx: &Ctx, x: i32, y: i32) -> u8 {
    dungeon_flat(ctx, x, y)
}

fn sd(ctx: &mut Ctx, x: i32, y: i32, v: u8) {
    set_dungeon_flat(ctx, x, y, v);
}

/// Original: `InitDungeonFlags` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::InitDungeonFlags() sha=cd2587d7b202
fn init_dungeon_flags(ctx: &mut Ctx) {
    ctx.gendung.dungeon = [[0; DMAXY]; DMAXX];
    ctx.gendung.Protected.reset();
}

/// Original: `FillRoom` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::FillRoom(int x1, int y1, int x2, int y2) sha=c7bf09620000
fn fill_room(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    if x1 <= 1 || x2 >= 34 || y1 <= 1 || y2 >= 38 {
        return false;
    }
    let mut v = 0;
    for j in y1..=y2 {
        for i in x1..=x2 {
            v += d(ctx, i, j) as i32;
        }
    }
    if v != 0 {
        return false;
    }
    for j in y1 + 1..y2 {
        for i in x1 + 1..x2 {
            sd(ctx, i, j, 1);
        }
    }
    for j in y1..=y2 {
        if !flip(ctx) {
            sd(ctx, x1, j, 1);
        }
        if !flip(ctx) {
            sd(ctx, x2, j, 1);
        }
    }
    for i in x1..=x2 {
        if !flip(ctx) {
            sd(ctx, i, y1, 1);
        }
        if !flip(ctx) {
            sd(ctx, i, y2, 1);
        }
    }
    true
}

/// Original: `CreateBlock` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::CreateBlock(int x, int y, int obs, int dir) sha=e71d84de7bc5
fn create_block(ctx: &mut Ctx, x: i32, y: i32, obs: i32, dir: i32) {
    let (mut x1, mut y1, mut x2, mut y2) = (0, 0, 0, 0);
    let blksizex = rnd(ctx, 2) + 3;
    let blksizey = rnd(ctx, 2) + 3;
    if dir == 0 {
        y2 = y - 1;
        y1 = y2 - blksizey;
        if blksizex < obs {
            x1 = rnd(ctx, blksizex) + x;
        }
        if blksizex == obs {
            x1 = x;
        }
        if blksizex > obs {
            x1 = x - rnd(ctx, blksizex);
        }
        x2 = blksizex + x1;
    }
    if dir == 1 {
        x1 = x + 1;
        x2 = x1 + blksizex;
        if blksizey < obs {
            y1 = rnd(ctx, blksizey) + y;
        }
        if blksizey == obs {
            y1 = y;
        }
        if blksizey > obs {
            y1 = y - rnd(ctx, blksizey);
        }
        y2 = y1 + blksizey;
    }
    if dir == 2 {
        y1 = y + 1;
        y2 = y1 + blksizey;
        if blksizex < obs {
            x1 = rnd(ctx, blksizex) + x;
        }
        if blksizex == obs {
            x1 = x;
        }
        if blksizex > obs {
            x1 = x - rnd(ctx, blksizex);
        }
        x2 = blksizex + x1;
    }
    if dir == 3 {
        x2 = x - 1;
        x1 = x2 - blksizex;
        if blksizey < obs {
            y1 = rnd(ctx, blksizey) + y;
        }
        if blksizey == obs {
            y1 = y;
        }
        if blksizey > obs {
            y1 = y - rnd(ctx, blksizey);
        }
        y2 = y1 + blksizey;
    }
    if fill_room(ctx, x1, y1, x2, y2) {
        if ctx.rng.flip_coin(4) {
            return;
        }
        if dir != 2 {
            create_block(ctx, x1, y1, blksizey, 0);
        }
        if dir != 3 {
            create_block(ctx, x2, y1, blksizex, 1);
        }
        if dir != 0 {
            create_block(ctx, x1, y2, blksizey, 2);
        }
        if dir != 1 {
            create_block(ctx, x1, y1, blksizex, 3);
        }
    }
}

/// Original: `FloorArea` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::FloorArea(int x1, int y1, int x2, int y2) sha=e8eec3090f36
fn floor_area(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    for j in y1..=y2 {
        for i in x1..=x2 {
            sd(ctx, i, j, 1);
        }
    }
}

/// Original: `FillDiagonals` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::FillDiagonals() sha=9971233298b2
fn fill_diagonals(ctx: &mut Ctx) {
    for j in 0..DY - 1 {
        for i in 0..DX - 1 {
            let v = d(ctx, i + 1, j + 1) as i32 + 2 * d(ctx, i, j + 1) as i32 + 4 * d(ctx, i + 1, j) as i32 + 8 * d(ctx, i, j) as i32;
            if v == 6 {
                if flip(ctx) {
                    sd(ctx, i, j, 1);
                } else {
                    sd(ctx, i + 1, j + 1, 1);
                }
            }
            if v == 9 {
                if flip(ctx) {
                    sd(ctx, i + 1, j, 1);
                } else {
                    sd(ctx, i, j + 1, 1);
                }
            }
        }
    }
}

/// Original: `FillSingles` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::FillSingles() sha=b16897252150
fn fill_singles(ctx: &mut Ctx) {
    for j in 1..DY - 1 {
        for i in 1..DX - 1 {
            let s = |dx: i32, dy: i32| d(ctx, i + dx, j + dy) as i32;
            if s(0, 0) == 0 && s(0, -1) + s(-1, -1) + s(1, -1) == 3 && s(1, 0) + s(-1, 0) == 2 && s(0, 1) + s(-1, 1) + s(1, 1) == 3 {
                sd(ctx, i, j, 1);
            }
        }
    }
}

/// Original: `FillStraights` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::FillStraights() sha=642a1dd83d48
fn fill_straights(ctx: &mut Ctx) {
    // uninitialised in the original until the first run is found
    let mut xc = 0;
    let mut yc = 0;
    for j in 0..DY - 1 {
        let mut xs = 0;
        for i in 0..37 {
            if d(ctx, i, j) == 0 && d(ctx, i, j + 1) == 1 {
                if xs == 0 {
                    xc = i;
                }
                xs += 1;
            } else {
                if xs > 3 && !flip(ctx) {
                    for k in xc..i {
                        let rv = rnd(ctx, 2);
                        sd(ctx, k, j, rv as u8);
                    }
                }
                xs = 0;
            }
        }
    }
    for j in 0..DY - 1 {
        let mut xs = 0;
        for i in 0..37 {
            if d(ctx, i, j) == 1 && d(ctx, i, j + 1) == 0 {
                if xs == 0 {
                    xc = i;
                }
                xs += 1;
            } else {
                if xs > 3 && !flip(ctx) {
                    for k in xc..i {
                        let rv = rnd(ctx, 2);
                        sd(ctx, k, j + 1, rv as u8);
                    }
                }
                xs = 0;
            }
        }
    }
    for i in 0..DX - 1 {
        let mut ys = 0;
        for j in 0..37 {
            if d(ctx, i, j) == 0 && d(ctx, i + 1, j) == 1 {
                if ys == 0 {
                    yc = j;
                }
                ys += 1;
            } else {
                if ys > 3 && !flip(ctx) {
                    for k in yc..j {
                        let rv = rnd(ctx, 2);
                        sd(ctx, i, k, rv as u8);
                    }
                }
                ys = 0;
            }
        }
    }
    for i in 0..DX - 1 {
        let mut ys = 0;
        for j in 0..37 {
            if d(ctx, i, j) == 1 && d(ctx, i + 1, j) == 0 {
                if ys == 0 {
                    yc = j;
                }
                ys += 1;
            } else {
                if ys > 3 && !flip(ctx) {
                    for k in yc..j {
                        let rv = rnd(ctx, 2);
                        sd(ctx, i + 1, k, rv as u8);
                    }
                }
                ys = 0;
            }
        }
    }
}

/// Original: `Edges` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::Edges() sha=fa9a7138ca35
fn edges(ctx: &mut Ctx) {
    for j in 0..DMAXY {
        ctx.gendung.dungeon[DMAXX - 1][j] = 0;
    }
    for i in 0..DMAXX {
        ctx.gendung.dungeon[i][DMAXY - 1] = 0;
    }
}

/// Original: `GetFloorArea` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::GetFloorArea() sha=344688a2a791
fn get_floor_area(ctx: &Ctx) -> i32 {
    ctx.gendung.dungeon.iter().flatten().map(|&v| v as i32).sum()
}

/// Original: `MakeMegas` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::MakeMegas() sha=00cfd11b85cf
fn make_megas(ctx: &mut Ctx) {
    for j in 0..DY - 1 {
        for i in 0..DX - 1 {
            let mut v = d(ctx, i + 1, j + 1) as i32 + 2 * d(ctx, i, j + 1) as i32 + 4 * d(ctx, i + 1, j) as i32 + 8 * d(ctx, i, j) as i32;
            if v == 6 {
                v = ctx.rng.pick_randomly_among(&[12, 5]);
            }
            if v == 9 {
                v = ctx.rng.pick_randomly_among(&[13, 14]);
            }
            sd(ctx, i, j, L3ConvTbl[v as usize]);
        }
        sd(ctx, DX - 1, j, 8);
    }
    for i in 0..DX {
        sd(ctx, i, DY - 1, 8);
    }
}

/// `river[3][100]` with the original's flat indexing (index 100 of a row is the next row's 0).
struct RiverTable([i32; 300]);

impl RiverTable {
    fn get(&self, r: usize, c: i32) -> i32 {
        let i = r as i32 * 100 + c;
        if (0..300).contains(&i) {
            self.0[i as usize]
        } else {
            0
        }
    }
    fn set(&mut self, r: usize, c: i32, v: i32) {
        let i = r as i32 * 100 + c;
        if (0..300).contains(&i) {
            self.0[i as usize] = v;
        }
    }
}

/// Original: `River` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::River() sha=6f76c6af19fc
fn river(ctx: &mut Ctx) {
    let mut dir = 0;
    let mut nodir = 0;
    let mut river = RiverTable([0; 300]);
    let mut riveramt = 0;
    let mut rivercnt = 0;
    let mut trys = 0;
    // BUGFIX: pdir is uninitialized, add code `pdir = -1;`(fixed)
    let mut pdir = -1;
    while trys < 200 && rivercnt < 4 {
        let mut bail = false;
        while !bail && trys < 200 {
            trys += 1;
            let mut rx = 0;
            let mut ry = 0;
            let mut i = 0;
            // BUGFIX: Replace with `(ry >= DMAXY || dungeon[rx][ry] < 25 || dungeon[rx][ry] > 28) && i < 100` (fixed)
            while (ry >= DY || d(ctx, rx, ry) < 25 || d(ctx, rx, ry) > 28) && i < 100 {
                rx = rnd(ctx, DX);
                ry = rnd(ctx, DY);
                i += 1;
                // BUGFIX: Move `ry < DMAXY` check before dungeon checks (fixed)
                while ry < DY && (d(ctx, rx, ry) < 25 || d(ctx, rx, ry) > 28) {
                    rx += 1;
                    if rx >= DX {
                        rx = 0;
                        ry += 1;
                    }
                }
            }
            // BUGFIX: Continue if `ry >= DMAXY` (fixed)
            if ry >= DY {
                continue;
            }
            if i >= 100 {
                return;
            }
            match d(ctx, rx, ry) {
                25 => {
                    dir = 3;
                    nodir = 2;
                    river.set(2, 0, 40);
                }
                26 => {
                    dir = 0;
                    nodir = 1;
                    river.set(2, 0, 38);
                }
                27 => {
                    dir = 1;
                    nodir = 0;
                    river.set(2, 0, 41);
                }
                28 => {
                    dir = 2;
                    nodir = 3;
                    river.set(2, 0, 39);
                }
                _ => {}
            }
            river.set(0, 0, rx);
            river.set(1, 0, ry);
            riveramt = 1;
            let mut nodir2 = 4;
            let mut dircheck = 0;
            while dircheck < 4 && riveramt < 100 {
                let px = rx;
                let py = ry;
                if dircheck == 0 {
                    dir = rnd(ctx, 4);
                } else {
                    dir = (dir + 1) & 3;
                }
                dircheck += 1;
                while dir == nodir || dir == nodir2 {
                    dir = (dir + 1) & 3;
                    dircheck += 1;
                }
                if dir == 0 && ry > 0 {
                    ry -= 1;
                }
                if dir == 1 && ry < DY {
                    ry += 1;
                }
                if dir == 2 && rx < DX {
                    rx += 1;
                }
                if dir == 3 && rx > 0 {
                    rx -= 1;
                }
                if d(ctx, rx, ry) == 7 {
                    dircheck = 0;
                    if dir < 2 {
                        let v = rnd(ctx, 2) + 17;
                        river.set(2, riveramt, v);
                    }
                    if dir > 1 {
                        let v = rnd(ctx, 2) + 15;
                        river.set(2, riveramt, v);
                    }
                    river.set(0, riveramt, rx);
                    river.set(1, riveramt, ry);
                    riveramt += 1;
                    if (dir == 0 && pdir == 2) || (dir == 3 && pdir == 1) {
                        if riveramt > 2 {
                            river.set(2, riveramt - 2, 22);
                        }
                        nodir2 = if dir == 0 { 1 } else { 2 };
                    }
                    if (dir == 0 && pdir == 3) || (dir == 2 && pdir == 1) {
                        if riveramt > 2 {
                            river.set(2, riveramt - 2, 21);
                        }
                        nodir2 = if dir == 0 { 1 } else { 3 };
                    }
                    if (dir == 1 && pdir == 2) || (dir == 3 && pdir == 0) {
                        if riveramt > 2 {
                            river.set(2, riveramt - 2, 20);
                        }
                        nodir2 = if dir == 1 { 0 } else { 2 };
                    }
                    if (dir == 1 && pdir == 3) || (dir == 2 && pdir == 0) {
                        if riveramt > 2 {
                            river.set(2, riveramt - 2, 19);
                        }
                        nodir2 = if dir == 1 { 0 } else { 3 };
                    }
                    pdir = dir;
                } else {
                    rx = px;
                    ry = py;
                }
            }
            // BUGFIX: Check `ry >= 2` (fixed)
            if dir == 0 && ry >= 2 && d(ctx, rx, ry - 1) == 10 && d(ctx, rx, ry - 2) == 8 {
                river.set(0, riveramt, rx);
                river.set(1, riveramt, ry - 1);
                river.set(2, riveramt, 24);
                if pdir == 2 {
                    river.set(2, riveramt - 1, 22);
                }
                if pdir == 3 {
                    river.set(2, riveramt - 1, 21);
                }
                bail = true;
            }
            // BUGFIX: Check `ry + 2 < DMAXY` (fixed)
            if dir == 1 && ry + 2 < DY && d(ctx, rx, ry + 1) == 2 && d(ctx, rx, ry + 2) == 8 {
                river.set(0, riveramt, rx);
                river.set(1, riveramt, ry + 1);
                river.set(2, riveramt, 42);
                if pdir == 2 {
                    river.set(2, riveramt - 1, 20);
                }
                if pdir == 3 {
                    river.set(2, riveramt - 1, 19);
                }
                bail = true;
            }
            // BUGFIX: Check `rx + 2 < DMAXX` (fixed)
            if dir == 2 && rx + 2 < DX && d(ctx, rx + 1, ry) == 4 && d(ctx, rx + 2, ry) == 8 {
                river.set(0, riveramt, rx + 1);
                river.set(1, riveramt, ry);
                river.set(2, riveramt, 43);
                if pdir == 0 {
                    river.set(2, riveramt - 1, 19);
                }
                if pdir == 1 {
                    river.set(2, riveramt - 1, 21);
                }
                bail = true;
            }
            // BUGFIX: Check `rx >= 2` (fixed)
            if dir == 3 && rx >= 2 && d(ctx, rx - 1, ry) == 9 && d(ctx, rx - 2, ry) == 8 {
                river.set(0, riveramt, rx - 1);
                river.set(1, riveramt, ry);
                river.set(2, riveramt, 23);
                if pdir == 0 {
                    river.set(2, riveramt - 1, 20);
                }
                if pdir == 1 {
                    river.set(2, riveramt - 1, 22);
                }
                bail = true;
            }
        }
        if bail && riveramt < 7 {
            bail = false;
        }
        if bail {
            let mut found = 0;
            let mut lpcnt = 0;
            let mut bridge = 0;
            while found == 0 && lpcnt < 30 {
                lpcnt += 1;
                bridge = rnd(ctx, riveramt);
                let (bx, by, bt) = (river.get(0, bridge), river.get(1, bridge), river.get(2, bridge));
                if (bt == 15 || bt == 16) && d(ctx, bx, by - 1) == 7 && d(ctx, bx, by + 1) == 7 {
                    found = 1;
                }
                if (bt == 17 || bt == 18) && d(ctx, bx - 1, by) == 7 && d(ctx, bx + 1, by) == 7 {
                    found = 2;
                }
                let mut i = 0;
                while i < riveramt && found != 0 {
                    if found == 1 && (by - 1 == river.get(1, i) || by + 1 == river.get(1, i)) && bx == river.get(0, i) {
                        found = 0;
                    }
                    if found == 2 && (bx - 1 == river.get(0, i) || bx + 1 == river.get(0, i)) && by == river.get(1, i) {
                        found = 0;
                    }
                    i += 1;
                }
            }
            if found != 0 {
                river.set(2, bridge, if found == 1 { 44 } else { 45 });
                rivercnt += 1;
                for b in 0..=riveramt {
                    let (x, y, v) = (river.get(0, b), river.get(1, b), river.get(2, b));
                    sd(ctx, x, y, v as u8);
                }
            }
        }
    }
}

/// Original: `SpawnEdge` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::SpawnEdge(int x, int y, int *totarea) sha=64d73cb9d6f8
fn spawn_edge(ctx: &mut Ctx, x: i32, y: i32, totarea: &mut i32) -> bool {
    const SPAWNTABLE: [u8; 15] = [0x00, 0x0A, 0x43, 0x05, 0x2c, 0x06, 0x09, 0x00, 0x00, 0x1c, 0x83, 0x06, 0x09, 0x0A, 0x05];
    if *totarea > 40 {
        return true;
    }
    if x < 0 || y < 0 || x >= DX || y >= DY {
        return true;
    }
    let v = d(ctx, x, y);
    if (v & 0x80) != 0 {
        return false;
    }
    if v > 15 {
        return true;
    }
    let i = v as usize;
    sd(ctx, x, y, v | 0x80);
    *totarea += 1;
    let t = SPAWNTABLE[i];
    if (t & 8) != 0 && spawn_edge(ctx, x, y - 1, totarea) {
        return true;
    }
    if (t & 4) != 0 && spawn_edge(ctx, x, y + 1, totarea) {
        return true;
    }
    if (t & 2) != 0 && spawn_edge(ctx, x + 1, y, totarea) {
        return true;
    }
    if (t & 1) != 0 && spawn_edge(ctx, x - 1, y, totarea) {
        return true;
    }
    if (t & 0x80) != 0 && spawn(ctx, x, y - 1, totarea) {
        return true;
    }
    if (t & 0x40) != 0 && spawn(ctx, x, y + 1, totarea) {
        return true;
    }
    if (t & 0x20) != 0 && spawn(ctx, x + 1, y, totarea) {
        return true;
    }
    if (t & 0x10) != 0 && spawn(ctx, x - 1, y, totarea) {
        return true;
    }
    false
}

/// Original: `Spawn` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::Spawn(int x, int y, int *totarea) sha=efc2f9eeea02
fn spawn(ctx: &mut Ctx, x: i32, y: i32, totarea: &mut i32) -> bool {
    const SPAWNTABLE: [u8; 15] = [0x00, 0x0A, 0x03, 0x05, 0x0C, 0x06, 0x09, 0x00, 0x00, 0x0C, 0x03, 0x06, 0x09, 0x0A, 0x05];
    if *totarea > 40 {
        return true;
    }
    if x < 0 || y < 0 || x >= DX || y >= DY {
        return true;
    }
    let v = d(ctx, x, y);
    if (v & 0x80) != 0 {
        return false;
    }
    if v > 15 {
        return true;
    }
    let i = v;
    sd(ctx, x, y, v | 0x80);
    *totarea += 1;
    if i != 8 {
        let t = SPAWNTABLE[i as usize];
        if (t & 8) != 0 && spawn_edge(ctx, x, y - 1, totarea) {
            return true;
        }
        if (t & 4) != 0 && spawn_edge(ctx, x, y + 1, totarea) {
            return true;
        }
        if (t & 2) != 0 && spawn_edge(ctx, x + 1, y, totarea) {
            return true;
        }
        if (t & 1) != 0 && spawn_edge(ctx, x - 1, y, totarea) {
            return true;
        }
    } else {
        if spawn(ctx, x + 1, y, totarea) {
            return true;
        }
        if spawn(ctx, x - 1, y, totarea) {
            return true;
        }
        if spawn(ctx, x, y + 1, totarea) {
            return true;
        }
        if spawn(ctx, x, y - 1, totarea) {
            return true;
        }
    }
    false
}

/// Original: `CanReplaceTile` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::CanReplaceTile(uint8_t replace, Point tile) sha=7cfb72a76160
fn can_replace_tile(ctx: &Ctx, replace: u8, tile: Point) -> bool {
    use crate::engine::geometry::Direction;
    if !(84..=100).contains(&replace) {
        return true;
    }
    // BUGFIX: p2 is a workaround for a bug, only p1 should have been used (fixing this breaks compatability)
    let in_b = |p: Point| p.x >= 0 && p.x < DX && p.y >= 0 && p.y < DY;
    let cmp = |p1: Point, p2: Point| in_b(p1) && in_b(p2) && (d(ctx, p1.x, p1.y) >= 84 && d(ctx, p2.x, p2.y) <= 100);
    let nw = tile + Direction::NorthWest;
    !(cmp(tile + Direction::NorthWest, nw) || cmp(tile + Direction::SouthEast, nw) || cmp(tile + Direction::SouthWest, nw) || cmp(tile + Direction::NorthEast, nw))
}

/// Original: `PlaceMiniSetRandom` (levels/drlg_l3.cpp): randomly places the miniset wherever it
/// fits; returns whether at least one was placed.
// @port levels/drlg_l3.cpp|devilution::PlaceMiniSetRandom(const Miniset &miniset, int rndper) sha=522345a03c3c
fn place_mini_set_random(ctx: &mut Ctx, miniset: &Miniset, rndper: i32) -> bool {
    let sw = miniset.size.width;
    let sh = miniset.size.height;
    let mut placed = false;
    for sy in 0..DY - sh {
        for sx in 0..DX - sw {
            if !miniset.matches(&ctx.gendung, Point::new(sx, sy), true) {
                continue;
            }
            // BUGFIX: This should not be applied to Nest levels
            if !can_replace_tile(ctx, miniset.replace[0][0], Point::new(sx, sy)) {
                continue;
            }
            if rnd(ctx, 100) >= rndper {
                continue;
            }
            miniset.place(&mut ctx.gendung, Point::new(sx, sy), false);
            placed = true;
        }
    }
    placed
}

/// Original: `PlaceMiniSetRandom1x1` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::PlaceMiniSetRandom1x1(uint8_t search, uint8_t replace, int rndper) sha=6a343fa7f345
fn place_mini_set_random_1x1(ctx: &mut Ctx, search: u8, replace: u8, rndper: i32) {
    let mut m = Miniset { size: Size::new(1, 1), search: [[0; 6]; 6], replace: [[0; 6]; 6] };
    m.search[0][0] = search;
    m.replace[0][0] = replace;
    place_mini_set_random(ctx, &m, rndper);
}

/// Original: `PlaceSlimePool` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::PlaceSlimePool() sha=3522dc7af7f6
fn place_slime_pool(ctx: &mut Ctx) -> bool {
    let mut lavapool = 0;
    if place_mini_set_random(ctx, &HivePattern41, 30) {
        lavapool += 1;
    }
    if place_mini_set_random(ctx, &HivePattern42, 40) {
        lavapool += 1;
    }
    if place_mini_set_random(ctx, &HivePattern39, 50) {
        lavapool += 1;
    }
    if place_mini_set_random(ctx, &HivePattern40, 60) {
        lavapool += 1;
    }
    lavapool >= 3
}

/// Original: `PlaceLavaPool` (levels/drlg_l3.cpp): flood fills dirt and wall tiles looking for
/// an area of at most 40 tiles disconnected from the map edge and turns it into lava.
// @port levels/drlg_l3.cpp|devilution::PlaceLavaPool() sha=5dbcb26b3db4
fn place_lava_pool(ctx: &mut Ctx) -> bool {
    const POOLSUB: [u8; 15] = [0, 35, 26, 36, 25, 29, 34, 7, 33, 28, 27, 37, 32, 31, 30];
    let mut lave_pool_placed = false;
    for duny in 0..DY {
        for dunx in 0..DY {
            if d(ctx, dunx, duny) != 8 {
                continue;
            }
            let v = d(ctx, dunx, duny);
            sd(ctx, dunx, duny, v | 0x80);
            let mut totarea = 1;
            let mut found = true;
            if dunx + 1 < DX {
                found = spawn(ctx, dunx + 1, duny, &mut totarea);
            }
            if dunx - 1 > 0 && !found {
                found = spawn(ctx, dunx - 1, duny, &mut totarea);
            } else {
                found = true;
            }
            if duny + 1 < DY && !found {
                found = spawn(ctx, dunx, duny + 1, &mut totarea);
            } else {
                found = true;
            }
            if duny - 1 > 0 && !found {
                found = spawn(ctx, dunx, duny - 1, &mut totarea);
            } else {
                found = true;
            }
            let place_pool = rnd(ctx, 100) < 25;
            for j in (duny - totarea).max(0)..(duny + totarea).min(DY) {
                for i in (dunx - totarea).max(0)..(dunx + totarea).min(DX) {
                    // BUGFIX: In the following swap the order to first do the
                    // index checks and only then access dungeon[i][j] (fixed)
                    let v = d(ctx, i, j);
                    if (v & 0x80) != 0 {
                        let v = v & !0x80;
                        sd(ctx, i, j, v);
                        if totarea > 4 && place_pool && !found {
                            let k = POOLSUB[v as usize];
                            if k != 0 && k <= 37 {
                                sd(ctx, i, j, k);
                            }
                            lave_pool_placed = true;
                        }
                    }
                }
            }
        }
    }
    lave_pool_placed
}

/// Original: `PlacePool` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::PlacePool() sha=54f884982b3a
fn place_pool(ctx: &mut Ctx) -> bool {
    if ctx.gendung.leveltype == DungeonType::Nest {
        return place_slime_pool(ctx);
    }
    place_lava_pool(ctx)
}

/// Original: `PoolFix` (levels/drlg_l3.cpp): fills lava pools that River() only outlined.
// @port levels/drlg_l3.cpp|devilution::PoolFix() sha=542c46b92f2e
fn pool_fix(ctx: &mut Ctx) {
    for tile in crate::engine::geometry::points_in_rectangle(Rectangle::new(Point::new(1, 1), Size::new(DX - 2, DY - 2))) {
        // Check if the tile is a the default dirt ceiling tile
        if d(ctx, tile.x, tile.y) != 8 {
            continue;
        }
        for adjacent in crate::engine::geometry::points_in_rectangle(Rectangle::new(tile - Displacement::new(1, 1), Size::new(3, 3))) {
            let tile_id = d(ctx, adjacent.x, adjacent.y);
            // Check if the adjacent tile is a ground lava tile
            if (25..=41).contains(&tile_id) {
                // A ground lava tile can never be directly connected to our ceiling tile.
                // There must always be a kind of transition tile between (from ground to ceiling).
                // That means our tile is part of a lava pool (and was missed in River()), so we should change our tile to a ground lava tile.
                sd(ctx, tile.x, tile.y, 33);
                break;
            }
        }
    }
}

fn not_fence(v: u8) -> bool {
    !(130..=152).contains(&v)
}

/// Original: `FenceVerticalUp` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::FenceVerticalUp(int i, int y) sha=f8bfa76708f1
fn fence_vertical_up(ctx: &Ctx, i: i32, y: i32) -> bool {
    not_fence(d(ctx, i + 1, y)) && not_fence(d(ctx, i - 1, y)) && matches!(d(ctx, i, y), 7 | 10 | 126 | 129 | 134 | 136)
}

/// Original: `FenceVerticalDown` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::FenceVerticalDown(int i, int y) sha=bf0325c06a95
fn fence_vertical_down(ctx: &Ctx, i: i32, y: i32) -> bool {
    not_fence(d(ctx, i + 1, y)) && not_fence(d(ctx, i - 1, y)) && matches!(d(ctx, i, y), 2 | 7 | 134 | 136)
}

/// Original: `FenceHorizontalLeft` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::FenceHorizontalLeft(int x, int j) sha=5e6eaf410cc4
fn fence_horizontal_left(ctx: &Ctx, x: i32, j: i32) -> bool {
    not_fence(d(ctx, x, j + 1)) && not_fence(d(ctx, x, j - 1)) && matches!(d(ctx, x, j), 7 | 9 | 121 | 124 | 135 | 137)
}

/// Original: `FenceHorizontalRight` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::FenceHorizontalRight(int x, int j) sha=7f48c29d9d82
fn fence_horizontal_right(ctx: &Ctx, x: i32, j: i32) -> bool {
    not_fence(d(ctx, x, j + 1)) && not_fence(d(ctx, x, j - 1)) && matches!(d(ctx, x, j), 4 | 7 | 135 | 137)
}

/// Original: `AddFenceDoors` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::AddFenceDoors() sha=da936d83adff
fn add_fence_doors(ctx: &mut Ctx) {
    let is_fence = |v: u8| (130..=152).contains(&v);
    for j in 0..DY {
        for i in 0..DX {
            if d(ctx, i, j) == 7 && is_fence(d(ctx, i - 1, j)) && is_fence(d(ctx, i + 1, j)) {
                sd(ctx, i, j, 146);
                continue;
            }
            if d(ctx, i, j) == 7 && is_fence(d(ctx, i, j - 1)) && is_fence(d(ctx, i, j + 1)) {
                sd(ctx, i, j, 147);
                continue;
            }
        }
    }
}

/// Original: `FenceDoorFix` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::FenceDoorFix() sha=dff627c70823
fn fence_door_fix(ctx: &mut Ctx) {
    for j in 0..DY {
        for i in 0..DX {
            if d(ctx, i, j) == 146 && (not_fence(d(ctx, i + 1, j)) || not_fence(d(ctx, i - 1, j))) {
                sd(ctx, i, j, 7);
                continue;
            }
            const H: [u8; 7] = [130, 132, 133, 134, 136, 138, 140];
            if d(ctx, i, j) == 146 && !H.contains(&d(ctx, i + 1, j)) && !H.contains(&d(ctx, i - 1, j)) {
                sd(ctx, i, j, 7);
                continue;
            }
            if d(ctx, i, j) == 147 && (not_fence(d(ctx, i, j + 1)) || not_fence(d(ctx, i, j - 1))) {
                sd(ctx, i, j, 7);
                continue;
            }
            const V: [u8; 7] = [131, 132, 133, 135, 137, 138, 139];
            if d(ctx, i, j) == 147 && !V.contains(&d(ctx, i, j + 1)) && !V.contains(&d(ctx, i, j - 1)) {
                sd(ctx, i, j, 7);
                continue;
            }
        }
    }
}

/// Original: `Fence` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::Fence() sha=aa1d687f6b84
fn fence(ctx: &mut Ctx) {
    for j in 1..DY - 1 {
        // BUGFIX: Change '0' to '1' (fixed)
        for i in 1..DX - 1 {
            // BUGFIX: Change '0' to '1' (fixed)
            if d(ctx, i, j) == 10 && !flip(ctx) {
                let mut x = i;
                while d(ctx, x, j) == 10 {
                    x += 1;
                }
                x -= 1;
                if x - i > 0 {
                    sd(ctx, i, j, 127);
                    for xx in i + 1..x {
                        let v = ctx.rng.pick_randomly_among(&[129, 126]);
                        sd(ctx, xx, j, v);
                    }
                    sd(ctx, x, j, 128);
                }
            }
            if d(ctx, i, j) == 9 && !flip(ctx) {
                let mut y = j;
                while d(ctx, i, y) == 9 {
                    y += 1;
                }
                y -= 1;
                if y - j > 0 {
                    sd(ctx, i, j, 123);
                    for yy in j + 1..y {
                        let v = ctx.rng.pick_randomly_among(&[124, 121]);
                        sd(ctx, i, yy, v);
                    }
                    sd(ctx, i, y, 122);
                }
            }
            if d(ctx, i, j) == 11 && d(ctx, i + 1, j) == 10 && d(ctx, i, j + 1) == 9 && !flip(ctx) {
                sd(ctx, i, j, 125);
                let mut x = i + 1;
                while d(ctx, x, j) == 10 {
                    x += 1;
                }
                x -= 1;
                for xx in i + 1..x {
                    let v = ctx.rng.pick_randomly_among(&[129, 126]);
                    sd(ctx, xx, j, v);
                }
                sd(ctx, x, j, 128);
                let mut y = j + 1;
                while d(ctx, i, y) == 9 {
                    y += 1;
                }
                y -= 1;
                for yy in j + 1..y {
                    let v = ctx.rng.pick_randomly_among(&[124, 121]);
                    sd(ctx, i, yy, v);
                }
                sd(ctx, i, y, 122);
            }
        }
    }
    for j in 1..DY {
        // BUGFIX: Change '0' to '1' (fixed)
        for i in 1..DX {
            // BUGFIX: Change '0' to '1' (fixed)
            if d(ctx, i, j) != 7 {
                continue;
            }
            // note the comma operator is used here to advance the RNG state
            ctx.rng.discard_random_values(1);
            if is_near_theme_room(ctx, Point::new(i, j)) {
                continue;
            }
            if flip(ctx) {
                let mut y1 = j;
                // BUGFIX: Check `y1 >= 0` first (fixed)
                while y1 >= 0 && fence_vertical_up(ctx, i, y1) {
                    y1 -= 1;
                }
                y1 += 1;
                let mut y2 = j;
                // BUGFIX: Check `y2 < DMAXY` first (fixed)
                while y2 < DY && fence_vertical_down(ctx, i, y2) {
                    y2 += 1;
                }
                y2 -= 1;
                let mut skip = true;
                if d(ctx, i, y1) == 7 {
                    skip = false;
                }
                if d(ctx, i, y2) == 7 {
                    skip = false;
                }
                if y2 - y1 > 1 && skip {
                    let rp = rnd(ctx, y2 - y1 - 1) + y1 + 1;
                    for y in y1..=y2 {
                        if y == rp {
                            continue;
                        }
                        if d(ctx, i, y) == 7 {
                            let v = ctx.rng.pick_randomly_among(&[137, 135]);
                            sd(ctx, i, y, v);
                        }
                        let v = match d(ctx, i, y) {
                            10 => 131,
                            126 | 129 => 133,
                            2 => 139,
                            134 | 136 => 138,
                            other => other,
                        };
                        sd(ctx, i, y, v);
                    }
                }
            } else {
                let mut x1 = i;
                // BUGFIX: Check `x1 >= 0` first (fixed)
                while x1 >= 0 && fence_horizontal_left(ctx, x1, j) {
                    x1 -= 1;
                }
                x1 += 1;
                let mut x2 = i;
                // BUGFIX: Check `x2 < DMAXX` first (fixed)
                while x2 < DX && fence_horizontal_right(ctx, x2, j) {
                    x2 += 1;
                }
                x2 -= 1;
                let mut skip = true;
                if d(ctx, x1, j) == 7 {
                    skip = false;
                }
                if d(ctx, x2, j) == 7 {
                    skip = false;
                }
                if x2 - x1 > 1 && skip {
                    let rp = rnd(ctx, x2 - x1 - 1) + x1 + 1;
                    for x in x1..=x2 {
                        if x == rp {
                            continue;
                        }
                        if d(ctx, x, j) == 7 {
                            let v = ctx.rng.pick_randomly_among(&[136, 134]);
                            sd(ctx, x, j, v);
                        }
                        let v = match d(ctx, x, j) {
                            9 => 130,
                            121 | 124 => 132,
                            4 => 140,
                            135 | 137 => 138,
                            other => other,
                        };
                        sd(ctx, x, j, v);
                    }
                }
            }
        }
    }
    add_fence_doors(ctx);
    fence_door_fix(ctx);
}

/// Original: `LoadQuestSetPieces` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::LoadQuestSetPieces() sha=4e682b1e8d9f
fn load_quest_set_pieces(ctx: &mut Ctx) {
    if crate::quests::is_quest_available(ctx, Q_ANVIL) {
        ctx.gendung.pSetPiece = Some(load_u16_file(ctx, "levels\\l3data\\anvil.dun"));
    }
}

/// Original: `PlaceAnvil` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::PlaceAnvil() sha=b368644d6152
fn place_anvil(ctx: &mut Ctx) -> bool {
    let sp = ctx.gendung.pSetPiece.clone().expect("pSetPiece");
    // growing the size by 2 to allow a 1 tile border on all sides
    let area = Size::new(((sp[0] + 2) & 0xff) as i32, ((sp[1] + 2) & 0xff) as i32);
    let mut sx = rnd(ctx, DX - area.width) as u8;
    let mut sy = rnd(ctx, DY - area.height) as u8;
    let mut trys = 0;
    loop {
        if trys > 198 {
            return false;
        }
        if sx as i32 == DX - area.width {
            sx = 0;
            sy = sy.wrapping_add(1);
            if sy as i32 == DY - area.height {
                sy = 0;
            }
        }
        let mut found = true;
        for tile in crate::engine::geometry::points_in_rectangle(Rectangle::new(Point::new(sx as i32, sy as i32), area)) {
            if ctx.gendung.Protected.test(tile.x, tile.y) || d(ctx, tile.x, tile.y) != 7 {
                found = false;
                break;
            }
        }
        if found {
            break;
        }
        trys += 1;
        sx = sx.wrapping_add(1);
    }
    place_dun_tiles(ctx, &sp, Point::new(sx as i32 + 1, sy as i32 + 1), 7);
    ctx.gendung.SetPiece = Rectangle::new(Point::new(sx as i32, sy as i32), area);
    for tile in crate::engine::geometry::points_in_rectangle(ctx.gendung.SetPiece) {
        ctx.gendung.Protected.set(tile.x, tile.y);
    }
    // Hack to avoid rivers entering the island, reversed later
    let p = ctx.gendung.SetPiece.position;
    sd(ctx, p.x + 7, p.y + 5, 2);
    sd(ctx, p.x + 8, p.y + 5, 2);
    sd(ctx, p.x + 9, p.y + 5, 2);
    true
}

/// Original: `Warp` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::Warp() sha=c9e27dffe968
fn warp(ctx: &mut Ctx) {
    for j in 0..DY {
        for i in 0..DX {
            if d(ctx, i, j) == 125 && d(ctx, i + 1, j) == 125 && d(ctx, i, j + 1) == 125 && d(ctx, i + 1, j + 1) == 125 {
                sd(ctx, i, j, 156);
                sd(ctx, i + 1, j, 155);
                sd(ctx, i, j + 1, 153);
                sd(ctx, i + 1, j + 1, 154);
                return;
            }
            if d(ctx, i, j) == 5 && d(ctx, i + 1, j + 1) == 7 {
                sd(ctx, i, j, 7);
            }
        }
    }
}

/// Original: `HallOfHeroes` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::HallOfHeroes() sha=6d6bdbbdb103
fn hall_of_heroes(ctx: &mut Ctx) {
    for j in 0..DY {
        for i in 0..DX {
            if d(ctx, i, j) == 5 && d(ctx, i + 1, j + 1) == 7 {
                sd(ctx, i, j, 7);
            }
        }
    }
    for j in 0..DY {
        for i in 0..DX {
            if d(ctx, i, j) == 5 && d(ctx, i + 1, j + 1) == 12 && d(ctx, i + 1, j) == 7 {
                sd(ctx, i, j, 7);
                sd(ctx, i, j + 1, 7);
                sd(ctx, i + 1, j + 1, 7);
            }
            if d(ctx, i, j) == 5 && d(ctx, i + 1, j + 1) == 12 && d(ctx, i, j + 1) == 7 {
                sd(ctx, i, j, 7);
                sd(ctx, i + 1, j, 7);
                sd(ctx, i + 1, j + 1, 7);
            }
        }
    }
}

/// Original: `LockRectangle` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::LockRectangle(int x, int y) sha=470e6c89a888
fn lock_rectangle(ctx: &mut Ctx, x: i32, y: i32) {
    if !ctx.gendung.DungeonMask.test(x, y) {
        return;
    }
    ctx.gendung.DungeonMask.reset_at(x, y);
    ctx.drlg_l3.lockoutcnt += 1;
    lock_rectangle(ctx, x, y - 1);
    lock_rectangle(ctx, x, y + 1);
    lock_rectangle(ctx, x - 1, y);
    lock_rectangle(ctx, x + 1, y);
}

/// Original: `Lockout` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::Lockout() sha=efb69e1cc1f5
fn lockout(ctx: &mut Ctx) -> bool {
    ctx.gendung.DungeonMask.reset();
    let (mut fx, mut fy) = (0, 0);
    let mut t = 0;
    for j in 0..DY {
        for i in 0..DX {
            if d(ctx, i, j) != 0 {
                ctx.gendung.DungeonMask.set(i, j);
                fx = i;
                fy = j;
                t += 1;
            }
        }
    }
    ctx.drlg_l3.lockoutcnt = 0;
    lock_rectangle(ctx, fx, fy);
    t == ctx.drlg_l3.lockoutcnt
}

/// Original: `PlaceCaveStairs` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::PlaceCaveStairs(lvl_entry entry) sha=f48d32d52d21
fn place_cave_stairs(ctx: &mut Ctx, entry: lvl_entry) -> bool {
    // Place stairs up
    let Some(position) = place_mini_set(ctx, &L3UP, 199, false) else {
        return false;
    };
    if entry == ENTRY_MAIN {
        ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(1, 3);
    }
    // Place stairs down
    let Some(position) = place_mini_set(ctx, &L3DOWN, 199, false) else {
        return false;
    };
    if entry == ENTRY_PREV {
        ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(3, 1);
    }
    // Place town warp stairs
    if ctx.gendung.currlevel == 9 {
        let Some(position) = place_mini_set(ctx, &L3HOLDWARP, 199, false) else {
            return false;
        };
        if entry == ENTRY_TWARPDN {
            ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(1, 3);
        }
    }
    true
}

/// Original: `PlaceNestStairs` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::PlaceNestStairs(lvl_entry entry) sha=827ed70ed170
fn place_nest_stairs(ctx: &mut Ctx, entry: lvl_entry) -> bool {
    // Place stairs up
    let up = if ctx.gendung.currlevel != 17 { &L6UP } else { &L6HOLDWARP };
    let Some(position) = place_mini_set(ctx, up, 199, false) else {
        return false;
    };
    if entry == ENTRY_MAIN || entry == ENTRY_TWARPDN {
        ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(1, 3);
    }
    // Place stairs down
    if ctx.gendung.currlevel != 20 {
        let Some(position) = place_mini_set(ctx, &L6DOWN, 199, false) else {
            return false;
        };
        if entry == ENTRY_PREV {
            ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(3, 1);
        }
    }
    true
}

/// Original: `PlaceStairs` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::PlaceStairs(lvl_entry entry) sha=598361cf26be
fn place_stairs(ctx: &mut Ctx, entry: lvl_entry) -> bool {
    if ctx.gendung.leveltype == DungeonType::Nest {
        return place_nest_stairs(ctx, entry);
    }
    place_cave_stairs(ctx, entry)
}

/// Original: `GenerateLevel` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::GenerateLevel(lvl_entry entry) sha=1e4221d50987
fn generate_level(ctx: &mut Ctx, entry: lvl_entry) {
    use crate::quests::is_quest_available;
    load_quest_set_pieces(ctx);
    loop {
        init_dungeon_flags(ctx);
        let mut x1 = rnd(ctx, 20) + 10;
        let mut y1 = rnd(ctx, 20) + 10;
        let mut x2 = x1 + 2;
        let mut y2 = y1 + 2;
        fill_room(ctx, x1, y1, x2, y2);
        create_block(ctx, x1, y1, 2, 0);
        create_block(ctx, x2, y1, 2, 1);
        create_block(ctx, x1, y2, 2, 2);
        create_block(ctx, x1, y1, 2, 3);
        if is_quest_available(ctx, Q_ANVIL) {
            x1 = rnd(ctx, 10) + 10;
            y1 = rnd(ctx, 10) + 10;
            x2 = x1 + 12;
            y2 = y1 + 12;
            floor_area(ctx, x1, y1, x2, y2);
        }
        fill_diagonals(ctx);
        fill_singles(ctx);
        fill_straights(ctx);
        fill_diagonals(ctx);
        edges(ctx);
        if get_floor_area(ctx) < 600 || !lockout(ctx) {
            continue;
        }
        make_megas(ctx);
        if !place_stairs(ctx, entry) {
            continue;
        }
        if is_quest_available(ctx, Q_ANVIL) && !place_anvil(ctx) {
            continue;
        }
        if place_pool(ctx) {
            break;
        }
    }
    free_quest_set_pieces(ctx);
    if ctx.gendung.leveltype == DungeonType::Nest {
        for (m, p) in [(&L6ISLE1, 70), (&L6ISLE2, 70), (&L6ISLE3, 30), (&L6ISLE4, 30), (&L6ISLE1, 100), (&L6ISLE2, 100), (&L6ISLE5, 90)] {
            place_mini_set_random(ctx, m, p);
        }
        for r in [25, 26, 27, 28] {
            place_mini_set_random_1x1(ctx, 8, r, 20);
        }
        for (m, p) in [
            (&HivePattern29, 10),
            (&HivePattern30, 15),
            (&HivePattern31, 20),
            (&HivePattern32, 25),
            (&HivePattern33, 30),
            (&HivePattern34, 35),
            (&HivePattern35, 40),
            (&HivePattern36, 45),
            (&HivePattern37, 50),
            (&HivePattern38, 55),
            (&HivePattern38, 10),
            (&HivePattern37, 15),
            (&HivePattern36, 20),
            (&HivePattern35, 25),
            (&HivePattern34, 30),
            (&HivePattern33, 35),
            (&HivePattern32, 40),
            (&HivePattern31, 45),
            (&HivePattern30, 50),
            (&HivePattern29, 55),
            (&HivePattern9, 40),
            (&HivePattern10, 45),
        ] {
            place_mini_set_random(ctx, m, p);
        }
        for (s, r) in [
            (7, 29),
            (7, 30),
            (7, 31),
            (7, 32),
            (9, 33),
            (9, 34),
            (9, 35),
            (9, 36),
            (9, 37),
            (10, 39),
            (10, 40),
            (10, 41),
            (10, 42),
            (10, 43),
            (9, 45),
            (9, 46),
            (10, 47),
            (10, 48),
            (11, 38),
            (11, 44),
            (11, 49),
            (11, 50),
        ] {
            place_mini_set_random_1x1(ctx, s, r, 25);
        }
    } else {
        pool_fix(ctx);
        warp(ctx);
        for (m, p) in [(&L3ISLE1, 70), (&L3ISLE2, 70), (&L3ISLE3, 30), (&L3ISLE4, 30), (&L3ISLE1, 100), (&L3ISLE2, 100), (&L3ISLE5, 90)] {
            place_mini_set_random(ctx, m, p);
        }
        hall_of_heroes(ctx);
        river(ctx);
        if is_quest_available(ctx, Q_ANVIL) {
            let p = ctx.gendung.SetPiece.position;
            sd(ctx, p.x + 7, p.y + 5, 7);
            sd(ctx, p.x + 8, p.y + 5, 7);
            sd(ctx, p.x + 9, p.y + 5, 7);
            let v = d(ctx, p.x + 10, p.y + 5);
            if v == 17 || v == 18 {
                sd(ctx, p.x + 10, p.y + 5, 45);
            }
        }
        drlg_place_theme_rooms(ctx, 5, 10, 7, 0, false);
        fence(ctx);
        for (m, p) in [
            (&L3TITE1, 10),
            (&L3TITE2, 10),
            (&L3TITE3, 10),
            (&L3TITE6, 20),
            (&L3TITE7, 20),
            (&L3TITE8, 20),
            (&L3TITE9, 20),
            (&L3TITE10, 20),
            (&L3TITE11, 30),
            (&L3TITE12, 20),
            (&L3TITE13, 20),
            (&L3CREV1, 30),
            (&L3CREV2, 30),
            (&L3CREV3, 30),
            (&L3CREV4, 30),
            (&L3CREV5, 30),
            (&L3CREV6, 30),
            (&L3CREV7, 30),
            (&L3CREV8, 30),
            (&L3CREV9, 30),
            (&L3CREV10, 30),
            (&L3CREV11, 30),
        ] {
            place_mini_set_random(ctx, m, p);
        }
        place_mini_set_random_1x1(ctx, 7, 106, 25);
        place_mini_set_random_1x1(ctx, 7, 107, 25);
        place_mini_set_random_1x1(ctx, 7, 108, 25);
        place_mini_set_random_1x1(ctx, 9, 109, 25);
        place_mini_set_random_1x1(ctx, 10, 110, 25);
    }
    ctx.gendung.pdungeon = ctx.gendung.dungeon;
}

/// Original: `Pass3` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::Pass3() sha=8c3950eb0af8
fn pass3(ctx: &mut Ctx) {
    drlg_l_pass3(ctx, 8 - 1);
}

/// Original: `PlaceCaveLights` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::PlaceCaveLights() sha=383964d8a994
fn place_cave_lights(ctx: &mut Ctx) {
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            let p = ctx.gendung.dPiece[i][j];
            if (55..=146).contains(&p) || (153..=160).contains(&p) || p == 149 || p == 151 {
                crate::lighting::do_lighting(ctx, Point::new(i as i32, j as i32), 7, Displacement::default());
            }
        }
    }
}

/// Original: `PlaceHiveLights` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::PlaceHiveLights() sha=3e90cbae8223
fn place_hive_lights(ctx: &mut Ctx) {
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            if (381..=456).contains(&ctx.gendung.dPiece[i][j]) {
                crate::lighting::do_lighting(ctx, Point::new(i as i32, j as i32), 9, Displacement::default());
            }
        }
    }
}

/// Original: `PlaceLights` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::PlaceLights() sha=10d39f926abb
fn place_lights(ctx: &mut Ctx) {
    if ctx.gendung.leveltype == DungeonType::Nest {
        place_hive_lights(ctx);
        return;
    }
    place_cave_lights(ctx);
}

/// Original: `devilution::CreateL3Dungeon` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::CreateL3Dungeon(uint32_t rseed, lvl_entry entry) sha=ab1271ab85f7
pub fn create_l3_dungeon(ctx: &mut Ctx, rseed: u32, entry: lvl_entry) {
    ctx.rng.set_rnd_seed(rseed);
    generate_level(ctx, entry);
    pass3(ctx);
    place_lights(ctx);
}

/// Original: `devilution::LoadPreL3Dungeon` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::LoadPreL3Dungeon(const char *path) sha=d1e02f5147a5
pub fn load_pre_l3_dungeon(ctx: &mut Ctx, path: &str) {
    ctx.gendung.dungeon = [[8; DMAXY]; DMAXX];
    let dun_data = load_u16_file(ctx, path);
    place_dun_tiles(ctx, &dun_data, Point::new(0, 0), 7);
    ctx.gendung.pdungeon = ctx.gendung.dungeon;
}

/// Original: `devilution::LoadL3Dungeon` (levels/drlg_l3.cpp).
// @port levels/drlg_l3.cpp|devilution::LoadL3Dungeon(const char *path, Point spawn) sha=a4a036ecdd14
pub fn load_l3_dungeon(ctx: &mut Ctx, path: &str, spawn: Point) {
    load_dungeon_base(ctx, path, spawn, 7, 8);
    pass3(ctx);
    place_lights(ctx);
    if ctx.gendung.leveltype == DungeonType::Caves {
        crate::objects::add_l3_objs(ctx, 0, 0, MAXDUNX as i32, MAXDUNY as i32);
    }
}
