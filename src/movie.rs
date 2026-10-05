//! `Source/movie` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn play_movie(ctx: &mut Ctx, movie: &str, user_can_close: bool), "movie.cpp|devilution::play_movie(const char *pszMovie, bool userCanClose)");

/// `movie_playing`: no movie plays until play_movie is ported.
pub fn movie_playing(_ctx: &Ctx) -> bool {
    false
}

crate::pending_fn!(pub fn play_in_game_movie(ctx: &mut Ctx, movie: &str), "movie.cpp|devilution::PlayInGameMovie(const char *pszMovie)");

/// Globals of movie.cpp.
#[derive(Default)]
pub struct MovieState {
    /// `loop_movie`
    pub loop_movie: bool,
}
