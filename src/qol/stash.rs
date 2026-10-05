//! `Source/qol/stash.cpp`: the shared stash.

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;

pub const CountStashPages: u32 = 100;
pub const LastStashPage: u32 = CountStashPages - 1;

/// `StashStruct::StashCell`
pub type StashCell = u16;
/// `StashStruct::StashGrid` (`[x][y]`)
pub type StashGrid = [[StashCell; 10]; 10];

/// `StashStruct`
#[derive(Clone, Default)]
pub struct StashStruct {
    /// `std::map<unsigned, StashGrid>` (ordered by page)
    pub stashGrids: std::collections::BTreeMap<u32, StashGrid>,
    pub stashList: Vec<crate::items::Item>,
    pub gold: i32,
    pub dirty: bool,
    page: u32,
}

impl StashStruct {
    pub const EmptyCell: StashCell = u16::MAX;

    pub fn GetPage(&self) -> u32 {
        self.page
    }

    /// `GetCurrentGrid` (creates the page like `std::map::operator[]`)
    pub fn get_current_grid(&mut self) -> &mut StashGrid {
        let p = self.page;
        self.stashGrids.entry(p).or_default()
    }

    /// Original: `StashStruct::SetPage` (qol/stash.cpp).
    // @port qol/stash.cpp|devilution::StashStruct::SetPage(unsigned newPage) sha=19d07dd41d48
    pub fn set_page(&mut self, new_page: u32) {
        self.page = new_page.min(LastStashPage);
        self.dirty = true;
    }

    /// Original: `StashStruct::NextPage` (qol/stash.cpp).
    // @port qol/stash.cpp|devilution::StashStruct::NextPage(unsigned offset) sha=0f754e5adeaa
    pub fn next_page(&mut self, offset: u32) {
        if self.page <= LastStashPage {
            self.page += offset.min(LastStashPage - self.page);
        } else {
            self.page = LastStashPage;
        }
        self.dirty = true;
    }

    /// Original: `StashStruct::PreviousPage` (qol/stash.cpp).
    // @port qol/stash.cpp|devilution::StashStruct::PreviousPage(unsigned offset) sha=74a66afb7faa
    pub fn previous_page(&mut self, offset: u32) {
        if self.page <= LastStashPage {
            self.page -= offset.min(self.page);
        } else {
            self.page = LastStashPage;
        }
        self.dirty = true;
    }

    /// Original: `StashStruct::RemoveStashItem` (qol/stash.cpp).
    // @port qol/stash.cpp|devilution::StashStruct::RemoveStashItem(StashStruct::StashCell iv) sha=ac534aa46049
    pub fn remove_stash_item(&mut self, iv: StashCell) {
        for row in self.get_current_grid().iter_mut() {
            for item_id in row.iter_mut() {
                if item_id.wrapping_sub(1) == iv {
                    *item_id = 0;
                }
            }
        }
        if self.stashList.is_empty() {
            return;
        }
        let last_item_index = (self.stashList.len() - 1) as StashCell;
        if last_item_index != iv {
            self.stashList[iv as usize] = self.stashList[last_item_index as usize].clone();
            for grid in self.stashGrids.values_mut() {
                for row in grid.iter_mut() {
                    for item_id in row.iter_mut() {
                        if *item_id == last_item_index.wrapping_add(1) {
                            *item_id = iv.wrapping_add(1);
                        }
                    }
                }
            }
        }
        self.stashList.pop();
        self.dirty = true;
    }
}

/// `Stash.SetPage(page)`
pub fn set_page(ctx: &mut Ctx, page: u32) {
    ctx.stash.Stash.set_page(page);
}

/// Original: `StashStruct::RefreshItemStatFlags` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::StashStruct::RefreshItemStatFlags() sha=4cfce2b411ff
pub fn refresh_item_stat_flags(ctx: &mut Ctx) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let mut list = std::mem::take(&mut ctx.stash.Stash.stashList);
    for item in list.iter_mut() {
        item.update_required_stats_cache_for_player(&ctx.players.Players[me]);
    }
    ctx.stash.Stash.stashList = list;
}

