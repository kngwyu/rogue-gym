use super::{Damage, Defense, Enemy, HitPoint, Level, Player, Strength};
use crate::rng::RngHandle;
use std::cmp::{max, min};

pub fn player_attack(player: &Player, enemy: &mut Enemy, rng: &mut RngHandle) -> Option<HitPoint> {
    if !hit_attack(player, enemy, rng) {
        return None;
    }
    None
}

pub fn enemy_attack(enemy: &Enemy, player: &mut Player, rng: &mut RngHandle) -> Option<HitPoint> {
    if !hit_defense(player, enemy, rng) {
        return None;
    }
    None
}

fn hit_attack(player: &Player, enemy: &Enemy, rng: &mut RngHandle) -> bool {
    let st = player.status.strength.current;
    let str_p = str_plus(st) + if enemy.is_running() { 0 } else { 4 };
    hit_sub(player.status.level, enemy.defense(), str_p + 1, rng)
}

fn hit_defense(player: &Player, enemy: &Enemy, rng: &mut RngHandle) -> bool {
    let arm = Defense::max() - player.arm();
    hit_sub(enemy.level(), arm, 1, rng)
}

fn hit_sub(level: Level, armor: Defense, revision: i64, rng: &mut RngHandle) -> bool {
    const HIT_RATE_MAX: i64 = 20;
    let mut val = level.0 + armor.0 + revision;
    val = min(val, HIT_RATE_MAX);
    val = max(val, 0);
    rng.parcent((val * 100 / HIT_RATE_MAX) as u32)
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
