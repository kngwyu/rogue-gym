use super::{Damage, Defense, Enemy, HitPoint, Level, Player, Strength};
use crate::rng::{Parcent, RngHandle};
use std::cmp::{max, min};

pub fn player_attack(player: &Player, enemy: &mut Enemy, rng: &mut RngHandle) -> Option<HitPoint> {
    if !rng.parcent(hit_attack(player, enemy)) {
        return None;
    }
    None
}

pub fn enemy_attack(enemy: &Enemy, player: &mut Player, rng: &mut RngHandle) -> Option<HitPoint> {
    if !rng.parcent(hit_defense(player, enemy)) {
        return None;
    }
    None
}

fn hit_attack(player: &Player, enemy: &Enemy) -> Parcent {
    let st = player.status.strength.current;
    let str_p = str_plus(st) + if enemy.is_running() { 0 } else { 4 };
    hit_sub(player.status.level, enemy.defense(), str_p + 1)
}

fn hit_defense(player: &Player, enemy: &Enemy) -> Parcent {
    let arm = Defense::max() - player.arm();
    hit_sub(enemy.level(), arm, 1)
}

fn hit_sub(level: Level, armor: Defense, revision: i64) -> Parcent {
    const HIT_RATE_MAX: u32 = 20;
    let mut val = level.0 + armor.0 + revision;
    val = min(val, i64::from(HIT_RATE_MAX));
    val = max(val, 0);
    Parcent::new((100 / HIT_RATE_MAX) * val as u32)
}

fn str_plus(strength: Strength) -> i64 {
    const STR_PLUS: [i64; 32] = [
        -7, -6, -5, -4, -3, -2, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 3,
    ];
    if strength.0 <= 0 || strength.0 > STR_PLUS.len() as i64 {
        return 0;
    }
    STR_PLUS[strength.0 as usize - 1]
}
