use crate::InfoCmd;
use chrono::Duration;
use rspotify::{
    model::{
        AdditionalType, AlbumId, AlbumType, Country, FullAlbum, FullArtist, FullTrack, Market,
        PlayableItem, SimplifiedAlbum, SimplifiedArtist,
    },
    prelude::*,
    AuthCodeSpotify,
};

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
    return format!("\"{}\" - {}- \"{}\"", t.name, artists_str, t.album.name);
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
            "\n\t {} - {}:{:02}",
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
    out.push_str(&format!(" - Album type: {:?}", a.album_type));
    out.push_str(&format!(" - Release Date: {}", a.release_date));
    out.push_str(&format!(" - Genres: {:?}", a.genres));
    out.push_str(&tracks_block_str);
    return out;
}

async fn album_str(spotify: &AuthCodeSpotify, simp_a: SimplifiedAlbum) -> String {
    let market = Market::Country(Country::UnitedStates);
    let album_req = spotify.album(simp_a.id.unwrap(), Some(market)).await;

    let a = match album_req {
        Ok(album) => album,
        Err(e) => panic!("Error {}", e),
    };

    return __album_str(a);
}

async fn artist_str(spotify: &AuthCodeSpotify, simp_a: SimplifiedArtist) -> String {
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

    out.push_str(&format!("{} - Genres: {:?}", artist.name, artist.genres));
    for alb in albums {
        out.push_str(&format!("\n\t{}", __album_slug(alb)));
    }

    return out;
}
fn item_str(i: &PlayableItem) -> String {
    match i {
        PlayableItem::Track(t) => track_str(&t),
        PlayableItem::Episode(_e) => {
            std::unimplemented!("TODO: print podcast ifno")
        }
    }
}

pub async fn print_info(spotify: &AuthCodeSpotify, pcmd: &InfoCmd) {
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
            out.push_str(&album_str(&spotify, t.album).await);
        }

        if pcmd.artist {
            // Just assume artist 0 is the one you want...
            out.push_str("\n\n");
            out.push_str(&artist_str(&spotify, t.artists[0].clone()).await);
        }
    }

    println!("{}", out);
}
