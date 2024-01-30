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
  address: 127.0.0.1
  port: 8081
  password: mypasword
  # ...
logging: # Optional field
  enable: true
  level: info
````
<br>

> [!NOTE]
> Types marked as ``?`` mean are not required but optional

### Server
| Field      | Data type                      | Explanation                                             | Example                  |
|------------|--------------------------------|---------------------------------------------------------|--------------------------|
| address    | `String`                       | The IP address the server will listen on                | `127.0.0.1` or `"[::1]"` |
| port       | `Integer`                      | The port the server will listen on                      | `8080`                   |
| password   | `String`                       | The password used to authenticate on Nightingale routes | `mypassword`             |
| http2      | `Boolean?` (default `false`)   | Whether if nightingale should use Http2                 | `false`                  |
| ssl        | `SSlOptions?` (default `null`) | Options for nightingale to use SSL                      | \<Empty>                 |
| filter_ips | `IpFilter?` (default `null`)   | Options to filter IPs that interact with Nightingale    | \<Empty>                 |

#### SSlOptions
| Field       | Data type                    | Explanation                                                 | Example          |
|-------------|------------------------------|-------------------------------------------------------------|------------------|
| enable      | `Boolean`                    | Whether to enable SSl                                       | `true`           |
| cert_path   | `String`                     | The path to the SSl certificate                             | `certs/cert.pem` |
| key_path    | `String`                     | The path to the key of the certificate                      | `certs/key.pem`  |
| use_openssl | `Boolean?` (default `false`) | Whether to use openssl, if not, nightingale will use rustls | `false`          |

#### IpFilter
| Field | Data type | Explanation                                 | Example          |
|-------|-----------|---------------------------------------------|------------------|
| v4    | `Ip/Mask` | The v4 IPs allowed to connect to the server | `192.168.0.0/24` |
| v6    | `Ip/Mask` | The v6 IPs allowed to connect to the server | `fd00::/32`      |

### Logging
| Field  | Data type                         | Explanation                      | Example |
|--------|-----------------------------------|----------------------------------|---------|
| enable | `Boolean`                         | Whether to enable logging or not | `true`  |
| level  | `LoggingLevel?` (defaults `info`) | The level of logging to use      | `info`  |

`LoggingLevel` consists of 5 options: `error`, `warn`, `info`, `debug` and `trace`. Ordered from more to less critical.








[Songbird]: https://github.com/serenity-rs/songbird
[Lavalink]: https://github.com/lavalink-devs/Lavalink
[yt-dlp]: https://github.com/yt-dlp/yt-dlp
