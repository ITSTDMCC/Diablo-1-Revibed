//! `Source/engine/backbuffer_state.cpp`: what needs redrawing, per back buffer.

use crate::ctx::Ctx;
use crate::engine::surface::Rect;

/// `PanelDrawComponent`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelDrawComponent {
    Health,
    Mana,
    ControlButtons,
    Belt,
}
const NUM_COMPONENTS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Redraw {
    None,
    ViewportOnly,
    All,
}

/// `DrawnCursor`
#[derive(Clone)]
pub struct DrawnCursor {
    pub rect: Rect,
    pub behind_buffer: Box<[u8; 8192]>,
}

#[derive(Clone)]
struct BackbufferState {
    redraw: Redraw,
    redraw_components: [bool; NUM_COMPONENTS],
    cursor: DrawnCursor,
}

/// `States`: keyed by the back buffer's pixel pointer, as in the original.
#[derive(Default)]
pub struct BackbufferStates {
    states: Vec<(usize, BackbufferState)>,
}

fn current_ptr(ctx: &Ctx) -> usize {
    ctx.dx.pal_surface.as_ref().map(|s| s.pixels.as_ptr() as usize).unwrap_or(0)
}

/// Original: `GetBackbufferState` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::GetBackbufferState() sha=3584cf5fbd07
fn get_backbuffer_state(ctx: &mut Ctx) -> &mut BackbufferState {
    let ptr = current_ptr(ctx);
    let states = &mut ctx.dx.backbuffer.states;
    if let Some(i) = states.iter().position(|(p, _)| *p == ptr) {
        return &mut states[i].1;
    }
    states.push((
        ptr,
        BackbufferState {
            redraw: Redraw::All,
            redraw_components: [false; NUM_COMPONENTS],
            cursor: DrawnCursor { rect: Rect::default(), behind_buffer: Box::new([0; 8192]) },
        },
    ));
    &mut states.last_mut().unwrap().1
}

/// Original: `devilution::IsRedrawEverything` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::IsRedrawEverything() sha=14903ebc9a23
pub fn is_redraw_everything(ctx: &mut Ctx) -> bool {
    get_backbuffer_state(ctx).redraw == Redraw::All
}

/// Original: `devilution::RedrawViewport` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::RedrawViewport() sha=0991944e1b35
pub fn redraw_viewport(ctx: &mut Ctx) {
    for (_, s) in ctx.dx.backbuffer.states.iter_mut() {
        if s.redraw != Redraw::All {
            s.redraw = Redraw::ViewportOnly;
        }
    }
}

/// Original: `devilution::IsRedrawViewport` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::IsRedrawViewport() sha=db110ef2ee18
pub fn is_redraw_viewport(ctx: &mut Ctx) -> bool {
    get_backbuffer_state(ctx).redraw == Redraw::ViewportOnly
}

/// Original: `devilution::RedrawComplete` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::RedrawComplete() sha=9af6b82b4147
pub fn redraw_complete(ctx: &mut Ctx) {
    get_backbuffer_state(ctx).redraw = Redraw::None;
}

/// Original: `devilution::RedrawEverything` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::RedrawEverything() sha=5ce66b6cb03e
pub fn redraw_everything(ctx: &mut Ctx) {
    for (_, s) in ctx.dx.backbuffer.states.iter_mut() {
        s.redraw = Redraw::All;
    }
}

/// Original: `devilution::InitBackbufferState` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::InitBackbufferState() sha=d78cf9ae2bc6
pub fn init_backbuffer_state(ctx: &mut Ctx) {
    ctx.dx.backbuffer.states.clear();
}

/// Original: `devilution::RedrawComponent` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::RedrawComponent(PanelDrawComponent component) sha=dcf316c15a3c
pub fn redraw_component(ctx: &mut Ctx, component: PanelDrawComponent) {
    for (_, s) in ctx.dx.backbuffer.states.iter_mut() {
        s.redraw_components[component as usize] = true;
    }
}

/// Original: `devilution::IsRedrawComponent` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::IsRedrawComponent(PanelDrawComponent component) sha=b5e1b38fcbc0
pub fn is_redraw_component(ctx: &mut Ctx, component: PanelDrawComponent) -> bool {
    get_backbuffer_state(ctx).redraw_components[component as usize]
}

/// Original: `devilution::RedrawComponentComplete` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::RedrawComponentComplete(PanelDrawComponent component) sha=30c25d89915a
pub fn redraw_component_complete(ctx: &mut Ctx, component: PanelDrawComponent) {
    get_backbuffer_state(ctx).redraw_components[component as usize] = false;
}

/// Original: `devilution::GetDrawnCursor` (engine/backbuffer_state.cpp).
// @port engine/backbuffer_state.cpp|devilution::GetDrawnCursor() sha=ad690d9e1b96
pub fn get_drawn_cursor(ctx: &mut Ctx) -> &mut DrawnCursor {
    &mut get_backbuffer_state(ctx).cursor
}
