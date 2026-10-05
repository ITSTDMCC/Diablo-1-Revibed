//! `Source/engine/path.cpp`: A* path finding over the dungeon tile grid, and tile walkability.

use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::enums::TileProperties;
use crate::levels::gendung::{in_dungeon_bounds, tile_has_any};

pub const MaxPathLength: usize = 25;
const MAX_PATH_NODES: usize = 300;
const INVALID_INDEX: u16 = u16::MAX;
const MAX_CHILDREN: usize = 8;

/// `PathDirs`
pub const PATH_DIRS: [Displacement; 8] = [
    Displacement::new(-1, -1), // North
    Displacement::new(-1, 1),  // West
    Displacement::new(1, -1),  // East
    Displacement::new(1, 1),   // South
    Displacement::new(-1, 0),  // NorthWest
    Displacement::new(0, -1),  // NorthEast
    Displacement::new(1, 0),   // SouthEast
    Displacement::new(0, 1),   // SouthWest
];

/// `PathNode`
#[derive(Clone, Copy)]
struct PathNode {
    x: i16,
    y: i16,
    parent_index: u16,
    child_indices: [u16; MAX_CHILDREN],
    next_node_index: u16,
    f: u8,
    h: u8,
    g: u8,
}

impl Default for PathNode {
    fn default() -> Self {
        PathNode { x: 0, y: 0, parent_index: INVALID_INDEX, child_indices: [INVALID_INDEX; MAX_CHILDREN], next_node_index: INVALID_INDEX, f: 0, h: 0, g: 0 }
    }
}

impl PathNode {
    fn position(&self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }

    /// Original: `PathNode::addChild` (engine/path.cpp).
    // @port engine/path.cpp|devilution::PathNode::addChild(uint16_t childIndex) sha=d11bd5731c15
    fn add_child(&mut self, child_index: u16) {
        let index = self.child_indices.iter().position(|&c| c == INVALID_INDEX).unwrap_or(MAX_CHILDREN);
        assert!(index < MAX_CHILDREN);
        self.child_indices[index] = child_index;
    }
}

/// The search state (`PathNodes`, `Path2Nodes`, `VisitedNodes`, `gdwCurNodes`, `pnode_tblptr`,
/// `gdwCurPathStep`, file statics in the original).
struct PathSearch {
    nodes: Vec<PathNode>,
    path2_nodes: usize,
    visited_nodes: usize,
    cur_nodes: usize,
    tblptr: [u16; MAX_PATH_NODES],
    cur_path_step: usize,
}

impl PathSearch {
    /// Original: `GetNode1` (engine/path.cpp).
    // @port engine/path.cpp|devilution::GetNode1(Point targetPosition) sha=0d083a53f253
    fn get_node1(&self, target: Point) -> u16 {
        let mut result = self.nodes[self.path2_nodes].next_node_index;
        while result != INVALID_INDEX {
            if self.nodes[result as usize].position() == target {
                return result;
            }
            result = self.nodes[result as usize].next_node_index;
        }
        INVALID_INDEX
    }

    /// Original: `NextNode` (engine/path.cpp).
    // @port engine/path.cpp|devilution::NextNode(uint16_t front) sha=1c561b11ab35
    fn next_node(&mut self, front: u16) {
        if self.nodes[self.path2_nodes].next_node_index == INVALID_INDEX {
            self.nodes[self.path2_nodes].next_node_index = front;
            return;
        }
        let mut current = self.path2_nodes;
        let mut next_index = self.nodes[self.path2_nodes].next_node_index;
        let max_f = self.nodes[front as usize].f;
        while next_index != INVALID_INDEX && self.nodes[next_index as usize].f < max_f {
            current = next_index as usize;
            next_index = self.nodes[current].next_node_index;
        }
        self.nodes[front as usize].next_node_index = next_index;
        self.nodes[current].next_node_index = front;
    }

    /// Original: `GetNode2` (engine/path.cpp).
    // @port engine/path.cpp|devilution::GetNode2(Point targetPosition) sha=ed7aa80f1770
    fn get_node2(&self, target: Point) -> u16 {
        let mut result = self.nodes[self.visited_nodes].next_node_index;
        while result != INVALID_INDEX {
            if self.nodes[result as usize].position() == target {
                return result;
            }
            result = self.nodes[result as usize].next_node_index;
        }
        result
    }

