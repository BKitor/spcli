mod infocmd;
mod playbackcmd;

use crate::infocmd::print_info;
use crate::playbackcmd::modify_playback;
use clap::{Parser, Subcommand};
use rspotify::{prelude::*, AuthCodeSpotify, Config, Credentials, OAuth};
use std::collections::HashSet;
use std::path::PathBuf;

const CACHE_PATH: &str = "/home/bkitor/.config/bkspt/.spotify_token_cache.json";
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

#[tokio::main]
async fn main() {
    // start album playback
    // start album/artist/song radio
    //
    // device mgmt
    //  Some sort of way to save deviecs to a cfg file?
    //  If request playback, and no context found, prompt for device
    //  Currently just grab dev[0]

    let args = Args::parse();

    // Setup spotify client and auth
    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(build_scopes()).unwrap();
    let config = Config {
        cache_path: PathBuf::from(CACHE_PATH),
        token_cached: true,
        ..Default::default()
    };
    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);
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
        Command::PlayBack(c) => modify_playback(&spotify, &c).await,
    }
}
