//! `Source/movie.cpp`: playing the game's SMK movies.

use crate::ctx::Ctx;
use crate::platform::events::Event;

/// Globals of movie.cpp.
#[derive(Default)]
pub struct MovieState {
    /// `movie_playing`: should the movie continue playing.
    pub movie_playing: bool,
    /// `loop_movie`: should the movie play in a loop.
    pub loop_movie: bool,
}

/// `movie_playing`
pub fn movie_playing(ctx: &Ctx) -> bool {
    ctx.movie.movie_playing
}

/// Original: `devilution::play_movie` (movie.cpp).
// @port movie.cpp|devilution::play_movie(const char *pszMovie, bool userCanClose) sha=4191acd576f7
pub fn play_movie(ctx: &mut Ctx, movie: &str, user_can_close: bool) {
    use crate::storm::storm_svid::{svid_play_begin, svid_play_continue, svid_play_end};
    if crate::engine::demomode::is_running(ctx) {
        return;
    }

    ctx.movie.movie_playing = true;

    crate::engine::sound::sound_disable_music(ctx, true);
    crate::effects::stream_stop(ctx);

    if crate::hwcursor::is_hardware_cursor_enabled(ctx) && ctx.controls.control_device == crate::controls::ControlTypes::KeyboardAndMouse {
        crate::hwcursor::set_hardware_cursor_visible(ctx, false);
    }

    let flags = if ctx.movie.loop_movie { 0x100C_0808 } else { 0x1028_0808 };
    if svid_play_begin(ctx, movie, flags) {
        while ctx.movie.movie_playing {
            while ctx.movie.movie_playing {
                let Some((event, _mod_state)) = crate::engine::events::fetch_message(ctx) else { break };
                let Some(event) = event else { continue };
                if user_can_close {
                    for ctrl_event in crate::controls::controller::to_controller_button_events(ctx, &event) {
                        if !crate::controls::game_controls::skips_movie(ctrl_event) {
                            continue;
                        }
                        ctx.movie.movie_playing = false;
                        break;
                    }
                }
                match event {
                    Event::KeyDown { key, .. } => {
                        if user_can_close || key == crate::platform::events::keys::SDLK_ESCAPE {
                            ctx.movie.movie_playing = false;
                        }
                    }
                    Event::MouseButtonUp { .. } => {
                        if user_can_close {
                            ctx.movie.movie_playing = false;
                        }
                    }
                    Event::FocusLost => crate::diablo::diablo_focus_pause(ctx),
                    Event::FocusGained => crate::diablo::diablo_focus_unpause(ctx),
                    Event::Quit => {
                        svid_play_end(ctx);
                        crate::diablo::diablo_quit(ctx, 0);
                    }
                    _ => {}
                }
            }
            if !svid_play_continue(ctx) {
                break;
            }
        }
        svid_play_end(ctx);
    }

    crate::engine::sound::sound_disable_music(ctx, false);

    ctx.movie.movie_playing = false;

    let ((x, y), _) = ctx.platform.mouse_state();
    ctx.diablo.mouse_position = (x, y);
    crate::engine::backbuffer_state::init_backbuffer_state(ctx);
}

/// Original: `devilution::PlayInGameMovie` (movie.cpp).
// @port movie.cpp|devilution::PlayInGameMovie(const char *pszMovie) sha=a276f1a0efec
pub fn play_in_game_movie(ctx: &mut Ctx, movie: &str) {
    crate::engine::palette::palette_fade_out(ctx, 8);
    play_movie(ctx, movie, false);
    crate::engine::render::scrollrt::clear_screen_buffer(ctx);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    crate::engine::render::scrollrt::scrollrt_draw_game_screen(ctx);
    crate::engine::palette::palette_fade_in(ctx, 8);
    crate::engine::backbuffer_state::redraw_everything(ctx);
}