/// Globals of stash.cpp.
pub struct StashState {
    /// `Stash`
    pub Stash: StashStruct,
    /// `StashPanelArt`
    pub stash_panel_art: Option<ClxSpriteList>,
    /// `StashNavButtonArt`
    pub stash_nav_button_art: Option<ClxSpriteList>,
    /// `IsStashOpen`
    pub IsStashOpen: bool,
    /// `IsWithdrawGoldOpen`
    pub IsWithdrawGoldOpen: bool,
    /// `WithdrawGoldValue`
    pub WithdrawGoldValue: i32,
    /// `InitialWithdrawGoldValue`
    pub InitialWithdrawGoldValue: i32,
    /// `StashButtonPressed`
    pub StashButtonPressed: i32,
}

impl Default for StashState {
    fn default() -> Self {
        StashState {
            Stash: StashStruct::default(),
            stash_panel_art: None,
            stash_nav_button_art: None,
            IsStashOpen: false,
            IsWithdrawGoldOpen: false,
            WithdrawGoldValue: 0,
            InitialWithdrawGoldValue: 0,
            StashButtonPressed: -1,
        }
    }
}

/// Original: `devilution::FreeStashGFX` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::FreeStashGFX() sha=5a2700c004bb
pub fn free_stash_gfx(ctx: &mut Ctx) {
    ctx.stash.stash_nav_button_art = None;
    ctx.stash.stash_panel_art = None;
}



