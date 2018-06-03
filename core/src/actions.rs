//! module for handling actions and do some operations related to multiple modules

use character::{Action, Player};
use dungeon::{Direction, Dungeon};
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use item::{pack::PackEntry, ItemHandler};
use {GameInfo, GameMsg, GlobalConfig, Reaction};

pub(crate) fn process_action(
    action: Action,
    config: &GlobalConfig,
    info: &mut GameInfo,
    dungeon: &mut Dungeon,
    item: &mut ItemHandler,
    player: &mut Player,
) -> GameResult<Vec<Reaction>> {
    match action {
        Action::DownStair => {
            if dungeon.is_downstair(player.pos.clone()) {
                new_level(info, dungeon, item, player, false)
                    .chain_err(|| "action::process_action")?;
                Ok(vec![Reaction::Redraw, Reaction::StatusUpdated])
            } else {
                Ok(vec![Reaction::Notify(GameMsg::NoDownStair)])
            }
        }
        Action::UpStair => {
            Err(ErrorId::Unimplemented.into_with(|| "UpStair Command is unimplemented"))
        }
        Action::Move(d) => move_player(d, dungeon, item, player),
    }
}

pub(crate) fn new_level(
    info: &GameInfo,
    dungeon: &mut Dungeon,
    item: &mut ItemHandler,
    player: &mut Player,
    is_init: bool,
) -> GameResult<()> {
    if !is_init {
        dungeon
            .new_level(info, item)
            .chain_err(|| "action::new_level")?;
    }
    player.pos = dungeon
        .select_cell(true)
        .ok_or_else(|| ErrorId::MaybeBug.into_with(|| "action::new_level No space for player!"))?;
    dungeon.enter_room(&player.pos)
}

fn move_player(
    direction: Direction,
    dungeon: &mut Dungeon,
    item: &mut ItemHandler,
    player: &mut Player,
) -> GameResult<Vec<Reaction>> {
    if !dungeon.can_move_player(player.pos.clone(), direction) {
        return Ok(vec![Reaction::Notify(GameMsg::CantMove(direction))]);
    }
    let new_pos = dungeon
        .move_player(&player.pos, direction)
        .chain_err(|| "actions::move_player")?;
    player.pos = new_pos;
    let mut res = vec![Reaction::Redraw];
    if let Some(msg) = get_item(dungeon, item, player).chain_err(|| "in actions::move_player")? {
        res.push(Reaction::Notify(msg));
        res.push(Reaction::StatusUpdated);
    }
    Ok(res)
}

fn get_item(
    dungeon: &mut Dungeon,
    item_handle: &mut ItemHandler,
    player: &mut Player,
) -> GameResult<Option<GameMsg>> {
    macro_rules! try_or_ok {
        ($res:expr) => {
            match $res {
                Some(v) => v,
                None => return Ok(None),
            }
        };
    }
    let placed_id = try_or_ok!(item_handle.get_placed_id(&player.pos));
    let pack_entry = try_or_ok!(player.items.entry(placed_id, item_handle));
    let show_id = match pack_entry {
        PackEntry::Insert(player_entry) => {
            player_entry.exec(&mut player.items, placed_id);
            placed_id
        }
        PackEntry::Merge(player_entry) => {
            let item = match item_handle.remove(placed_id) {
                Some(i) => i,
                None => {
                    return Err(ErrorId::MaybeBug
                        .into_with(|| "[actions::get_item] Invalid ItemId in player's pack"))
                }
            };
            player_entry
                .exec(item_handle, item)
                .chain_err(|| "in actions::get_item")?;
            player_entry.id()
        }
    };

    if !dungeon
        .remove_object(&player.pos, false)
        .chain_err(|| "in actions::get_item")?
    {
        warn!("[actions::get_item] couldn't remove object!!!")
    }
    item_handle.remove_from_place(&player.pos);
    let item = item_handle.get(show_id).ok_or_else(|| {
        ErrorId::MaybeBug.into_with(|| "[actions::get_item] No item for placed_id(bug)")
    })?;
    Ok(Some(GameMsg::GotItem {
        kind: item.kind.clone(),
        num: item.how_many.0,
    }))
}
