use rogue_gym_core::{
    error::{GameResult, ResultExt2},
    GameConfig,
};
use state_impls::GameStateImpl;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use PlayerState;

pub(crate) struct ThreadConductor {
    receivers: Vec<Receiver<GameResult<(PlayerState, bool)>>>,
    senders: Vec<SyncSender<Instruction>>,
}

impl ThreadConductor {
    const SENDER_BOUND: usize = 4;
    pub fn new(configs: Vec<GameConfig>, max_steps: usize) -> GameResult<Self> {
        let mut receivers = vec![];
        let mut senders = vec![];
        for config in configs {
            let state = GameStateImpl::new(config.clone(), max_steps)?;
            let (tx1, rx1) = mpsc::sync_channel(Self::SENDER_BOUND);
            let (tx2, rx2) = mpsc::sync_channel(Self::SENDER_BOUND);
            thread::spawn(move || {
                let mut worker = ThreadWorker {
                    game_state: state,
                    receiver: rx1,
                    sender: tx2,
                    config,
                };
                worker.run();
            });
            receivers.push(rx2);
            senders.push(tx1);
        }
        Ok(ThreadConductor { receivers, senders })
    }
    pub fn stop(self) -> GameResult<()> {
        for sender in self.senders {
            sender.send(Instruction::Stop).compat()?;
        }
        Ok(())
    }
    pub fn reset(&mut self) -> GameResult<Vec<PlayerState>> {
        for sender in &mut self.senders {
            sender.send(Instruction::Reset).compat()?;
        }
        let mut result = vec![];
        for res in self.receivers.iter_mut().map(|rx| rx.recv().compat()) {
            result.push(res??.0);
        }
        Ok(result)
    }
    pub fn states(&mut self) -> GameResult<Vec<PlayerState>> {
        for sender in &mut self.senders {
            sender.send(Instruction::State).compat()?;
        }
        let mut result = vec![];
        for res in self.receivers.iter_mut().map(|rx| rx.recv().compat()) {
            result.push(res??.0);
        }
        Ok(result)
    }
    pub fn step(&mut self, inputs: Vec<u8>) -> GameResult<Vec<(PlayerState, bool)>> {
        for (sender, input) in self.senders.iter_mut().zip(inputs) {
            sender.send(Instruction::Step(input)).compat()?;
        }
        let mut result = vec![];
        for res in self.receivers.iter_mut().map(|rx| rx.recv().compat()) {
            result.push(res??);
        }
        Ok(result)
    }
}

#[derive(Clone, Debug)]
enum Instruction {
    Step(u8),
    Reset,
    State,
    Stop,
}

unsafe impl Send for Instruction {}

struct ThreadWorker {
    game_state: GameStateImpl,
    config: GameConfig,
    receiver: Receiver<Instruction>,
    sender: SyncSender<GameResult<(PlayerState, bool)>>,
}

impl ThreadWorker {
    fn run(&mut self) {
        for inst in self.receiver.iter() {
            match inst {
                Instruction::Step(code) => {
                    let res = self
                        .game_state
                        .react(code)
                        .map(|done| (self.game_state.state(), done));
                    self.sender.send(res)
                }
                Instruction::Reset => {
                    let res = self
                        .game_state
                        .reset(self.config.clone())
                        .map(|_| (self.game_state.state(), false));
                    self.sender.send(res)
                }
                Instruction::State => self.sender.send(Ok((self.game_state.state(), false))),
                Instruction::Stop => break,
            }
            .expect("ThreadWorker: disconnected")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const CONFIG: &str = r#"
{
    "width": 32,
    "height": 16,
    "seed": 0,
    "dungeon": {
        "style": "rogue",
        "room_num_x": 2,
        "room_num_y": 2,
        "min_room_size": {
            "x": 4,
            "y": 4
        }
    }
}
"#;
    #[test]
    fn test_threads() {
        use std::iter::repeat_with;
        let config = GameConfig::from_json(CONFIG).unwrap();
        let mut threads =
            ThreadConductor::new(repeat_with(|| config.clone()).take(8).collect(), 100).unwrap();
        let states = threads.states().unwrap();
        for state in &states {
            assert_eq!(*state, states[0]);
        }
        let actions: Vec<_> = "hjklyubn".as_bytes().iter().map(|&x| x).collect();
        let states = threads.step(actions).unwrap();
        let mut same = true;
        for state in &states {
            same &= *state == states[0];
        }
        assert!(!same);
        threads.stop().unwrap();
    }
}