use crate::engine::geometry::{Displacement, Point, Rectangle, Size};
use crate::engine::render::text_render::{draw_string, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::enums::*;
use crate::inv::{InventorySlotSizeInPixels, INV_SLOT_HALF_SIZE_PX, INV_SLOT_SIZE_PX};
use crate::tables::items_tables::{ItemCAnimTbl, ItemInvSnds};

/// `ButtonSize`
const BUTTON_SIZE: Size = Size::new(27, 16);
/// `StashButtonRect`: the 2 navigation buttons, withdraw gold, 2 navigation buttons.
const STASH_BUTTON_RECT: [Rectangle; 5] = [
    Rectangle::new(Point::new(19, 19), BUTTON_SIZE),  // 10 left
    Rectangle::new(Point::new(56, 19), BUTTON_SIZE),  // 1 left
    Rectangle::new(Point::new(93, 19), BUTTON_SIZE),  // withdraw gold
    Rectangle::new(Point::new(242, 19), BUTTON_SIZE), // 1 right
    Rectangle::new(Point::new(279, 19), BUTTON_SIZE), // 10 right
];
/// `StashGridSize`
const STASH_GRID_SIZE: Size = Size::new(10, 10);
/// `InvalidStashPoint`
const INVALID_STASH_POINT: Point = Point::new(-1, -1);

/// `StashGridRange` (row-major, as `PointsInRectangleRange`)
fn stash_grid_range() -> impl Iterator<Item = Point> {
    crate::engine::geometry::points_in_rectangle(Rectangle::new(Point::new(0, 0), STASH_GRID_SIZE))
}

impl StashStruct {
    /// `GetItemIdAtPosition`: adds a blank grid if it doesn't exist.
    pub fn get_item_id_at_position(&mut self, grid_position: Point) -> StashCell {
        let cell = self.get_current_grid()[grid_position.x as usize][grid_position.y as usize];
        cell.wrapping_sub(1)
    }
}

fn play_item_sfx(ctx: &mut Ctx, curs: u8) {
    crate::effects::play_sfx(ctx, ItemInvSnds[ItemCAnimTbl[curs as usize] as usize]);
}

fn my_player(ctx: &Ctx) -> usize {
    ctx.players.MyPlayer.expect("MyPlayer")
}

/// Original: `AddItemToStashGrid` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::AddItemToStashGrid(unsigned page, Point position, uint16_t stashListIndex, Size itemSize) sha=411888145f18
fn add_item_to_stash_grid(ctx: &mut Ctx, page: u32, position: Point, stash_list_index: u16, item_size: Size) {
    let grid = ctx.stash.Stash.stashGrids.entry(page).or_default();
    for point in crate::engine::geometry::points_in_rectangle(Rectangle::new(position, item_size)) {
        grid[point.x as usize][point.y as usize] = stash_list_index + 1;
    }
}

/// Original: `FindTargetSlotUnderItemCursor` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::FindTargetSlotUnderItemCursor(Point cursorPosition, Size itemSize) sha=04a72052d876
fn find_target_slot_under_item_cursor(ctx: &Ctx, cursor_position: Point, item_size: Size) -> Option<Point> {
    for mut point in stash_grid_range() {
        let cell = Rectangle::new(get_stash_slot_coord(ctx, point), Size::new(InventorySlotSizeInPixels.width + 1, InventorySlotSizeInPixels.height + 1));
        if cell.contains(cursor_position) {
            if item_size.height <= 1 && item_size.width <= 1 {
                return Some(point);
            }
            let mut hot = Displacement::new((item_size.width - 1) / 2, (item_size.height - 1) / 2);
            if item_size.width % 2 == 0 && cell.contains(cursor_position + Displacement::new(INV_SLOT_HALF_SIZE_PX, 0)) {
                hot.delta_x += 1;
            }
            if item_size.height % 2 == 0 && cell.contains(cursor_position + Displacement::new(0, INV_SLOT_HALF_SIZE_PX)) {
                hot.delta_y += 1;
            }
            point.y = (point.y - hot.delta_y).clamp(0, STASH_GRID_SIZE.height - item_size.height);
            point.x = (point.x - hot.delta_x).clamp(0, STASH_GRID_SIZE.width - item_size.width);
            return Some(point);
        }
    }
    None
}

/// Original: `IsItemAllowedInStash` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::IsItemAllowedInStash(const Item &item) sha=786aae108893
fn is_item_allowed_in_stash(item: &crate::items::Item) -> bool {
    item._iMiscId != IMISC_ARENAPOT
}

/// Original: `CheckStashPaste` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::CheckStashPaste(Point cursorPosition) sha=8e1d1a7a1d36
fn check_stash_paste(ctx: &mut Ctx, cursor_position: Point) {
    let me = my_player(ctx);
    if !is_item_allowed_in_stash(&ctx.players.Players[me].HoldItem) {
        return;
    }
    if ctx.players.Players[me].HoldItem._itype == ItemType::Gold {
        let v = ctx.players.Players[me].HoldItem._ivalue;
        if ctx.stash.Stash.gold > i32::MAX - v {
            return;
        }
        ctx.stash.Stash.gold += v;
        ctx.players.Players[me].HoldItem.clear();
        crate::effects::play_sfx(ctx, crate::effects_data::IS_GOLD);
        ctx.stash.Stash.dirty = true;
        crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_HAND);
        return;
    }
    let item_size = crate::inv::get_inventory_size(&ctx.players.Players[me].HoldItem);
    let Some(first_slot) = find_target_slot_under_item_cursor(ctx, cursor_position, item_size) else { return };
    // Check that no more than 1 item is replaced by the move
    let mut stash_index = StashStruct::EmptyCell;
    for point in crate::engine::geometry::points_in_rectangle(Rectangle::new(first_slot, item_size)) {
        let iv = ctx.stash.Stash.get_item_id_at_position(point);
        if iv == StashStruct::EmptyCell || stash_index == iv {
            continue;
        }
        if stash_index == StashStruct::EmptyCell {
            stash_index = iv; // Found first item
            continue;
        }
        return; // Found a second item
    }
    let curs = ctx.players.Players[me].HoldItem._iCurs;
    play_item_sfx(ctx, curs);
    // Need to set the item anchor position to the bottom left so drawing code functions correctly.
    ctx.players.Players[me].HoldItem.position = first_slot + Displacement::new(0, item_size.height - 1);
    if stash_index == StashStruct::EmptyCell {
        let item = ctx.players.Players[me].HoldItem.pop();
        ctx.stash.Stash.stashList.push(item);
        // stashList will have at most 10 000 items, up to 65 535 are supported with uint16_t indexes
        stash_index = (ctx.stash.Stash.stashList.len() - 1) as u16;
    } else {
        // swap the held item and whatever was in the stash at this position
        std::mem::swap(&mut ctx.stash.Stash.stashList[stash_index as usize], &mut ctx.players.Players[me].HoldItem);
        // then clear the space occupied by the old item
        for row in ctx.stash.Stash.get_current_grid().iter_mut() {
            for item_id in row.iter_mut() {
                if item_id.wrapping_sub(1) == stash_index {
                    *item_id = 0;
                }
            }
        }
    }
    // Finally mark the area now occupied by the pasted item in the current page/grid.
    let page = ctx.stash.Stash.GetPage();
    add_item_to_stash_grid(ctx, page, first_slot, stash_index, item_size);
    ctx.stash.Stash.dirty = true;
    let hold = &ctx.players.Players[me].HoldItem;
    let (e, c) = (hold.is_empty(), hold._iCurs);
    crate::cursor::new_cursor_item(ctx, e, c);
}

