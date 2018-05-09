//! module for handling actions and do some operations related to multiple modules

use character::{Action, Player};
use dungeon::Dungeon;
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use item::ItemHandler;
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
                new_level(info, dungeon, player, item, false)
                    .chain_err("[action::process_action]")?;
                Ok(vec![Reaction::Redraw])
            } else {
                Ok(vec![Reaction::Notify(GameMsg::NoDownStair)])
            }
        }
        Action::UpStair => {
            Err(ErrorId::Unimplemented.into_with("UpStair Command is unimplemented"))
        }
        Action::Move(d) => {
            if !dungeon.can_move_player(player.pos.clone(), d) {
                return Ok(vec![Reaction::Notify(GameMsg::CantMove)]);
            }
            let new_pos = dungeon
                .move_player(player.pos.clone(), d)
                .chain_err("[action::process_action]")?;
            player.pos = new_pos;
            Ok(vec![Reaction::Redraw])
        }
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
            .chain_err("[action::process_action]")?;
    }

    Ok(())
}
