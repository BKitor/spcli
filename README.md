# Yet another rust based spotify tool

Barebones tool for how I like to use Spotify.
I have been using spotifyd + spotify-tui, but spotify-tui is unmaintained.
So I fugred I'd roll my own. 

### Setup
1. Setup a new app through spotify's developer portal
2. create a .env file with RSPOTIFY_CLIENT_ID/RSPOTIFY_CLIENT_SECRET/RSPOTIFY_REDIRECT_URI



TODO:
Havn't added any playback functionality yet
  - Command to play an Album
  - Playback Device mgmt
  - Radio isn't a thing, 
    - Homebrew a playlist generation thing? 
    - Play the last track of and album/artist and see what happens?
  - Centralize config into a ~/.config/<whatever> file
