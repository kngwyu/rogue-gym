#[macro_use]
extern crate failure;

mod draw;
mod font;
mod term_image;
mod theme;
use self::draw::GifEncoder;
use clap::{self, ArgMatches};
use rogue_gym_core::{error::*, input::InputCode, json_to_inputs, read_file, GameConfig};
const DEFAULT_INTERVAL_MS: u32 = 50;
const DEFAULT_MAX_ACTIONS: usize = 30;
const DEFAULT_FONT_SIZE: u32 = 16;
const UBUNTU_MONO: &[u8; 205748] = include_bytes!("../../data/fonts/UbuntuMono-R.ttf");
use self::font::FontHandle;
use self::theme::Theme;

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
        .arg(
            clap::Arg::with_name("theme")
                .short("t")
                .long("theme")
                .value_name("THEME")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("fontsize")
                .short("f")
                .long("font")
                .value_name("FONT")
                .help("Font size")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("max_actions")
                .short("m")
                .long("max")
                .value_name("MAX")
                .help("Replay only <MAX> times")
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

fn get_replay(args: &ArgMatches) -> GameResult<Vec<InputCode>> {
    let fname = args.value_of("actions").unwrap();
    let replay = read_file(fname).into_chained(|| "Failed to read replay file!")?;
    json_to_inputs(&replay)
}

fn get_arg<T: ::std::str::FromStr>(args: &ArgMatches, value: &str) -> Option<T> {
    args.value_of(value).and_then(|v| v.parse::<T>().ok())
}

fn setup<'a>() -> GameResult<(GifEncoder<'a>, Vec<InputCode>)> {
    let args = parse_args();
    let config = get_config(&args)?;
    let mut replay = get_replay(&args)?;
    let interval = get_arg(&args, "interval").unwrap_or(DEFAULT_INTERVAL_MS);
    let scale = get_arg(&args, "fontsize").unwrap_or(DEFAULT_FONT_SIZE);
    let max = get_arg(&args, "max_actions").unwrap_or(DEFAULT_MAX_ACTIONS);
    replay.truncate(max);
    let theme = args.value_of("theme").unwrap_or("solarized-dark");
    let theme = Theme::from_str(theme).expect("Unknown theme was specified");
    let font = FontHandle::new(&UBUNTU_MONO[..], scale);
    Ok((
        GifEncoder::new(config, font, scale, theme, interval),
        replay,
    ))
}

fn main() -> GameResult<()> {
    let (mut encoder, inputs) = setup()?;
    encoder.exec(inputs, "rogue-gym.gif")
}
