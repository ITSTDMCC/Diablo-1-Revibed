//! `Source/appfat.cpp`: fatal error dialogs.

use crate::ctx::Ctx;
use crate::utils::language::tr;

#[derive(Default)]
pub struct AppfatState {
    /// `Terminating`
    terminating: bool,
    /// `CleanupThreadId`
    cleanup_thread_id: Option<std::thread::ThreadId>,
}

/// Original: `FreeDlg` (appfat.cpp).
// @port appfat.cpp|devilution::FreeDlg() sha=e63c3e7eda0e
fn free_dlg(ctx: &mut Ctx) {
    let me = std::thread::current().id();
    if ctx.appfat.terminating && ctx.appfat.cleanup_thread_id != Some(me) {
        ctx.platform.delay(20000);
    }
    ctx.appfat.terminating = true;
    ctx.appfat.cleanup_thread_id = Some(me);
    if ctx.init.gb_is_multiplayer && crate::storm::storm_net::snet_leave_game(ctx, 3) {
        ctx.platform.delay(2000);
    }
    crate::storm::storm_net::snet_destroy(ctx);
}

/// Original: `devilution::app_fatal` (appfat.cpp).
// @port appfat.cpp|devilution::app_fatal(string_view str) sha=9a60c9a2623b
pub fn app_fatal(ctx: &mut Ctx, s: &str) -> ! {
    free_dlg(ctx);
    crate::diablo_ui::dialogs::ui_error_ok_dialog(ctx, &tr("Error"), s, true);
    crate::diablo::diablo_quit(ctx, 1)
}

/// Original: `devilution::ErrDlg` (appfat.cpp).
// @port appfat.cpp|devilution::ErrDlg(const char *title, string_view error, string_view logFilePath, int logLineNr) sha=6b1ea8a548a7
pub fn err_dlg(ctx: &mut Ctx, title: &str, error: &str, log_file_path: &str, log_line_nr: i32) -> ! {
    free_dlg(ctx);
    let text = tr("{:s}\n\nThe error occurred at: {:s} line {:d}")
        .replacen("{:s}", error, 1)
        .replacen("{:s}", log_file_path, 1)
        .replacen("{:d}", &log_line_nr.to_string(), 1);
    crate::diablo_ui::dialogs::ui_error_ok_dialog(ctx, title, &text, true);
    crate::diablo::diablo_quit(ctx, 1)
}

/// Original: `devilution::InsertCDDlg` (appfat.cpp).
// @port appfat.cpp|devilution::InsertCDDlg(string_view archiveName) sha=951576ae9abe
pub fn insert_cd_dlg(ctx: &mut Ctx, archive_name: &str) -> ! {
    let text = tr("Unable to open main data archive ({:s}).\n\nMake sure that it is in the game folder.").replacen("{:s}", archive_name, 1);
    crate::diablo_ui::dialogs::ui_error_ok_dialog(ctx, &tr("Data File Error"), &text, true);
    crate::diablo::diablo_quit(ctx, 1)
}

/// Original: `devilution::DirErrorDlg` (appfat.cpp).
// @port appfat.cpp|devilution::DirErrorDlg(string_view error) sha=0a7fb959b202
pub fn dir_error_dlg(ctx: &mut Ctx, error: &str) -> ! {
    let text = tr("Unable to write to location:\n{:s}").replacen("{:s}", error, 1);
    crate::diablo_ui::dialogs::ui_error_ok_dialog(ctx, &tr("Read-Only Directory Error"), &text, true);
    crate::diablo::diablo_quit(ctx, 1)
}
