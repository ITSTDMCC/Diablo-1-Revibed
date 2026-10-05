//! `Source/qol/itemlabels` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn highlight_key_pressed(ctx: &mut Ctx, pressed: bool), "qol/itemlabels.cpp|devilution::HighlightKeyPressed(bool pressed)");

crate::pending_fn!(pub fn toggle_item_label_highlight(ctx: &mut Ctx), "qol/itemlabels.cpp|devilution::ToggleItemLabelHighlight()");
