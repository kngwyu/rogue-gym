mod draw;
mod font;
mod term_image;
use self::draw::GifEncoder;
use self::term_image::TermImage;
use clap::{self, ArgMatches};
use rogue_gym_core::{error::*, input::InputCode, json_to_inputs, read_file, GameConfig};
use std::collections::VecDeque;
const DEFAULT_INTERVAL_MS: u32 = 50;
const UBUNTU_MONO: &[u8; 205748] = include_bytes!("../../data/fonts/UbuntuMono-R.ttf");
use self::font::FontHandle;
use image::Rgb;

fn parse_args<'a>() -> ArgMatches<'a> {
    clap::App::new("rogue-gym-act2gif")
        .version("0.1.0")
        .author("Yuji Kanagawa <yuji.kngw.80s.revive@gmail.com>")
        .about("make gif file from rogue-gym config and action file")
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG")
                .help("Sets your config json file")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("actions")
                .short("a")
                .long("actions")
                .required(true)
                .value_name("ACTIONS")
                .help("replay json file")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("interval")
                .short("i")
                .long("interval")
                .value_name("INTERVAL")
                .help("Interval in replay mode")
                .takes_value(true),
        )
        .get_matches()
}

fn get_config(args: &ArgMatches) -> GameResult<GameConfig> {
    let file_name = match args.value_of("config") {
        Some(fname) => fname,
        None => {
            return Ok(GameConfig::default());
        }
    };
    let f = read_file(file_name).into_chained(|| "in get_config")?;
    GameConfig::from_json(&f)
}

fn get_replay(args: &ArgMatches) -> GameResult<VecDeque<InputCode>> {
    let fname = args.value_of("actions").unwrap();
    let replay = read_file(fname).into_chained(|| "Failed to read replay file!")?;
    json_to_inputs(&replay)
}

fn setup<'a>() -> GameResult<GifEncoder<'a>> {
    let args = parse_args();
    let config = get_config(&args)?;
    let replay = get_replay(&args)?;
    let mut interval = DEFAULT_INTERVAL_MS;
    if let Some(inter) = args.value_of("interval") {
        interval = inter
            .parse()
            .into_chained(|| "Failed to parse 'interval' arg!")?;
    }
    let black = Rgb([0, 0, 0]);
    let white = Rgb([255, 255, 255]);
    let scale = 16;
    let term = TermImage::new(
        config.width.into(),
        config.height.into(),
        scale,
        white,
        black,
    );
    let font = FontHandle::new(&UBUNTU_MONO[..], scale);
    Ok(GifEncoder::new(replay, config, font, term, interval))
}

fn main() -> GameResult<()> {
    let mut encoder = setup()?;
    encoder.exec("rogue.gif")
}
