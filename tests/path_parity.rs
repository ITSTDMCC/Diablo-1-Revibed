//! The port of DevilutionX 1.5.3's `test/path_test.cpp` (pathfinding, solidity and the closest
//! valid position search), assertion for assertion.

use std::cell::RefCell;

use diablo1_rs::ctx::Ctx;
use diablo1_rs::engine::geometry::{Direction, Displacement, Point};
use diablo1_rs::engine::path::*;
use diablo1_rs::enums::*;

use diablo1_rs::platform::Platform;

fn ctx() -> Ctx {
    let mut ctx = Ctx::new(Platform::headless_from_env());
    ctx.diablo.headless_mode = true; // test/main.cpp: HeadlessMode = true
    ctx
}

#[test]
fn heuristics() {
    let source = Point::new(25, 32);
    let cost = |d: Point| get_heuristic_cost(source, d);
    assert_eq!(cost(source), 0, "Wrong cost for travelling to the same tile");
    for d in [Direction::NorthEast, Direction::SouthEast, Direction::SouthWest, Direction::NorthWest] {
        assert_eq!(cost(source + d), 2, "Wrong cost for travelling to horizontal/vertical adjacent tile");
    }
    for d in [Direction::North, Direction::East, Direction::South, Direction::West] {
        assert_eq!(cost(source + d), 4, "Wrong cost for travelling to diagonally adjacent tile");
    }
    assert_eq!(cost(source + Direction::SouthWest + Direction::SouthEast), 4, "Wrong cost for travelling to diagonally adjacent tile");
    assert_eq!(cost(source + Direction::NorthEast + Direction::North), 6, "Wrong cost for travelling to a {{ 2, 1 }} offset");
    assert_eq!(cost(source + Direction::SouthEast + Direction::SouthEast), 4, "Wrong cost for travelling to a {{ 2, 0 }} offset");
}

#[test]
fn solid() {
    let mut c = ctx();
    c.gendung.dPiece[5][5] = 0;
    c.gendung.SOLData[0] = TileProperties::Solid;
    assert!(is_tile_solid(&c, Point::new(5, 5)), "Solid in-bounds tiles are solid");
    assert!(!is_tile_not_solid(&c, Point::new(5, 5)), "IsTileNotSolid returns the inverse of IsTileSolid for in-bounds tiles");

    c.gendung.dPiece[6][6] = 1;
    c.gendung.SOLData[1] = TileProperties::None;
    assert!(!is_tile_solid(&c, Point::new(6, 6)), "Non-solid in-bounds tiles are not solid");
    assert!(is_tile_not_solid(&c, Point::new(6, 6)), "IsTileNotSolid returns the inverse of IsTileSolid for in-bounds tiles");

    assert!(!is_tile_solid(&c, Point::new(-1, 1)), "Out of bounds tiles are not solid");
    assert!(!is_tile_not_solid(&c, Point::new(-1, 1)), "Out of bounds tiles are also not not solid");
}

