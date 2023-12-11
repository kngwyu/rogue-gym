//! module for handling actions and do some operations related to multiple modules
use log::warn;

use crate::character::{
    fight, player::PlayerEvent, Action, DamageReaction, Enemy, EnemyHandler, Player,
};
use crate::dungeon::{Direction, Dungeon, DungeonPath};
use crate::error::*;
use crate::item::{itembox::Entry as ItemEntry, ItemHandler, ItemToken};
use crate::ui::UiState;
use crate::{GameInfo, GameMsg, Reaction};
use anyhow::{bail, Context};
use std::iter;
use std::rc::Rc;

pub(crate) fn process_action(
    action: Action,
    info: &mut GameInfo,
    dungeon: &mut dyn Dungeon,
    item: &mut ItemHandler,
    player: &mut Player,
    enemies: &mut EnemyHandler,
) -> GameResult<(Option<UiState>, Vec<Reaction>)> {
    let mut out = Vec::new();
    let mut ui = None;
    match action {
        Action::DownStair => {
            if dungeon.is_downstair(&player.pos) {
                new_level(info, dungeon, item, player, enemies, false)
                    .context("action::process_action")?;
                out.extend_from_slice(&[Reaction::Redraw, Reaction::StatusUpdated]);
            } else {
                out.push(Reaction::Notify(GameMsg::NoDownStair));
            }
            ui = after_turn(player, enemies, dungeon, &mut out)?;
        }
        Action::UpStair => {
            bail!(ErrorKind::Unimplemented("UpStair Command"));
        }
        Action::Move(d) => {
            out.append(&mut move_player(d, dungeon, player, enemies)?.0);
            ui = after_turn(player, enemies, dungeon, &mut out)?;
        }
        Action::MoveUntil(d) => loop {
            let res = move_player(d, dungeon, player, enemies)?;
            let tile = dungeon
                .tile(&player.pos)
                .map(|t| t.to_char())
                .unwrap_or(' ');
            if res.1 || (tile != '.' && tile != '#') {
                out.extend(res.0);
                break;
            } else if out.is_empty() {
                out.extend(res.0);
            }
            ui = after_turn(player, enemies, dungeon, &mut out)?;
        },
        Action::Search => {
            out.append(&mut search(dungeon, player)?);
            ui = after_turn(player, enemies, dungeon, &mut out)?;
        }
        Action::NoOp => return Ok((None, out)),
    }
    Ok((ui, out))
}

fn after_turn(
    player: &mut Player,
    enemies: &mut EnemyHandler,
    dungeon: &mut dyn Dungeon,
    res: &mut Vec<Reaction>,
) -> GameResult<Option<UiState>> {
    for event in player.turn_passed(enemies.rng()) {
        match event {
            PlayerEvent::Dead => {}
            PlayerEvent::Healed | PlayerEvent::Hungry => res.push(Reaction::StatusUpdated),
        }
    }
    move_active_enemies(enemies, dungeon, player, res)
}

fn move_active_enemies(
    enemies: &mut EnemyHandler,
    dungeon: &mut dyn Dungeon,
    player: &mut Player,
    res: &mut Vec<Reaction>,
) -> GameResult<Option<UiState>> {
    let attacks = enemies.move_actives(&player.pos, None, dungeon);
    if !attacks.is_empty() {
        player.buttle();
    }
    let mut did_hit = false;
    for at in attacks {
        match fight::enemy_attack(at.enemy(), player, enemies.rng()) {
            Some(hp) => {
                let name = at.enemy().name();
                res.push(Reaction::Notify(GameMsg::HitFrom(name.to_owned())));
                did_hit = true;
                match player.get_damage(hp) {
                    DamageReaction::Death => {
                        let mordal = UiState::die(format!("Killed by {}", name));
                        res.push(Reaction::UiTransition(mordal.clone()));
                        return Ok(Some(mordal));
                    }
                    DamageReaction::None => {}
                }
            }
            None => {
                res.push(Reaction::Notify(GameMsg::MissFrom(
                    at.enemy().name().to_owned(),
                )));
            }
        }
    }
    if did_hit {
        res.push(Reaction::StatusUpdated);
    }
    Ok(None)
}

