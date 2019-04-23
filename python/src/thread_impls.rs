use rogue_gym_core::{
    error::{GameResult, ResultExt2},
    GameConfig,
};
use state_impls::GameStateImpl;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use PlayerState;

pub(crate) struct ThreadConductor {
    receivers: Vec<Receiver<GameResult<PlayerState>>>,
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
    pub fn reset(&mut self) -> GameResult<Vec<PlayerState>> {
        for sender in &mut self.senders {
            sender.send(Instruction::Reset).compat()?;
        }
        let mut result = vec![];
        for res in self.receivers.iter_mut().map(|rx| rx.recv().compat()) {
            result.push(res??);
        }
        Ok(result)
    }
    pub fn seed(&mut self, seeds: Vec<u128>) -> GameResult<()> {
        for (sender, seed) in self.senders.iter_mut().zip(seeds) {
            sender.send(Instruction::Seed(seed)).compat()?;
        }
        Ok(())
    }
    pub fn states(&mut self) -> GameResult<Vec<PlayerState>> {
        for sender in &mut self.senders {
            sender.send(Instruction::State).compat()?;
        }
        let mut result = vec![];
        for res in self.receivers.iter_mut().map(|rx| rx.recv().compat()) {
            result.push(res??);
        }
        Ok(result)
    }
    pub fn step(&mut self, inputs: Vec<u8>) -> GameResult<Vec<PlayerState>> {
        for (sender, input) in self.senders.iter_mut().zip(inputs) {
            sender.send(Instruction::Step(input)).compat()?;
        }
        let mut result = vec![];
        for res in self.receivers.iter_mut().map(|rx| rx.recv().compat()) {
            result.push(res??);
        }
        for (i, res) in result.iter().enumerate() {
            if res.is_terminal {
                self.senders[i].send(Instruction::Reset).compat()?;
            }
        }
        for (i, res) in result.iter_mut().enumerate() {
            if res.is_terminal {
                *res = self.receivers[i].recv().compat()??;
                res.is_terminal = true;
            }
        }
        Ok(result)
    }
    pub fn close(&mut self) -> GameResult<()> {
        for sender in &mut self.senders {
            sender.send(Instruction::Stop).compat()?;
        }
        Ok(())
    }
}

/// Thread instruction
/// have no 'stop' or 'close', becase they're not integrated with python's GC well.
#[derive(Clone, Debug)]
enum Instruction {
    Step(u8),
    Reset,
    Seed(u128),
    State,
    Stop,
}

unsafe impl Send for Instruction {}

struct ThreadWorker {
    game_state: GameStateImpl,
    config: GameConfig,
    receiver: Receiver<Instruction>,
    sender: SyncSender<GameResult<PlayerState>>,
}

impl ThreadWorker {
    fn run(&mut self) {
        for inst in self.receiver.iter() {
            match inst {
                Instruction::Step(code) => {
                    let res = self.game_state.react(code).map(|_| self.game_state.state());
                    self.sender.send(res)
                }
                Instruction::Reset => {
                    let res = self
                        .game_state
                        .reset(self.config.clone())
                        .map(|_| self.game_state.state());
                    self.sender.send(res)
                }
                Instruction::Seed(seed) => {
                    self.config.seed_range = Some([seed, seed + 1]);
                    continue;
                }
                Instruction::State => self.sender.send(Ok(self.game_state.state())),
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
    }
}
