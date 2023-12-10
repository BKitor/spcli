use chrono::Duration;
use clap::{Parser, Subcommand};
use rspotify::{
    model::{
        AdditionalType, AlbumId, AlbumType, Country, FullAlbum, FullArtist, FullTrack, Market,
        PlayableItem, SimplifiedAlbum, SimplifiedArtist,
    },
    prelude::*,
    AuthCodeSpotify, Config, Credentials, OAuth,
};
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
    skip: bool,
    #[arg(short, long)]
    next: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(visible_alias = "i")]
    Info(InfoCmd),
    #[command(visible_alias = "pb")]
    PlayBack(PlayBackCmd),
}

fn artist_slice_str(aslice: &[SimplifiedArtist]) -> String {
    let mut artists_str = String::new();
    for a in aslice {
        artists_str.push('"');
        artists_str.push_str(&a.name);
        artists_str.push_str("\" ");
    }
    return artists_str;
}

fn track_str(t: &FullTrack) -> String {
    let artists_str = artist_slice_str(&t.artists);
    return format!("\"{}\" by {}on \"{}\"", t.name, artists_str, t.album.name);
}

fn item_str(i: &PlayableItem) -> String {
    match i {
        PlayableItem::Track(t) => track_str(&t),
        PlayableItem::Episode(_e) => {
            std::unimplemented!("TODO: print podcast ifno")
        }
    }
}

fn __album_slug(a: FullAlbum) -> String {
    let mut out = String::new();
    let mut runtime = Duration::zero();
    for t in a.tracks.items {
        runtime = runtime + t.duration;
    }

    out.push_str(&a.name);
    out.push_str(&format!(" - {} ", a.release_date));
    out.push_str(&format!(" - {:?} ", a.album_type));
    out.push_str(&format!(
        " - {}:{:02}",
        runtime.num_minutes(),
        runtime.num_seconds() - runtime.num_minutes() * 60
    ));

    return out;
}

fn __album_str(a: FullAlbum) -> String {
    let mut runtime = Duration::zero();
    let mut tracks_block_str = String::new();
    for t in a.tracks.items {
        runtime = runtime + t.duration;
        tracks_block_str.push_str(&format!(
            "\n\t\t {} - {}:{:02}",
            t.name,
            t.duration.num_minutes(),
            t.duration.num_seconds() - t.duration.num_minutes() * 60
        ));
    }

    let mut out = String::new();
    out.push_str(&a.name);
    out.push_str(" - ");
    out.push_str(&artist_slice_str(&a.artists));
    out.push_str(&format!(
        " - {}:{:02}",
        runtime.num_minutes(),
        runtime.num_seconds() - runtime.num_minutes() * 60
    ));
    out.push_str(&format!("\n\tAlbum type: {:?}", a.album_type));
    out.push_str(&format!("\n\tRelease Date: {}", a.release_date));
    out.push_str(&format!("\n\tGenres: {:?}", a.genres));
    out.push_str(&tracks_block_str);
    return out;
}

async fn album_str(spotify: AuthCodeSpotify, simp_a: SimplifiedAlbum) -> String {
    let market = Market::Country(Country::UnitedStates);
    let album_req = spotify.album(simp_a.id.unwrap(), Some(market)).await;

    let a = match album_req {
        Ok(album) => album,
        Err(e) => panic!("Error {}", e),
    };

    return __album_str(a);
}

async fn artist_str(spotify: AuthCodeSpotify, simp_a: SimplifiedArtist) -> String {
    let market = Market::Country(Country::UnitedStates);
    let v = vec![AlbumType::Album, AlbumType::Single];
    let artist_req = spotify.artist(simp_a.id.clone().unwrap()).await;
    let artist_albums_req = spotify
        .artist_albums_manual(simp_a.id.unwrap(), v, Some(market), Some(32), Some(0))
        .await;

    let artist: FullArtist = match artist_req {
        Ok(artist) => artist,
        Err(e) => panic!("{}", e),
    };

    let simp_albums: Vec<SimplifiedAlbum> = match artist_albums_req {
        Ok(alst) => alst.items,
        Err(e) => panic!("{}", e),
    };

    let album_ids: Vec<AlbumId> = simp_albums
        .clone()
        .into_iter()
        .map(|alb| alb.id.unwrap())
        .collect();

    let albums_req = spotify.albums(album_ids.clone(), Some(market)).await;
    let albums: Vec<FullAlbum> = match albums_req {
        Ok(alst) => alst,
        Err(e) => panic!("bad albums request: {} \n album_ids: {:?}", e, album_ids),
    };

    let mut out = String::new();

    out.push_str(&format!("{} - {:?}", artist.name, artist.genres));
    for alb in albums {
        out.push_str(&format!("\n\t{}", __album_slug(alb)));
    }

    return out;
}

async fn print_info(spotify: AuthCodeSpotify, pcmd: &InfoCmd) {
    let market = Market::Country(Country::UnitedStates);
    let additional_types = [AdditionalType::Episode];
    let ctx_req = spotify
        .current_playing(Some(market), Some(&additional_types))
        .await;

    let itm = match ctx_req {
        Ok(Some(ctx)) => ctx.item.unwrap(),
        Ok(None) => panic!("Nothing currently playing"),
        Err(e) => {
            panic!("{}", e)
        }
    };
    let mut out = item_str(&itm);
    println!("{out}");
    out.clear();

    if pcmd.album || pcmd.artist {
        let t = match itm {
            PlayableItem::Track(t) => t,
            PlayableItem::Episode(_) => {
                panic!("poscasts not supproted for album and artist output")
            }
        };
        if pcmd.album {
            out.push_str("\n");
            out.push_str(&album_str(spotify.clone(), t.album).await);
        }

        if pcmd.artist {
            // Just assume artist 0 is the one you want...
            out.push_str("\n\n");
            out.push_str(&artist_str(spotify.clone(), t.artists[0].clone()).await);
        }
    }

    println!("{}", out);
}

fn modify_playback(spotify: AuthCodeSpotify, args: &PlayBackCmd) {
    println!("{:?} {:?}", args, spotify);
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
    // pause/play/toggle
    // skip/back
    // info of current artist
    //  artist albums -with len+year  info
    // start album playback
    // start album/artist/song radio

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
        Command::Info(c) => print_info(spotify, &c).await,
        Command::PlayBack(c) => modify_playback(spotify, &c),
    }
}