#[test]
fn solid_pieces() {
    let mut c = ctx();
    let p = |x, y| Point::new(x, y);
    for (x, y) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        c.gendung.dPiece[x][y] = 0;
    }
    c.gendung.SOLData[0] = TileProperties::None;
    assert!(path_solid_pieces(&c, p(0, 0), p(1, 1)), "A step in open space is free of solid pieces");
    assert!(path_solid_pieces(&c, p(1, 1), p(0, 0)), "A step in open space is free of solid pieces");
    assert!(path_solid_pieces(&c, p(1, 0), p(0, 1)), "A step in open space is free of solid pieces");
    assert!(path_solid_pieces(&c, p(0, 1), p(1, 0)), "A step in open space is free of solid pieces");

    c.gendung.SOLData[1] = TileProperties::Solid;
    c.gendung.dPiece[1][0] = 1;
    assert!(path_solid_pieces(&c, p(0, 1), p(1, 0)), "Can path to a destination which is solid");
    assert!(path_solid_pieces(&c, p(1, 0), p(0, 1)), "Can path from a starting position which is solid");
    assert!(path_solid_pieces(&c, p(0, 1), p(1, 1)), "Stepping in a cardinal direction ignores solid pieces");
    assert!(path_solid_pieces(&c, p(1, 0), p(1, 1)), "Stepping in a cardinal direction ignores solid pieces");
    assert!(path_solid_pieces(&c, p(0, 0), p(1, 0)), "Stepping in a cardinal direction ignores solid pieces");
    assert!(path_solid_pieces(&c, p(1, 1), p(1, 0)), "Stepping in a cardinal direction ignores solid pieces");

    assert!(!path_solid_pieces(&c, p(0, 0), p(1, 1)), "Can't cut a solid corner");
    assert!(!path_solid_pieces(&c, p(1, 1), p(0, 0)), "Can't cut a solid corner");
    c.gendung.dPiece[0][1] = 1;
    assert!(!path_solid_pieces(&c, p(0, 0), p(1, 1)), "Can't walk through the boundary between two corners");
    assert!(!path_solid_pieces(&c, p(1, 1), p(0, 0)), "Can't walk through the boundary between two corners");
    c.gendung.dPiece[1][0] = 0;
    assert!(!path_solid_pieces(&c, p(0, 0), p(1, 1)), "Can't cut a solid corner");
    assert!(!path_solid_pieces(&c, p(1, 1), p(0, 0)), "Can't cut a solid corner");
    c.gendung.dPiece[0][1] = 0;

    c.gendung.dPiece[0][0] = 1;
    assert!(!path_solid_pieces(&c, p(1, 0), p(0, 1)), "Can't cut a solid corner");
    assert!(!path_solid_pieces(&c, p(0, 1), p(1, 0)), "Can't cut a solid corner");
    c.gendung.dPiece[1][1] = 1;
    assert!(!path_solid_pieces(&c, p(1, 0), p(0, 1)), "Can't walk through the boundary between two corners");
    assert!(!path_solid_pieces(&c, p(0, 1), p(1, 0)), "Can't walk through the boundary between two corners");
    c.gendung.dPiece[0][0] = 0;
    assert!(!path_solid_pieces(&c, p(1, 0), p(0, 1)), "Can't cut a solid corner");
    assert!(!path_solid_pieces(&c, p(0, 1), p(1, 0)), "Can't cut a solid corner");
}

/// `CheckPath`
fn check_path(start: Point, destination: Point, expected: &[i8]) {
    let c = ctx();
    let mut steps = [0i8; diablo1_rs::engine::path::MaxPathLength];
    let len = find_path(&c, &|_, _| true, start, destination, &mut steps);
    assert_eq!(len as usize, expected.len(), "Wrong path length for a path from {start:?} to {destination:?}");
    assert_eq!(&steps[..len as usize], expected, "path from {start:?} to {destination:?}");
}

#[test]
fn find_path_steps() {
    let p = |x, y| Point::new(x, y);
    check_path(p(8, 8), p(8, 8), &[]);

    // Traveling in cardinal directions is the only way to get a first step in a cardinal direction
    check_path(p(8, 8), p(8, 6), &[1, 1]);
    check_path(p(8, 8), p(6, 8), &[2, 2]);
    check_path(p(8, 8), p(10, 8), &[3, 3]);
    check_path(p(8, 8), p(8, 10), &[4, 4]);

    // Otherwise pathing biases along diagonals and the diagonal steps will always be first
    check_path(p(8, 8), p(5, 6), &[5, 5, 2]);
    check_path(p(8, 8), p(4, 4), &[5, 5, 5, 5]);
    check_path(p(8, 8), p(12, 20), &[7, 7, 7, 7, 4, 4, 4, 4, 4, 4, 4, 4]);
}

#[test]
fn long_paths() {
    // Starting from the middle of the world and trying to path to a border exceeds the maximum path size
    check_path(Point::new(56, 56), Point::new(0, 0), &[]);

    // Longest possible path is currently 24 steps meaning tiles 24 units away are reachable
    let start = Point::new(56, 56);
    check_path(start, start + Displacement::new(24, 24), &[7; 24]);

    // But trying to navigate 25 units fails
    check_path(start, start + Displacement::new(25, 25), &[]);
}