pub(crate) fn new_level(
    info: &GameInfo,
    dungeon: &mut dyn Dungeon,
    item: &mut ItemHandler,
    player: &mut Player,
    enemies: &mut EnemyHandler,
    is_init: bool,
) -> GameResult<()> {
    if !is_init {
        dungeon
            .new_level(info, item, enemies)
            .context("action::new_level")?;
    }
    player.pos = dungeon.select_cell(true).ok_or(ErrorKind::MaybeBug(
        "action::new_level No space for player!",
    ))?;
    dungeon.enter_room(&player.pos, enemies)
}

fn player_attack(
    player: &mut Player,
    enemy: Rc<Enemy>,
    place: DungeonPath,
    enemies: &mut EnemyHandler,
) -> GameResult<Vec<Reaction>> {
    let mut res = Vec::new();
    player.buttle();
    enemies.activate(place.clone());
    if let Some(hp) = fight::player_attack(player, None, &*enemy, enemies.rng()) {
        res.push(Reaction::Notify(GameMsg::HitTo(enemy.name().to_owned())));
        match enemy.get_damage(hp) {
            DamageReaction::Death => {
                enemies.remove(place);
                if player.level_up(enemy.exp(), enemies.rng()) {
                    res.push(Reaction::StatusUpdated);
                }
                res.push(Reaction::Notify(GameMsg::Killed(enemy.name().to_owned())));
                res.push(Reaction::Redraw);
            }
            DamageReaction::None => {}
        }
    } else {
        res.push(Reaction::Notify(GameMsg::MissTo(enemy.name().to_owned())));
    }
    Ok(res)
}

fn move_player(
    direction: Direction,
    dungeon: &mut dyn Dungeon,
    player: &mut Player,
    enemies: &mut EnemyHandler,
) -> GameResult<(Vec<Reaction>, bool)> {
    let new_pos = if let Some(next) = dungeon.can_move_player(&player.pos, direction) {
        next
    } else {
        return Ok((vec![Reaction::Notify(GameMsg::CantMove(direction))], true));
    };
    if let Some(enemy) = enemies.get_cloned(&new_pos) {
        return player_attack(player, enemy, new_pos, enemies).map(|r| (r, true));
    }
    let new_pos = dungeon
        .move_player(&player.pos, direction, enemies)
        .context("actions::move_player")?;
    player.pos = new_pos;
    player.run(true);
    let mut done = false;
    let mut res = vec![Reaction::Redraw];
    if let Some(msg) = get_item(dungeon, player).context("in actions::move_player")? {
        res.push(Reaction::Notify(msg));
        res.push(Reaction::StatusUpdated);
        done = true;
    }
    Ok((res, done))
}

fn search(dungeon: &mut dyn Dungeon, player: &mut Player) -> GameResult<Vec<Reaction>> {
    dungeon.search(&player.pos).map(|v| {
        v.into_iter()
            .map(|msg| Reaction::Notify(msg))
            .chain(iter::once(Reaction::Redraw))
            .collect()
    })
}

fn get_item(dungeon: &mut dyn Dungeon, player: &mut Player) -> GameResult<Option<GameMsg>> {
    macro_rules! try_or_ok {
        ($res: expr) => {
            match $res {
                Some(v) => v,
                None => return Ok(None),
            }
        };
    }
    let got_item = {
        let item_ref = try_or_ok!(dungeon.get_item(&player.pos));
        let pack_entry = try_or_ok!(player.itembox.entry(item_ref));
        match pack_entry {
            ItemEntry::Insert(player_entry) => player_entry.exec(ItemToken::clone(item_ref)),
            ItemEntry::Merge(player_entry) => player_entry.exec(item_ref.get().clone()),
        }
    };
    if dungeon.remove_item(&player.pos).is_none() {
        warn!("[actions::get_item] couldn't remove object!!!")
    }
    //dungeon.remove_from_place(&player.pos);
    Ok(Some(GameMsg::GotItem {
        kind: got_item.kind.clone(),
        num: got_item.how_many.0,
    }))
}
