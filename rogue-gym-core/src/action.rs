use {GameInfo, GlobalConfig, Reaction, RunTime};
use dungeon::Dungeon;
use character::{Action, Player};
use item::ItemHandler;
use error::{ErrorId, ErrorKind, GameResult, ResultExt};

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
            let a = 4;
            if dungeon.is_downstair(player.pos.clone()) {
                dungeon.new_level(info, item);
            }
        }
        Action::UpStair => {}
        Action::Move(d) => {}
    }
    // STUB!
    Ok(vec![])
}
