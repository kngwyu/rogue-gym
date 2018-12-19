use rogue_gym_core::{input::InputCode, RunTime};
use std::sync::mpsc::{Receiver, SyncSender};

#[derive(Clone, Copy, Debug)]
enum Instruction {
    Step(InputCode),
}

struct ThreadWorker {
    runtime: RunTime,
}