    /// Original: `GetNextPath` (engine/path.cpp).
    // @port engine/path.cpp|devilution::GetNextPath() sha=91ec56f98f88
    fn get_next_path(&mut self) -> u16 {
        let result = self.nodes[self.path2_nodes].next_node_index;
        if result == INVALID_INDEX {
            return result;
        }
        self.nodes[self.path2_nodes].next_node_index = self.nodes[result as usize].next_node_index;
        self.nodes[result as usize].next_node_index = self.nodes[self.visited_nodes].next_node_index;
        self.nodes[self.visited_nodes].next_node_index = result;
        result
    }

    /// Original: `NewStep` (engine/path.cpp).
    // @port engine/path.cpp|devilution::NewStep() sha=626070134c9d
    fn new_step(&mut self) -> u16 {
        if self.cur_nodes >= MAX_PATH_NODES {
            return INVALID_INDEX;
        }
        self.nodes[self.cur_nodes] = PathNode::default();
        self.cur_nodes += 1;
        (self.cur_nodes - 1) as u16
    }

    /// Original: `PushActiveStep` (engine/path.cpp).
    // @port engine/path.cpp|devilution::PushActiveStep(uint16_t pPath) sha=e95c33ede46d
    fn push_active_step(&mut self, p: u16) {
        assert!(self.cur_path_step < MAX_PATH_NODES);
        self.tblptr[self.cur_path_step] = p;
        self.cur_path_step += 1;
    }

    /// Original: `PopActiveStep` (engine/path.cpp).
    // @port engine/path.cpp|devilution::PopActiveStep() sha=b2075e061b0c
    fn pop_active_step(&mut self) -> u16 {
        self.cur_path_step -= 1;
        self.tblptr[self.cur_path_step]
    }

    /// Original: `SetCoords` (engine/path.cpp).
    // @port engine/path.cpp|devilution::SetCoords(uint16_t pPath) sha=298c1b636266
    fn set_coords(&mut self, ctx: &Ctx, p: u16) {
        self.push_active_step(p);
        while self.cur_path_step > 0 {
            let path_old_index = self.pop_active_step();
            let path_old = self.nodes[path_old_index as usize];
            for &child_index in path_old.child_indices.iter() {
                if child_index == INVALID_INDEX {
                    break;
                }
                let path_act = self.nodes[child_index as usize];
                let step = check_equal(path_old.position(), path_act.position());
                if (path_old.g as i32 + step) < path_act.g as i32 && path_solid_pieces(ctx, path_old.position(), path_act.position()) {
                    let a = &mut self.nodes[child_index as usize];
                    a.parent_index = path_old_index;
                    a.g = (path_old.g as i32 + step) as u8;
                    a.f = a.g.wrapping_add(a.h);
                    self.push_active_step(child_index);
                }
            }
        }
    }

    /// Original: `ParentPath` (engine/path.cpp).
    // @port engine/path.cpp|devilution::ParentPath(uint16_t pathIndex, Point candidatePosition, Point destinationPosition) sha=f42bd03e5493
    fn parent_path(&mut self, ctx: &Ctx, path_index: u16, candidate: Point, destination: Point) -> bool {
        let path = self.nodes[path_index as usize];
        let next_g = path.g as i32 + check_equal(path.position(), candidate);
        let mut dxdy_index = self.get_node1(candidate);
        if dxdy_index != INVALID_INDEX {
            self.nodes[path_index as usize].add_child(dxdy_index);
            let dxdy = self.nodes[dxdy_index as usize];
            if next_g < dxdy.g as i32 && path_solid_pieces(ctx, path.position(), candidate) {
                let d = &mut self.nodes[dxdy_index as usize];
                d.parent_index = path_index;
                d.g = next_g as u8;
                d.f = (next_g + d.h as i32) as u8;
            }
        } else {
            dxdy_index = self.get_node2(candidate);
            if dxdy_index != INVALID_INDEX {
                self.nodes[path_index as usize].add_child(dxdy_index);
                let dxdy = self.nodes[dxdy_index as usize];
                if next_g < dxdy.g as i32 && path_solid_pieces(ctx, path.position(), candidate) {
                    let d = &mut self.nodes[dxdy_index as usize];
                    d.parent_index = path_index;
                    d.g = next_g as u8;
                    d.f = (next_g + d.h as i32) as u8;
                    self.set_coords(ctx, dxdy_index);
                }
            } else {
                dxdy_index = self.new_step();
                if dxdy_index == INVALID_INDEX {
                    return false;
                }
                let d = &mut self.nodes[dxdy_index as usize];
                d.parent_index = path_index;
                d.g = next_g as u8;
                d.h = get_heuristic_cost(candidate, destination) as u8;
                d.f = (next_g + d.h as i32) as u8;
                d.x = candidate.x as i16;
                d.y = candidate.y as i16;
                self.next_node(dxdy_index);
                self.nodes[path_index as usize].add_child(dxdy_index);
            }
        }
        true
    }

