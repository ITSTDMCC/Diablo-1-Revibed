//! `Source/qol/stash.cpp`: the shared stash (only its graphics so far).

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
#[derive(Default)]
pub struct StashState {
    /// `Stash`
    pub Stash: StashStruct,
    /// `StashPanelArt`
    pub stash_panel_art: Option<ClxSpriteList>,
    /// `StashNavButtonArt`
    pub stash_nav_button_art: Option<ClxSpriteList>,
    /// `IsStashOpen`
    pub IsStashOpen: bool,
}

/// Original: `devilution::FreeStashGFX` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::FreeStashGFX() sha=5a2700c004bb
pub fn free_stash_gfx(ctx: &mut Ctx) {
    ctx.stash.stash_nav_button_art = None;
    ctx.stash.stash_panel_art = None;
}



crate::pending_fn!(pub fn close_gold_withdraw(ctx: &mut Ctx), "qol/stash.cpp|devilution::CloseGoldWithdraw()");
crate::pending_fn!(pub fn auto_place_item_in_stash(ctx: &mut Ctx, pnum: usize, item: &crate::items::Item, persist_item: bool) -> bool, "qol/stash.cpp|devilution::AutoPlaceItemInStash(Player &player, const Item &item, bool persistItem)");