#[test]
fn walkable() {
    let mut c = ctx();
    let p = Point::new(5, 5);
    c.gendung.dPiece[5][5] = 0;
    c.gendung.SOLData[0] = TileProperties::Solid;
    assert!(!is_tile_walkable(&c, p, false), "Tile which is marked as solid should be considered blocked");
    assert!(!is_tile_walkable(&c, p, true), "Solid non-door tiles remain unwalkable when ignoring doors");

    c.gendung.SOLData[0] = TileProperties::None;
    assert!(is_tile_walkable(&c, p, false), "Non-solid tiles are walkable");
    assert!(is_tile_walkable(&c, p, true), "Non-solid tiles remain walkable when ignoring doors");

    c.gendung.dObject[5][5] = 1;
    c.objects.Objects[0]._oSolidFlag = true;
    assert!(!is_tile_walkable(&c, p, false), "Tile occupied by a solid object is unwalkable");
    assert!(!is_tile_walkable(&c, p, true), "Tile occupied by a solid non-door object are unwalkable when ignoring doors");

    c.objects.Objects[0]._otype = OBJ_L1LDOOR;
    assert!(!is_tile_walkable(&c, p, false), "Tile occupied by a door which is marked as solid should be considered blocked");
    assert!(is_tile_walkable(&c, p, true), "Tile occupied by a door is considered walkable when ignoring doors");

    c.objects.Objects[0]._oSolidFlag = false;
    assert!(is_tile_walkable(&c, p, false), "Tile occupied by an open door is walkable");
    assert!(is_tile_walkable(&c, p, true), "Tile occupied by a door is considered walkable when ignoring doors");

    c.gendung.SOLData[0] = TileProperties::Solid;
    assert!(!is_tile_walkable(&c, p, false), "Solid tiles occupied by an open door remain unwalkable");
    assert!(is_tile_walkable(&c, p, true), "Solid tiles occupied by an open door become walkable when ignoring doors");
}

fn searched<const N: usize>(start: (i32, i32), min: u32, max: u32) -> (Option<Point>, [[i32; N]; N]) {
    let c = ctx();
    let tiles = RefCell::new([[0i32; N]; N]);
    let r = find_closest_valid_position(
        &c,
        &|_, t: Point| {
            tiles.borrow_mut()[t.x as usize][t.y as usize] += 1;
            false
        },
        Point::new(start.0, start.1),
        min,
        max,
    );
    (r, tiles.into_inner())
}

#[test]
fn find_closest() {
    {
        let (near, tiles) = searched::<101>((50, 50), 0, 50);
        assert!(near.is_none(), "Searching with no valid tiles should return an empty optional");
        for x in 0..101 {
            for y in 0..101 {
                if (x == 0 || x == 100) && (y == 0 || y == 100) {
                    assert_eq!(tiles[x][y], 0, "Extreme corners should be skipped due to the inset/rounded search space");
                } else {
                    assert_eq!(tiles[x][y], 1, "Position {x} {y} should have been searched exactly once");
                }
            }
        }
    }
    {
        let (near, tiles) = searched::<5>((2, 2), 1, 2);
        assert!(near.is_none(), "Still shouldn't find a valid position with no valid tiles");
        for x in 0..5 {
            for y in 0..5 {
                if x == 2 && y == 2 {
                    assert_eq!(tiles[x][y], 0, "The starting tile should be skipped with a min radius of 1");
                } else if (x == 0 || x == 4) && (y == 0 || y == 4) {
                    assert_eq!(tiles[x][y], 0, "Corners should be skipped");
                } else {
                    assert_eq!(tiles[x][y], 1, "All tiles in range should be searched exactly once");
                }
            }
        }
    }
    {
        let (near, tiles) = searched::<3>((1, 1), 0, 0);
        assert!(near.is_none(), "Searching with no valid tiles should return an empty optional");
        for x in 0..3 {
            for y in 0..3 {
                let expected = if x == 1 && y == 1 { 1 } else { 0 };
                assert_eq!(tiles[x][y], expected, "Only the starting tile should be searched with max radius 0 ({x} {y})");
            }
        }
    }
    {
        let (near, tiles) = searched::<7>((3, 3), 3, 3);
        assert!(near.is_none(), "Searching with no valid tiles should return an empty optional");
        for x in 0..7 {
            for y in 0..7 {
                let ring = ((x == 1 || x == 5) && (y == 1 || y == 5))
                    || ((x == 0 || x == 6) && y != 0 && y != 6)
                    || (x != 0 && x != 6 && (y == 0 || y == 6));
                assert_eq!(tiles[x][y], ring as i32, "Searching with a fixed radius should make a square with inset corners ({x} {y})");
            }
        }
    }
    {
        let c = ctx();
        let near = find_closest_valid_position(&c, &|_, _| true, Point::new(50, 50), 21, 50);
        assert_eq!(near, Some(Point::new(50, 50) + Displacement::new(0, 21)), "First candidate position with a minimum radius should be at {{0, +y}}");
    }
}
