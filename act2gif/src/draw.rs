use crate::font::FontHandle;
use crate::term_image::TermImage;
use image::gif::{DisposalMethod, Encoder};
use rogue_gym_core::{
    character::player::Status, dungeon::Positioned, error::*, input::InputCode, ui::*, GameConfig,
    GameMsg, Reaction, RunTime,
};
use std::collections::VecDeque;
use std::fs::File;

pub struct GifEncoder<'a> {
    inputs: VecDeque<InputCode>,
    config: GameConfig,
    font: FontHandle<'a>,
    term: TermImage,
    interval: u32,
}

impl<'a> GifEncoder<'a> {
    pub fn new(
        inputs: VecDeque<InputCode>,
        config: GameConfig,
        font: FontHandle<'a>,
        term: TermImage,
        interval: u32,
    ) -> Self {
        GifEncoder {
            inputs,
            config,
            font,
            term,
            interval,
        }
    }
    pub fn exec(&mut self, filename: &str) -> GameResult<()> {
        let mut runtime = self.config.clone().build()?;
        let file = File::create(filename).into_chained(|| "Failed to crate file")?;
        let mut encoder = Encoder::new(file);
        for &i in self.inputs.clone().iter().take(20) {
            let reaction = runtime.react_to_input(i)?;
            for r in reaction {
                let draw = r == Reaction::Redraw;
                self.process_reaction(&mut runtime, r);
                if draw {
                    let mut frame = self.term.frame();
                    frame.delay = self.interval as u16;
                    frame.dispose = DisposalMethod::Background;
                    encoder.encode(&frame).expect("Failed to encode gif");
                }
            }
        }
        Ok(())
    }
    fn process_reaction(&mut self, runtime: &mut RunTime, reaction: Reaction) {
        match reaction {
            Reaction::Notify(msg) => match msg {
                GameMsg::CantMove(_) => {}
                GameMsg::CantGetItem(kind) => self.message(format!("You walk onto {:?}", kind)),
                GameMsg::NoDownStair => {
                    self.message(format!("Hmm... there seems to be no downstair"))
                }
                GameMsg::GotItem { kind, num } => {
                    self.message(format!("Now you have {} {:?}", num, kind));
                }
                GameMsg::SecretDoor => self.message(format!("you found a secret door")),
                GameMsg::HitTo(s) => self.message(format!("You swings and hit {}", s)),
                GameMsg::HitFrom(s) => self.message(format!("{} swings and hits you", s)),
                GameMsg::MissTo(s) => self.message(format!("You swing and miss {}", s)),
                GameMsg::MissFrom(s) => self.message(format!("{} swings and misses you", s)),
                GameMsg::Quit => {
                    self.message(format!("Thank you for playing!"));
                }
            },
            Reaction::Redraw => {
                self.dungeon(runtime);
            }
            Reaction::StatusUpdated => {
                let status = runtime.player_status();
                self.status(status);
            }
            Reaction::UiTransition(ui_state) => {
                if let UiState::Mordal(kind) = ui_state {
                    match kind {
                        MordalKind::Quit => {
                            self.message(format!("You really quit game?(y/n)"));
                        }
                    }
                }
            }
        }
    }
    fn message(&mut self, s: String) {
        self.term.write_msg(&s, &mut self.font);
    }
    fn status(&mut self, status: Status) {
        let status = format!("{}", status);
        self.term.write_status(&status, &mut self.font)
    }
    fn dungeon(&mut self, runtime: &mut RunTime) {
        runtime
            .draw_screen(|Positioned(cd, tile)| {
                self.term.write_char(cd, tile.to_char(), &mut self.font);
                Ok(())
            })
            .unwrap();
    }
}
