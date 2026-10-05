//! `Source/DiabloUI/progress.cpp`: the progress dialog used while waiting for level data (pending).

use crate::ctx::Ctx;

crate::pending_fn!(pub fn ui_progress_dialog(ctx: &mut Ctx, f: fn(&mut Ctx) -> i32) -> bool, "DiabloUI/progress.cpp|devilution::UiProgressDialog(int (*fnfunc)())");