/// Original: `CheckStashCut` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::CheckStashCut(Point cursorPosition, bool automaticMove) sha=3c215a1bb7d0
fn check_stash_cut(ctx: &mut Ctx, cursor_position: Point, automatic_move: bool) {
    let me = my_player(ctx);
    if ctx.stash.IsWithdrawGoldOpen {
        ctx.stash.IsWithdrawGoldOpen = false;
        ctx.stash.WithdrawGoldValue = 0;
    }
    let mut slot = INVALID_STASH_POINT;
    for point in stash_grid_range() {
        let cell = Rectangle::new(get_stash_slot_coord(ctx, point), Size::new(InventorySlotSizeInPixels.width + 1, InventorySlotSizeInPixels.height + 1));
        // check which inventory rectangle the mouse is in, if any
        if cell.contains(cursor_position) {
            slot = point;
            break;
        }
    }
    if slot == INVALID_STASH_POINT {
        return;
    }
    ctx.players.Players[me].HoldItem.clear();
    let mut automatically_moved = false;
    let mut automatically_equipped = false;
    let iv = ctx.stash.Stash.get_item_id_at_position(slot);
    if iv != StashStruct::EmptyCell {
        let hold = ctx.stash.Stash.stashList[iv as usize].clone();
        ctx.players.Players[me].HoldItem = hold.clone();
        if automatic_move {
            if crate::inv::can_be_placed_on_belt(ctx, &hold) {
                automatically_moved = crate::inv::auto_place_item_in_belt(ctx, me, &hold, true);
            } else {
                automatically_moved = crate::inv::auto_equip(ctx, me, &hold, true);
                automatically_equipped = automatically_moved;
            }
        }
        if !automatic_move || automatically_moved {
            ctx.stash.Stash.remove_stash_item(iv);
        }
    }
    if !ctx.players.Players[me].HoldItem.is_empty() {
        crate::items::calc_plr_inv(ctx, me, true);
        {
            let p = &mut ctx.players.Players[me];
            p.HoldItem._iStatFlag = p.can_use_item(&p.HoldItem);
        }
        let hold = ctx.players.Players[me].HoldItem.clone();
        if automatically_equipped {
            play_item_sfx(ctx, hold._iCurs);
        } else if !automatic_move || automatically_moved {
            crate::effects::play_sfx(ctx, crate::effects_data::IS_IGRAB);
        }
        if automatic_move {
            if !automatically_moved {
                if crate::inv::can_be_placed_on_belt(ctx, &hold) {
                    crate::player::player_say_specific(ctx, me, HeroSpeech::IHaveNoRoom);
                } else {
                    crate::player::player_say_specific(ctx, me, HeroSpeech::ICantDoThat);
                }
            }
            ctx.players.Players[me].HoldItem.clear();
        } else {
            crate::cursor::new_cursor_item(ctx, hold.is_empty(), hold._iCurs);
        }
    }
}

/// Original: `WithdrawGold` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::WithdrawGold(Player &player, int amount) sha=44d385899752
fn withdraw_gold(ctx: &mut Ctx, pnum: usize, amount: i32) {
    crate::inv::add_gold_to_inventory(ctx, pnum, amount);
    ctx.stash.Stash.gold -= amount;
    ctx.stash.Stash.dirty = true;
}

/// Original: `devilution::GetStashSlotCoord` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::GetStashSlotCoord(Point slot) sha=da7ca72f1eb1
pub fn get_stash_slot_coord(ctx: &Ctx, slot: Point) -> Point {
    const STASH_NEXT_CELL: i32 = INV_SLOT_SIZE_PX + 1; // spacing between each cell
    crate::control::get_panel_position(ctx, UiPanels::Stash, Point::new(slot.x * STASH_NEXT_CELL + 17, slot.y * STASH_NEXT_CELL + 48))
}

