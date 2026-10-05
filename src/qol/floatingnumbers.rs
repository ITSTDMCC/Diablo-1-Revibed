//! `Source/qol/floatingnumbers.cpp` (pending).

use crate::ctx::Ctx;
use crate::enums::DamageType;

crate::pending_fn!(pub fn add_floating_number_player(ctx: &mut Ctx, damage_type: DamageType, pnum: usize, damage: i32), "qol/floatingnumbers.cpp|devilution::AddFloatingNumber(DamageType damageType, const Player &player, int damage)");
