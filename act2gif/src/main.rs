#[macro_use]
extern crate failure;

mod draw;
mod font;
mod term_image;
mod theme;
use self::draw::GifEncoder;
use clap::{self, ArgMatches};
use rogue_gym_core::{error::*, input::InputCode, json_to_inputs, read_file, GameConfig};
const UBUNTU_MONO: &[u8; 205748] = include_bytes!("../../data/fonts/UbuntuMono-R.ttf");
use self::font::FontHandle;
use self::theme::Theme;
use anyhow::{bail, Context, Result};

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
                .default_value("50")
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
                .default_value("16")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("max_actions")
                .short("m")
                .long("max")
                .value_name("MAX")
                .help("Replay only <MAX> times")
                .default_value("30")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("seed")
                .short("s")
                .long("seed")
                .value_name("SEED")
                .help("Specify seed")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("Specify the name of output GIF file")
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
    let f = read_file(file_name).with_context(|| "in get_config")?;
    GameConfig::from_json(&f)
}

fn get_replay(args: &ArgMatches) -> GameResult<Vec<InputCode>> {
    let fname = args.value_of("actions").unwrap();
    let replay = read_file(fname).with_context(|| "Failed to read replay file!")?;
    json_to_inputs(&replay)
}

fn get_arg<T: ::std::str::FromStr>(args: &ArgMatches, value: &str) -> Option<T> {
    args.value_of(value).and_then(|v| v.parse::<T>().ok())
}

fn get_out_file(args: &ArgMatches) -> GameResult<String> {
    let name = args.value_of("output").unwrap();
    let contains_dot = name.contains(".");
    if contains_dot && !name.ends_with(".gif") {
        bail!("Invalid gif file name {}", name);
    }
    let mut res = name.to_string();
    if !contains_dot {
        res += ".gif";
    }
    Ok(res)
}

fn setup<'a>() -> GameResult<(GifEncoder<'a>, Vec<InputCode>, String)> {
    let args = parse_args();
    let mut config = get_config(&args)?;
    let mut replay = get_replay(&args)?;
    let interval = get_arg(&args, "interval").unwrap();
    let scale = get_arg(&args, "fontsize").unwrap();
    let max = get_arg(&args, "max_actions").unwrap();
    if let Some(seed) = get_arg(&args, "seed") {
        config.seed = Some(seed);
    }
    replay.truncate(max);
    let theme = args.value_of("theme").unwrap_or("solarized-dark");
    let theme = Theme::from_str(theme).expect("Unknown theme was specified");
    let font = FontHandle::new(&UBUNTU_MONO[..], scale);
    let out_file = get_out_file(&args)?;
    Ok((
        GifEncoder::new(config, font, scale, theme, interval),
        replay,
        out_file,
    ))
}

fn main() -> GameResult<()> {
    let (mut encoder, inputs, out_file) = setup()?;
    encoder.exec(inputs, &out_file)
}