    /// Original: `GetPath` (engine/path.cpp).
    // @port engine/path.cpp|devilution::GetPath(tl::function_ref<bool(Point)> posOk, uint16_t pathIndex, Point destination) sha=7e65bc874be4
    fn get_path(&mut self, ctx: &Ctx, pos_ok: &dyn Fn(&Ctx, Point) -> bool, path_index: u16, destination: Point) -> bool {
        for dir in PATH_DIRS {
            let pos = self.nodes[path_index as usize].position();
            let tile = pos + dir;
            let ok = pos_ok(ctx, tile);
            if ((ok && path_solid_pieces(ctx, pos, tile)) || (!ok && tile == destination)) && !self.parent_path(ctx, path_index, tile, destination) {
                return false;
            }
        }
        true
    }
}

/// Original: `CheckEqual` (engine/path.cpp).
// @port engine/path.cpp|devilution::CheckEqual(Point startPosition, Point destinationPosition) sha=6ceb4d647897
fn check_equal(a: Point, b: Point) -> i32 {
    if a.x == b.x || a.y == b.y {
        2
    } else {
        3
    }
}

/// Original: `GetPathDirection` (engine/path.cpp).
// @port engine/path.cpp|devilution::GetPathDirection(Point startPosition, Point destinationPosition) sha=f3a916be52ee
fn get_path_direction(start: Point, destination: Point) -> i8 {
    const PATH_DIRECTIONS: [i8; 9] = [5, 1, 6, 2, 0, 3, 8, 4, 7];
    PATH_DIRECTIONS[(3 * (destination.y - start.y) + 4 + destination.x - start.x) as usize]
}

/// Original: `GetHeuristicCost` (engine/path.cpp).
// @port engine/path.cpp|devilution::GetHeuristicCost(Point startPosition, Point destinationPosition) sha=56171f2acce8
fn get_heuristic_cost(start: Point, destination: Point) -> i32 {
    2 * start.manhattan_distance(destination)
}

/// Original: `devilution::IsTileNotSolid` (engine/path.cpp).
// @port engine/path.cpp|devilution::IsTileNotSolid(Point position) sha=15a262f43bec
pub fn is_tile_not_solid(ctx: &Ctx, position: Point) -> bool {
    if !in_dungeon_bounds(position) {
        return false;
    }
    !tile_has_any(ctx, ctx.gendung.dPiece[position.x as usize][position.y as usize] as i32, TileProperties::Solid)
}

/// Original: `devilution::IsTileSolid` (engine/path.cpp).
// @port engine/path.cpp|devilution::IsTileSolid(Point position) sha=f1b343e3cd40
pub fn is_tile_solid(ctx: &Ctx, position: Point) -> bool {
    if !in_dungeon_bounds(position) {
        return false;
    }
    tile_has_any(ctx, ctx.gendung.dPiece[position.x as usize][position.y as usize] as i32, TileProperties::Solid)
}

/// Original: `devilution::IsTileWalkable` (engine/path.cpp). `ignoreDoors` defaults to false.
// @port engine/path.cpp|devilution::IsTileWalkable(Point position, bool ignoreDoors) sha=445fe4310cf3
pub fn is_tile_walkable(ctx: &Ctx, position: Point, ignore_doors: bool) -> bool {
    if let Some(oi) = crate::objects::find_object_at_position(ctx, position, true) {
        let object = &ctx.objects.Objects[oi];
        if ignore_doors && object.is_door() {
            return true;
        }
        if object._oSolidFlag {
            return false;
        }
    }
    !is_tile_solid(ctx, position)
}

/// Original: `devilution::IsTileOccupied` (engine/path.cpp).
// @port engine/path.cpp|devilution::IsTileOccupied(Point position) sha=e019d8798786
pub fn is_tile_occupied(ctx: &Ctx, position: Point) -> bool {
    if !in_dungeon_bounds(position) {
        return true;
    }
    if is_tile_solid(ctx, position) {
        return true;
    }
    let (x, y) = (position.x as usize, position.y as usize);
    if ctx.gendung.dMonster[x][y] != 0 || ctx.gendung.dPlayer[x][y] != 0 {
        return true;
    }
    crate::objects::is_object_at_position(ctx, position)
}