/// Original: `devilution::InitStash` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::InitStash() sha=4dfab84b4491
pub fn init_stash(ctx: &mut Ctx) {
    ctx.stash.InitialWithdrawGoldValue = 0;
    if !ctx.diablo.headless_mode {
        ctx.stash.stash_panel_art = Some(crate::engine::load_sprites::load_clx(ctx, "data\\stash.clx"));
        ctx.stash.stash_nav_button_art = Some(crate::engine::load_sprites::load_clx(ctx, "data\\stashnavbtns.clx"));
    }
}

/// Original: `devilution::TransferItemToInventory` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::TransferItemToInventory(Player &player, uint16_t itemId) sha=a12d68ff5101
pub fn transfer_item_to_inventory(ctx: &mut Ctx, pnum: usize, item_id: u16) {
    if item_id == StashStruct::EmptyCell {
        return;
    }
    let item = ctx.stash.Stash.stashList[item_id as usize].clone();
    if item.is_empty() {
        return;
    }
    if !crate::inv::auto_place_item_in_inventory(ctx, pnum, &item, true) {
        crate::player::player_say_specific(ctx, pnum, HeroSpeech::IHaveNoRoom);
        return;
    }
    play_item_sfx(ctx, item._iCurs);
    ctx.stash.Stash.remove_stash_item(item_id);
}

fn stash_button(ctx: &Ctx, i: usize) -> Rectangle {
    let mut r = STASH_BUTTON_RECT[i];
    r.position = crate::control::get_panel_position(ctx, UiPanels::Stash, r.position);
    r
}

/// Original: `devilution::CheckStashButtonRelease` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::CheckStashButtonRelease(Point mousePosition) sha=366ed3a4d7cf
pub fn check_stash_button_release(ctx: &mut Ctx, mouse_position: Point) {
    if ctx.stash.StashButtonPressed == -1 {
        return;
    }
    let b = ctx.stash.StashButtonPressed as usize;
    if stash_button(ctx, b).contains(mouse_position) {
        match b {
            0 => ctx.stash.Stash.previous_page(10),
            1 => ctx.stash.Stash.previous_page(1),
            2 => start_gold_withdraw(ctx),
            3 => ctx.stash.Stash.next_page(1),
            4 => ctx.stash.Stash.next_page(10),
            _ => {}
        }
    }
    ctx.stash.StashButtonPressed = -1;
}

/// Original: `devilution::CheckStashButtonPress` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::CheckStashButtonPress(Point mousePosition) sha=4f7fd61ac533
pub fn check_stash_button_press(ctx: &mut Ctx, mouse_position: Point) {
    for i in 0..5 {
        if stash_button(ctx, i).contains(mouse_position) {
            ctx.stash.StashButtonPressed = i as i32;
            return;
        }
    }
    ctx.stash.StashButtonPressed = -1;
}

/// Original: `devilution::DrawStash` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::DrawStash(const Surface &out) sha=ff08d8903be6
pub fn draw_stash(ctx: &mut Ctx, out: &Surface) {
    let panel = crate::control::get_panel_position(ctx, UiPanels::Stash, Point::new(0, 0));
    let art = ctx.stash.stash_panel_art.as_ref().expect("StashPanelArt").get(0);
    crate::engine::render::text_render::render_clx_sprite(out, &art, (panel.x, panel.y));
    if ctx.stash.StashButtonPressed != -1 {
        let b = ctx.stash.StashButtonPressed as usize;
        let pos = crate::control::get_panel_position(ctx, UiPanels::Stash, STASH_BUTTON_RECT[b].position);
        let sprite = ctx.stash.stash_nav_button_art.as_ref().expect("StashNavButtonArt").get(b);
        crate::engine::render::text_render::render_clx_sprite(out, &sprite, (pos.x, pos.y));
    }
    let offset = Displacement::new(0, INV_SLOT_SIZE_PX - 1);
    for slot in stash_grid_range() {
        let item_id = ctx.stash.Stash.get_item_id_at_position(slot);
        if item_id == StashStruct::EmptyCell {
            continue; // No item in the given slot
        }
        let quality = ctx.stash.Stash.stashList[item_id as usize]._iMagical;
        let pos = get_stash_slot_coord(ctx, slot) + offset;
        crate::inv::inv_draw_slot_back(ctx, out, pos, InventorySlotSizeInPixels, quality);
    }
    for slot in stash_grid_range() {
        let item_id = ctx.stash.Stash.get_item_id_at_position(slot);
        if item_id == StashStruct::EmptyCell {
            continue; // No item in the given slot
        }
        let item = ctx.stash.Stash.stashList[item_id as usize].clone();
        if item.position != slot {
            continue; // Not the first slot of the item
        }
        let frame = item._iCurs as i32 + crate::cursor::CURSOR_FIRSTITEM;
        let position = get_stash_slot_coord(ctx, item.position) + offset;
        let sprite = crate::cursor::get_inv_item_sprite(ctx, frame);
        if ctx.cursor.pcursstashitem == item_id {
            let color = crate::items::get_outline_color(&item, true);
            crate::engine::render::clx_render::clx_draw_outline(out, color, (position.x, position.y), &sprite);
        }
        crate::cursor::draw_item(ctx, &item, out, position, &sprite);
    }
    let position = crate::control::get_panel_position(ctx, UiPanels::Stash, Point::new(0, 0));
    let style = UiFlags::VERTICAL_CENTER | UiFlags::COLOR_WHITE;
    let page = (ctx.stash.Stash.GetPage() + 1).to_string();
    draw_string(ctx, out, &page, Rect::new(position.x + 132, position.y, 57, 11), UiFlags::ALIGN_CENTER | style, 1, -1);
    let gold = crate::utils::format_int::format_integer(ctx.stash.Stash.gold);
    draw_string(ctx, out, &gold, Rect::new(position.x + 122, position.y + 19, 107, 13), UiFlags::ALIGN_RIGHT | style, 1, -1);
}

