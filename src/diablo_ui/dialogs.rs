//! `Source/DiabloUI/dialogs.cpp`: OK/error dialogs.

use crate::ctx::Ctx;
use crate::platform::log;

#[derive(Default)]
pub struct DialogsState {
    /// `inDialog` (static in UiOkDialog)
    in_dialog: bool,
}

/// Original: `UiOkDialog` (DiabloUI/dialogs.cpp). Before the game window is active (or while
/// another dialog is open) the message goes to a system message box, as in the original; the
/// in-game dialog path is pending (`dialog_in_game`).
// @port DiabloUI/dialogs.cpp|devilution::UiOkDialog(string_view caption, string_view text, bool error, const std::vector<std::unique_ptr<UiItemBase>> &renderBehind) sha=d809f0e6c0a3
fn ui_ok_dialog(ctx: &mut Ctx, caption: &str, text: &str, error: bool) {
    if !caption.is_empty() {
        log::error!("{}\n{}", caption, text);
    } else {
        log::error!("{}", text);
    }
    if !ctx.init.gb_active || ctx.dialogs.in_dialog {
        if !ctx.diablo.headless_mode {
            ctx.platform.show_cursor(true);
            crate::platform::win32::show_error_message_box(caption, text);
        }
        return;
    }
    if crate::hwcursor::is_hardware_cursor(ctx) {
        ctx.platform.show_cursor(true);
    }
    ctx.dialogs.in_dialog = true;
    dialog_in_game(ctx, caption, text, error);
    ctx.dialogs.in_dialog = false;
}

crate::pending_fn!(fn dialog_in_game(ctx: &mut Ctx, caption: &str, text: &str, error: bool), "DiabloUI/dialogs.cpp|devilution::Init(string_view caption, string_view text, bool error, bool renderBehind) + DialogLoop");

/// Original: `UiErrorOkDialog(string_view caption, string_view text, bool error)` (DiabloUI/dialogs.cpp).
// @port DiabloUI/dialogs.cpp|devilution::UiErrorOkDialog(string_view caption, string_view text, bool error) sha=5ece290ebb0e
pub fn ui_error_ok_dialog(ctx: &mut Ctx, caption: &str, text: &str, error: bool) {
    ui_ok_dialog(ctx, caption, text, error);
}
