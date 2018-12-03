use super::{Damage, Defense, Enemy, HitPoint, Level, Player, Strength};
use crate::rng::{Parcent, RngHandle};

pub fn player_attack(player: &Player, enemy: &mut Enemy, rng: &mut RngHandle) -> Option<HitPoint> {
    if !rng.parcent(attack_rate_player(player, enemy)) {
        return None;
    }
    None
}

pub fn enemy_attack(enemy: &Enemy, player: &mut Player, rng: &mut RngHandle) -> Option<HitPoint> {
    if !rng.parcent(attack_rate_enemy(player, enemy)) {
        return None;
    }
    None
}

fn attack_rate_player(player: &Player, enemy: &Enemy) -> Parcent {
    let st = player.strength().current;
    let str_p = hit_prob_plus(st) + if enemy.is_running() { 0 } else { 4 };
    attack_rate(player.level(), enemy.defense(), str_p + 1)
}

fn attack_rate_enemy(player: &Player, enemy: &Enemy) -> Parcent {
    attack_rate(enemy.level(), player.arm(), 1)
}

fn attack_rate(level: Level, armor: Defense, revision: i64) -> Parcent {
    let val = level.0 + i64::from(armor.0) + revision;
    Parcent::truncate(val * 5)
}

fn hit_prob_plus(strength: Strength) -> i64 {
    const DATA: [i64; 32] = [
        -7, -6, -5, -4, -3, -2, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 3,
    ];
    if strength.0 <= 0 || strength.0 > DATA.len() as i64 {
        return 0;
    }
    DATA[strength.0 as usize - 1]
}

fn damage_plus(strength: Strength) -> i64 {
    const DATA: [i64; 32] = [
        -7, -6, -5, -4, -3, -2, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3, 3, 4, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 6,
    ];
    if strength.0 <= 0 || strength.0 > DATA.len() as i64 {
        return 0;
    }
    DATA[strength.0 as usize - 1]
}
