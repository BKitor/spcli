mod infocmd;
mod playbackcmd;

use crate::infocmd::print_info;
use crate::playbackcmd::modify_playback;
use clap::{Parser, Subcommand};
use home::home_dir;
use rspotify::{prelude::*, AuthCodeSpotify, Config as RSptConfig, Credentials, OAuth};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::PathBuf;

const SCOPES: [&str; 14] = [
    "playlist-read-collaborative",
    "playlist-read-private",
    "playlist-modify-private",
    "playlist-modify-public",
    "user-follow-read",
    "user-follow-modify",
    "user-library-modify",
    "user-library-read",
    "user-modify-playback-state",
    "user-read-currently-playing",
    "user-read-playback-state",
    "user-read-playback-position",
    "user-read-private",
    "user-read-recently-played",
];

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Parser, Debug)]
struct InfoCmd {
    #[arg(short, long)]
    artist: bool,
    #[arg(short = 'A', long)]
    album: bool,
    #[arg(short, long)]
    song: bool,
}

#[derive(Parser, Debug)]
struct PlayBackCmd {
    #[arg(short, long)]
    toggle: bool,
    #[arg(short, long)]
    next: bool,
    #[arg(short, long)]
    prev: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(visible_alias = "i")]
    Info(InfoCmd),
    #[command(visible_alias = "pb")]
    PlayBack(PlayBackCmd),
}

fn build_scopes() -> HashSet<String> {
    let mut hs = HashSet::new();
    for s in SCOPES {
        hs.insert(s.to_string());
    }
    return hs;
}

#[derive(Deserialize, Debug)]
struct Config {
    client_id: String,
    client_secret: String,
    reddirect_uri: String,
    prefered_device: Option<String>,
}

#[tokio::main]
async fn main() {
    // Rework duplicate error handling with ? operator

    // Tab aligned output (find crate?)

    // start album playback
    // start album/artist/song radio
    //
    // device mgmt
    //  Some sort of way to save deviecs to a cfg file?
    //  If request playback, and no context found, prompt for device
    //  Currently just grab dev[0]

    let args = Args::parse();

    let cfg_file: PathBuf = home_dir().unwrap().join(".config/spcli/config.toml");
    let cfg_toml_str = std::fs::read_to_string(cfg_file).unwrap();
    let config: Config = toml::from_str(&cfg_toml_str).unwrap();

    let creds = Credentials::new(&config.client_id, &config.client_secret);
    let oauth = OAuth {
        scopes: build_scopes(),
        redirect_uri: config.reddirect_uri.clone(),
        ..Default::default()
    };
    let cache_token_file: PathBuf = home_dir()
        .unwrap()
        .join(".config/spcli/.spotify_token_cache.json");
    let rspt_config = RSptConfig {
        cache_path: PathBuf::from(cache_token_file),
        token_cached: true,
        ..Default::default()
    };
    let spotify = AuthCodeSpotify::with_config(creds, oauth, rspt_config);
    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).await.unwrap();
    let _ = spotify.write_token_cache();

    let cmd: Command = match args.command {
        Some(cmd) => cmd,
        None => Command::Info(InfoCmd {
            artist: true,
            album: true,
            song: true,
        }),
    };

    match cmd {
        Command::Info(c) => print_info(&spotify, &c).await,
        Command::PlayBack(c) => modify_playback(&spotify, &c, config.prefered_device).await,
    }
}
