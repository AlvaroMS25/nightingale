# Nightingale

An audio provider node based on [Songbird]

Nightingale does **not** aim to replace [Lavalink], instead it aims to be another option 
as lightweight and as feature rich as possible, so users can choose the one that fits their
needs better.

## Features
- Powered by [Songbird]
- Minimal resource consumption
- Nightingale can play from any source supported by [yt-dlp]
- WebSocket gateway for event-related communication
- REST API for controlling Nightingale players and connections such as changing volume, and more...
- Password authentication
- IP Whitelisting

## Roadmap
- [ ] Allow forcing tracks to play at arrival
- [ ] Playback seeking route
- [ ] Playback skip route
- [ ] Player information route
- [ ] Clear queue route
- [ ] System information such as CPU and Memory usage
- [ ] Prometheus Metrics
- [ ] Allow writing logs to file
- [ ] Routes for searching from different sources
- [ ] Allow adding playlists directly

## Usage
Nightingale requires a ``nightingale.yml`` configuration file, which has the following options:

[Songbird]: https://github.com/serenity-rs/songbird
[Lavalink]: https://github.com/lavalink-devs/Lavalink
[yt-dlp]: https://github.com/yt-dlp/yt-dlp
