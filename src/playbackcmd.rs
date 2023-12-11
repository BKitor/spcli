use crate::infocmd::print_info;
use crate::{InfoCmd, PlayBackCmd};
use rspotify::{
    model::{enums::types::AdditionalType, PlayableId},
    model::{Country, CurrentPlaybackContext, CurrentlyPlayingContext, Device, Market},
    prelude::OAuthClient,
    AuthCodeSpotify, ClientResult,
};
use std::vec::Vec;

// TODO: come up with better solution
// This plyas the last song played, but will loop on that single song context
// Idealy, would play the last 'album' context
async fn start_new_pb(spotify: &AuthCodeSpotify, dev_id: Option<&str>) -> ClientResult<()> {
    let req = spotify.current_user_recently_played(Some(1), None).await;

    let hist = match req {
        Ok(cp) => cp.items[0].clone(),
        Err(e) => panic!("req error {}", e),
    };

    let mut v: Vec<PlayableId> = Vec::new();
    let playid = PlayableId::Track(hist.track.id.unwrap());
    v.push(playid);
    spotify.start_uris_playback(v, dev_id, None, None).await
}

async fn get_cp_ctx(spotify: &AuthCodeSpotify) -> Option<CurrentlyPlayingContext> {
    let req = spotify
        .current_playing(None, None::<Vec<&AdditionalType>>)
        .await;

    match req {
        Ok(cp) => cp,
        Err(e) => panic!("req error {}", e),
    }
}

async fn get_pb_ctx(spotify: &AuthCodeSpotify) -> Option<CurrentPlaybackContext> {
    let market = Market::Country(Country::UnitedStates);
    let cpb_req = spotify
        .current_playback(Some(market), None::<Vec<&AdditionalType>>)
        .await;

    match cpb_req {
        Ok(pb_ctx) => pb_ctx,
        Err(e) => {
            panic!("Playback request Error: {}", e);
        }
    }
}

async fn get_devices(spotify: &AuthCodeSpotify) -> Vec<Device> {
    let req = spotify.device().await;
    match req {
        Ok(devs) => devs,
        Err(e) => panic!("bad devices req: {}", e),
    }
}

async fn next_track(spotify: &AuthCodeSpotify) {
    let req = spotify.next_track(None).await;
    match req {
        Ok(_) => {}
        Err(e) => panic!("req error: {}", e),
    }
}

async fn prev_track(spotify: &AuthCodeSpotify) {
    let req = spotify.previous_track(None).await;
    match req {
        Ok(_) => {}
        Err(e) => panic!("req error: {}", e),
    }
}

async fn toggle_playback(
    spotify: &AuthCodeSpotify,
    pb_ctx: Option<CurrentPlaybackContext>,
    devs: Vec<Device>,
) {
    let req: ClientResult<()> = match pb_ctx {
        Some(pb_ctx) => {
            if pb_ctx.is_playing {
                spotify.pause_playback(None).await
            } else {
                spotify.resume_playback(None, None).await
            }
        }
        None => start_new_pb(&spotify, devs[0].id.as_deref()).await,
    };

    match req {
        Ok(_) => {}
        Err(e) => {
            panic!("Toggle network error: {}", e);
        }
    };
}

fn nothing_set(args: &PlayBackCmd) -> bool {
    return !args.next && !args.prev && !args.toggle;
}

pub async fn modify_playback(spotify: &AuthCodeSpotify, args: &PlayBackCmd) {
    let pb_ctx = get_pb_ctx(&spotify).await;
    let devs = get_devices(&spotify).await;

    if args.prev && args.next {
        println!("ERROR: both prev and next specified, not sure what you expect...");
    } else if args.next {
        next_track(&spotify).await;
    } else if args.prev {
        prev_track(&spotify).await;
    }

    if args.toggle || nothing_set(args) {
        toggle_playback(&spotify, pb_ctx, devs).await;
    }

    let post_ctx = get_cp_ctx(&spotify).await;
    match post_ctx {
        Some(ctx) => {
            if ctx.is_playing {
                let icmd = InfoCmd {
                    song: true,
                    artist: false,
                    album: false,
                };
                print_info(spotify, &icmd).await;
            }
        }
        None => {}
    };
}
