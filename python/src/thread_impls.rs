use rogue_gym_core::{input::InputCode, GameConfig, RunTime};
use std::sync::mpsc::{Receiver, SyncSender};

#[derive(Clone, Debug)]
enum Instruction {
    Step(InputCode),
    Reset(GameConfig),
}

unsafe impl Send for Instruction {}

struct ThreadWorker {
    runtime: RunTime,
}