/// Original: `devilution::CheckStashItem` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::CheckStashItem(Point mousePosition, bool isShiftHeld, bool isCtrlHeld) sha=09fffb86e2a1
pub fn check_stash_item(ctx: &mut Ctx, mouse_position: Point, is_shift_held: bool, is_ctrl_held: bool) {
    let me = my_player(ctx);
    if !ctx.players.Players[me].HoldItem.is_empty() {
        check_stash_paste(ctx, mouse_position);
    } else if is_ctrl_held {
        let c = ctx.cursor.pcursstashitem;
        transfer_item_to_inventory(ctx, me, c);
    } else {
        check_stash_cut(ctx, mouse_position, is_shift_held);
    }
}

/// Original: `devilution::CheckStashHLight` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::CheckStashHLight(Point mousePosition) sha=7836500a796d
pub fn check_stash_h_light(ctx: &mut Ctx, mouse_position: Point) -> u16 {
    let mut slot = INVALID_STASH_POINT;
    for point in stash_grid_range() {
        let cell = Rectangle::new(get_stash_slot_coord(ctx, point), Size::new(InventorySlotSizeInPixels.width + 1, InventorySlotSizeInPixels.height + 1));
        if cell.contains(mouse_position) {
            slot = point;
            break;
        }
    }
    if slot == INVALID_STASH_POINT {
        return u16::MAX;
    }
    ctx.control.info_color = UiFlags::COLOR_WHITE;
    let item_id = ctx.stash.Stash.get_item_id_at_position(slot);
    if item_id == StashStruct::EmptyCell {
        return u16::MAX;
    }
    let item = ctx.stash.Stash.stashList[item_id as usize].clone();
    if item.is_empty() {
        return u16::MAX;
    }
    ctx.control.info_color = item.get_text_color();
    ctx.control.info_string = item.get_name(ctx);
    if item._iIdentified {
        crate::items::print_item_details(ctx, &item);
    } else {
        crate::items::print_item_dur(ctx, &item);
    }
    item_id
}

