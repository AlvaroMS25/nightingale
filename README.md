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
Nightingale requires a ``nightingale.yml`` configuration file, which has the following structure:

````yaml
server:
  address: "127.0.0.1"
  port: 8081
  password: "incredibly hard password"
  # ...
logging: # Optional field
  enable: true
  level: info
````


### Server
| Field      | Data type                      | Explanation                                             | Example        |
|------------|--------------------------------|---------------------------------------------------------|----------------|
| address    | `String`                       | The IP address the server will listen on                | `"127.0.0.1"`  |
| port       | `Integer`                      | The port the server will listen on                      | `8080`         |
| password   | `String`                       | The password used to authenticate on Nightingale routes | `"mypassword"` |
| http2      | `Boolean?` (default `false`)   | Whether if nightingale should use Http2                 | `false`        |
| ssl        | `SSlOptions?` (default `null`) | Options for nightingale to use SSL                      | \<Empty>       |
| filter_ips | `IpFilter?` (default `null`)   | Options to filter IPs that interact with Nightingale    | \<Empty>       |

#### SSlOptions
| Field     | Data type | Explanation                     | Example          |
|-----------|-----------|---------------------------------|------------------|
| enable    | `Boolean` | Whether to enable SSl           | `true`           |
| cert_path | `String`  | The path to the SSl certificate | `certs/cert.pem` |





[Songbird]: https://github.com/serenity-rs/songbird
[Lavalink]: https://github.com/lavalink-devs/Lavalink
[yt-dlp]: https://github.com/yt-dlp/yt-dlp
