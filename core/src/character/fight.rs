use super::{Damage, Defense, Dice, Enemy, HitPoint, Level, Player, Strength};
use crate::item::ItemToken;
use crate::rng::{Parcent, RngHandle};
use std::iter;

pub fn player_attack(
    player: &Player,
    throw_weapon: Option<ItemToken>,
    enemy: &Enemy,
    rng: &mut RngHandle,
) -> Option<HitPoint> {
    let (attack_rate, dam_plus) = if let Some(ref item) = throw_weapon {
        let (mut hit_plus, mut dam_plus) = (item.hit_plus(), item.dam_plus());
        let name = player.weapon().and_then(|w| w.name());
        if name.is_some() && name == item.launcher() {
            hit_plus += player.weapon().map(|w| w.hit_plus()).unwrap_or(Level(0));
            dam_plus += player.weapon().map(|w| w.dam_plus()).unwrap_or(HitPoint(0));
        }
        let attack_rate = attack_rate_player(player, enemy, hit_plus);
        (attack_rate, dam_plus)
    } else {
        let hit_plus = player.weapon().map(|w| w.hit_plus()).unwrap_or(Level(0));
        let attack_rate = attack_rate_player(player, enemy, hit_plus);
        let dam_plus = player.weapon().map(|w| w.dam_plus()).unwrap_or(HitPoint(0));
        (attack_rate, dam_plus)
    };
    let dice = if let Some(ref item) = throw_weapon {
        item.at_throw()
    } else {
        player.weapon().and_then(|w| w.at_weild())
    }
    .unwrap_or(Dice::new(1, HitPoint(4)));
    roll(
        iter::once(&dice),
        attack_rate,
        dam_plus + damage_plus(player.strength().current),
        rng,
    )
}

pub fn enemy_attack(enemy: &Enemy, player: &Player, rng: &mut RngHandle) -> Option<HitPoint> {
    let attack_rate = attack_rate_enemy(player, enemy);
    let dam_plus = damage_plus(Enemy::STRENGTH);
    roll(
        enemy.attack().iter(),
        attack_rate,
        dam_plus + damage_plus(player.strength().current),
        rng,
    )
}

fn roll<'a>(
    dices: impl Iterator<Item = &'a Dice<HitPoint>>,
    attack_rate: Parcent,
    dam_plus: HitPoint,
    rng: &mut RngHandle,
) -> Option<HitPoint> {
    let mut did_hit = false;
    let mut sum = HitPoint(0);
    for dice in dices {
        if !rng.parcent(attack_rate) {
            continue;
        }
        did_hit = true;
        sum += dice.random(rng) + dam_plus;
    }
    if did_hit {
        Some(sum)
    } else {
        None
    }
}

fn attack_rate_player(player: &Player, enemy: &Enemy, hit_plus: Level) -> Parcent {
    let st = player.strength().current;
    let str_p = hit_prob_plus(st) + if enemy.is_running() { 0 } else { 4 }.into() + hit_plus;
    attack_rate(player.level(), enemy.defense(), str_p)
}

fn attack_rate_enemy(player: &Player, enemy: &Enemy) -> Parcent {
    attack_rate(enemy.level(), player.arm(), hit_prob_plus(Enemy::STRENGTH))
}

fn attack_rate(level: Level, armor: Defense, revision: Level) -> Parcent {
    let val = level.0 + i64::from(armor.0) + revision.0 + 1;
    Parcent::truncate(val * 5)
}

fn hit_prob_plus(strength: Strength) -> Level {
    const DATA: [i64; 32] = [
        -7, -6, -5, -4, -3, -2, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 3,
    ];
    if strength.0 <= 0 || strength.0 > DATA.len() as i64 {
        return Level(0);
    }
    DATA[strength.0 as usize - 1].into()
}

fn damage_plus(strength: Strength) -> HitPoint {
    const DATA: [i64; 32] = [
        -7, -6, -5, -4, -3, -2, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3, 3, 4, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 6,
    ];
    if strength.0 <= 0 || strength.0 > DATA.len() as i64 {
        return HitPoint(0);
    }
    DATA[strength.0 as usize - 1].into()
}