/// Original: `devilution::UseStashItem` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::UseStashItem(uint16_t c) sha=057732de072b
pub fn use_stash_item(ctx: &mut Ctx, c: u16) -> bool {
    let me = my_player(ctx);
    {
        let p = &ctx.players.Players[me];
        if p._pInvincible && p._pHitPoints == 0 {
            return true;
        }
    }
    if ctx.cursor.pcurs != crate::cursor::CURSOR_HAND {
        return true;
    }
    if !crate::stores::stextflag_is_none(ctx) {
        return true;
    }
    let item = ctx.stash.Stash.stashList[c as usize].clone();
    const SPEECH_DELAY: i32 = 10;
    if item.IDidx == IDI_MUSHROOM {
        crate::player::player_say_delayed(ctx, me, HeroSpeech::NowThatsOneBigMushroom, SPEECH_DELAY);
        return true;
    }
    if item.IDidx == IDI_FUNGALTM {
        crate::effects::play_sfx(ctx, crate::effects_data::IS_IBOOK);
        crate::player::player_say_delayed(ctx, me, HeroSpeech::ThatDidntDoAnything, SPEECH_DELAY);
        return true;
    }
    if !item.is_usable(ctx) {
        return false;
    }
    if !ctx.players.Players[me].can_use_item(&item) {
        crate::player::player_say(ctx, me, HeroSpeech::ICantUseThisYet);
        return true;
    }
    if ctx.stash.IsWithdrawGoldOpen {
        ctx.stash.IsWithdrawGoldOpen = false;
        ctx.stash.WithdrawGoldValue = 0;
    }
    if item.is_scroll() {
        return true;
    }
    if item._iMiscId > IMISC_RUNEFIRST && item._iMiscId < IMISC_RUNELAST && ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town {
        return true;
    }
    if item._iMiscId == IMISC_BOOK {
        crate::effects::play_sfx(ctx, crate::effects_data::IS_RBOOK);
    } else {
        play_item_sfx(ctx, item._iCurs);
    }
    let my_id = ctx.players.MyPlayerId;
    crate::items::use_item(ctx, my_id, item._iMiscId, item._iSpell, -1);
    if ctx.stash.Stash.stashList[c as usize]._iMiscId == IMISC_MAPOFDOOM {
        return true;
    }
    if ctx.stash.Stash.stashList[c as usize]._iMiscId == IMISC_NOTE {
        crate::minitext::init_q_text_msg(ctx, TEXT_BOOK9);
        crate::inv::close_inventory(ctx);
        return true;
    }
    ctx.stash.Stash.remove_stash_item(c);
    true
}

/// Original: `devilution::StartGoldWithdraw` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::StartGoldWithdraw() sha=7fbd383a329a
pub fn start_gold_withdraw(ctx: &mut Ctx) {
    crate::control::close_gold_drop(ctx);
    ctx.stash.InitialWithdrawGoldValue = crate::inv::room_for_gold(ctx).min(ctx.stash.Stash.gold);
    if ctx.control.talkflag {
        crate::control::control_reset_talk(ctx);
    }
    let start = crate::control::get_panel_position(ctx, UiPanels::Stash, Point::new(67, 128));
    ctx.platform.text_input_rect = (start.x, start.y, 180, 20);
    ctx.stash.IsWithdrawGoldOpen = true;
    ctx.stash.WithdrawGoldValue = 0;
    ctx.platform.start_text_input();
}

/// Original: `devilution::WithdrawGoldKeyPress` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::WithdrawGoldKeyPress(SDL_Keycode vkey) sha=0da22da13f44
pub fn withdraw_gold_key_press(ctx: &mut Ctx, vkey: i32) {
    use crate::platform::events::keys::*;
    let me = my_player(ctx);
    if ctx.players.Players[me]._pHitPoints >> 6 <= 0 {
        close_gold_withdraw(ctx);
        return;
    }
    if vkey == SDLK_RETURN || vkey == SDLK_KP_ENTER {
        if ctx.stash.WithdrawGoldValue > 0 {
            let v = ctx.stash.WithdrawGoldValue;
            withdraw_gold(ctx, me, v);
            crate::effects::play_sfx(ctx, crate::effects_data::IS_GOLD);
        }
        close_gold_withdraw(ctx);
    } else if vkey == SDLK_ESCAPE {
        close_gold_withdraw(ctx);
    } else if vkey == SDLK_BACKSPACE {
        ctx.stash.WithdrawGoldValue /= 10;
    }
}

