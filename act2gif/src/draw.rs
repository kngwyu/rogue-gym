use crate::font::FontHandle;
use crate::term_image::TermImage;
use crate::theme::Theme;
use anyhow::{Context, Result};
use image::gif::{DisposalMethod, Encoder};
use rogue_gym_core::{error::*, input::InputCode, GameConfig, Reaction};
use rogue_gym_uilib::process_reaction;
use std::fs::File;
use std::io::BufWriter;

pub struct GifEncoder<'a> {
    config: GameConfig,
    font: FontHandle<'a>,
    scale: u32,
    theme: Theme,
    interval: u32,
}

impl<'a> GifEncoder<'a> {
    pub fn new(
        config: GameConfig,
        font: FontHandle<'a>,
        scale: u32,
        theme: Theme,
        interval: u32,
    ) -> Self {
        GifEncoder {
            config,
            font,
            scale,
            theme,
            interval,
        }
    }
    pub fn exec(&mut self, inputs: Vec<InputCode>, filename: &str) -> GameResult<()> {
        let mut runtime = self.config.clone().build()?;
        let file = File::create(filename).with_context(|| "Failed to crate file")?;
        let writer = BufWriter::new(file);
        let mut encoder = Encoder::new(writer);
        let mut term = TermImage::new(
            self.config.width.into(),
            self.config.height.into(),
            self.scale,
            self.theme.back,
            self.theme.font,
            &mut self.font,
        );
        for i in inputs {
            let reaction = match runtime.react_to_input(i) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("error: {}", e);
                    continue;
                }
            };
            for r in reaction {
                let draw = r == Reaction::Redraw;
                process_reaction(&mut term, &mut runtime, r)?;
                if draw {
                    let mut frame = term.frame();
                    frame.delay = self.interval as u16;
                    frame.dispose = DisposalMethod::Background;
                    encoder.encode(&frame).expect("Failed to encode gif");
                }
            }
        }
        Ok(())
    }
}