/// Original: `devilution::FindPath` (engine/path.cpp): writes up to `MaxPathLength` steps.
// @port engine/path.cpp|devilution::FindPath(tl::function_ref<bool(Point)> posOk, Point startPosition, Point destinationPosition, int8_t path[MaxPathLength]) sha=d729b135b34e
pub fn find_path(ctx: &Ctx, pos_ok: &dyn Fn(&Ctx, Point) -> bool, start: Point, destination: Point, path: &mut [i8; MaxPathLength]) -> i32 {
    let mut pnode_vals = [0i8; MaxPathLength];
    let mut s = PathSearch {
        nodes: vec![PathNode::default(); MAX_PATH_NODES],
        path2_nodes: 0,
        visited_nodes: 0,
        cur_nodes: 0,
        tblptr: [0; MAX_PATH_NODES],
        cur_path_step: 0,
    };
    s.path2_nodes = s.new_step() as usize;
    s.visited_nodes = s.new_step() as usize;
    s.cur_path_step = 0;
    let path_start_index = s.new_step();
    {
        let ps = &mut s.nodes[path_start_index as usize];
        ps.x = start.x as i16;
        ps.y = start.y as i16;
        ps.f = ps.h.wrapping_add(ps.g);
        ps.h = get_heuristic_cost(start, destination) as u8;
        ps.g = 0;
    }
    s.nodes[s.path2_nodes].next_node_index = path_start_index;
    loop {
        let next_node_index = s.get_next_path();
        if next_node_index == INVALID_INDEX {
            break;
        }
        if s.nodes[next_node_index as usize].position() == destination {
            let mut current = next_node_index as usize;
            let mut path_length = 0usize;
            while s.nodes[current].parent_index != INVALID_INDEX {
                if path_length >= MaxPathLength {
                    break;
                }
                let parent = s.nodes[current].parent_index as usize;
                pnode_vals[path_length] = get_path_direction(s.nodes[parent].position(), s.nodes[current].position());
                path_length += 1;
                current = parent;
            }
            if path_length != MaxPathLength {
                for i in 0..path_length {
                    path[i] = pnode_vals[path_length - i - 1];
                }
                return path_length as i32;
            }
            return 0;
        }
        if !s.get_path(ctx, pos_ok, next_node_index, destination) {
            return 0;
        }
    }
    0
}

/// Original: `devilution::path_solid_pieces` (engine/path.cpp).
// @port engine/path.cpp|devilution::path_solid_pieces(Point startPosition, Point destinationPosition) sha=c562cee6aad8
pub fn path_solid_pieces(ctx: &Ctx, start: Point, destination: Point) -> bool {
    match get_path_direction(start, destination) {
        5 => is_tile_not_solid(ctx, destination + Direction::SouthWest) && is_tile_not_solid(ctx, destination + Direction::SouthEast),
        6 => is_tile_not_solid(ctx, destination + Direction::SouthWest) && is_tile_not_solid(ctx, destination + Direction::NorthWest),
        7 => is_tile_not_solid(ctx, destination + Direction::NorthEast) && is_tile_not_solid(ctx, destination + Direction::NorthWest),
        8 => is_tile_not_solid(ctx, destination + Direction::SouthEast) && is_tile_not_solid(ctx, destination + Direction::NorthEast),
        _ => true,
    }
}

/// Original: `devilution::FindClosestValidPosition` (engine/path.cpp). Defaults: minimumRadius 0,
/// maximumRadius 18.
// @port engine/path.cpp|devilution::FindClosestValidPosition(tl::function_ref<bool(Point)> posOk, Point startingPosition, unsigned int minimumRadius, unsigned int maximumRadius) sha=83bcb1ade0a4
pub fn find_closest_valid_position(ctx: &Ctx, pos_ok: &dyn Fn(&Ctx, Point) -> bool, starting_position: Point, minimum_radius: u32, maximum_radius: u32) -> Option<Point> {
    crate::lighting::crawl_range(minimum_radius, maximum_radius, &mut |displacement| {
        let candidate = starting_position + displacement;
        if pos_ok(ctx, candidate) {
            Some(candidate)
        } else {
            None
        }
    })
}