/// Original: `devilution::DrawGoldWithdraw` (qol/stash.cpp). The amount is `WithdrawGoldValue`.
// @port qol/stash.cpp|devilution::DrawGoldWithdraw(const Surface &out, int amount) sha=4cee55796d98
pub fn draw_gold_withdraw(ctx: &mut Ctx, out: &Surface) {
    if !ctx.stash.IsWithdrawGoldOpen {
        return;
    }
    let amount = ctx.stash.WithdrawGoldValue;
    const DIALOG_X: i32 = 30;
    let pos = crate::control::get_panel_position(ctx, UiPanels::Stash, Point::new(DIALOG_X, 178));
    let sprite = crate::control::p_g_box_buff(ctx).get(0);
    crate::engine::render::clx_render::clx_draw(out, (pos.x, pos.y), &sprite);
    // Pre-wrap the string at spaces, otherwise DrawString would hard wrap in the middle of words
    let text = crate::utils::language::tr("How many gold pieces do you want to withdraw?");
    let wrapped = crate::engine::render::text_render::word_wrap_string(ctx, &text, 200, crate::engine::render::text_render::GameFontTables::GameFont12, 1);
    let p = crate::control::get_panel_position(ctx, UiPanels::Stash, Point::new(DIALOG_X + 31, 75));
    draw_string(ctx, out, &wrapped, Rect::new(p.x, p.y, 200, 50), UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, 1, 17);
    let value = if amount > 0 { amount.to_string() } else { String::new() };
    let p = crate::control::get_panel_position(ctx, UiPanels::Stash, Point::new(DIALOG_X + 37, 128));
    draw_string(ctx, out, &value, Rect::new(p.x, p.y, 0, 0), UiFlags::COLOR_WHITE | UiFlags::PENTA_CURSOR, 1, -1);
}

/// Original: `devilution::CloseGoldWithdraw` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::CloseGoldWithdraw() sha=e1748f3d6692
pub fn close_gold_withdraw(ctx: &mut Ctx) {
    if !ctx.stash.IsWithdrawGoldOpen {
        return;
    }
    ctx.stash.IsWithdrawGoldOpen = false;
    ctx.stash.WithdrawGoldValue = 0;
    ctx.platform.stop_text_input();
}

/// Original: `devilution::GoldWithdrawNewText` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::GoldWithdrawNewText(string_view text) sha=01cc805cb1a3
pub fn gold_withdraw_new_text(ctx: &mut Ctx, text: &str) {
    for vkey in text.bytes() {
        let digit = vkey as i8 as i32 - b'0' as i32;
        if (0..=9).contains(&digit) {
            let mut new_gold_value = ctx.stash.WithdrawGoldValue * 10;
            new_gold_value += digit;
            if new_gold_value <= ctx.stash.InitialWithdrawGoldValue {
                ctx.stash.WithdrawGoldValue = new_gold_value;
            }
        }
    }
}

/// Original: `devilution::AutoPlaceItemInStash` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::AutoPlaceItemInStash(Player &player, const Item &item, bool persistItem) sha=4f129f2ba930
pub fn auto_place_item_in_stash(ctx: &mut Ctx, _pnum: usize, item: &crate::items::Item, persist_item: bool) -> bool {
    if !is_item_allowed_in_stash(item) {
        return false;
    }
    if item._itype == ItemType::Gold {
        if ctx.stash.Stash.gold > i32::MAX - item._ivalue {
            return false;
        }
        if persist_item {
            ctx.stash.Stash.gold += item._ivalue;
            ctx.stash.Stash.dirty = true;
        }
        return true;
    }
    let item_size = crate::inv::get_inventory_size(item);
    // Try to add the item to the current active page and if it's not possible move forward
    for page_counter in 0..CountStashPages {
        let mut page_index = ctx.stash.Stash.GetPage() + page_counter;
        // Wrap around if needed
        if page_index >= CountStashPages {
            page_index -= CountStashPages;
        }
        // Search all possible position in stash grid
        let area = Rectangle::new(Point::new(0, 0), Size::new(10 - (item_size.width - 1), 10 - (item_size.height - 1)));
        for stash_position in crate::engine::geometry::points_in_rectangle(area) {
            // Check that all needed slots are free
            let grid = ctx.stash.Stash.stashGrids.entry(page_index).or_default();
            let is_space_free = crate::engine::geometry::points_in_rectangle(Rectangle::new(stash_position, item_size))
                .all(|item_point| grid[item_point.x as usize][item_point.y as usize] == 0);
            if !is_space_free {
                continue;
            }
            if persist_item {
                ctx.stash.Stash.stashList.push(item.clone());
                let stash_index = (ctx.stash.Stash.stashList.len() - 1) as u16;
                ctx.stash.Stash.stashList[stash_index as usize].position = stash_position + Displacement::new(0, item_size.height - 1);
                add_item_to_stash_grid(ctx, page_index, stash_position, stash_index, item_size);
                ctx.stash.Stash.dirty = true;
            }
            return true;
        }
    }
    false
}
