# Nightingale client implementation

> [!NOTE]
> Types ending in `?` mean they are optional

# WebSocket Gateway


## Opening a connection
Before starting to work with the server, the client must open a WebSocket connection,
this is done against the path `/ws`

The request must contain the following queries:

| Query name | Data type | Explanation                  |
|------------|-----------|------------------------------|
| `shards`   | `Integer` | Number of shards the bot has |
| `user_id`  | `Integer` | User id of the bot           |

After a connection is established, the server will send a [Ready](#ready) event. In this case, the `players` field will be empty.

## Resuming a connection
If for a reason, the client disconnects from the server, the session can be resumed
within a timespan, to do this, open a connection against the path `/ws/resume`

This request only requires a single header:
``
session: Integer. The session id of the session to resume, this is the id received in the Ready event
``

After successfully resuming a session, the server will send a [Ready](#ready) event. In this case, the `field` players will
contain all the players that are present in the server, this can be used to synchronize players with the client after resuming
sessions.

# Incoming Events
Nightingale sends events to the clients via the WebSocket gateway, all events have the following
structure:
````json
{
  "op": <opcode>,
  "data": object
}
````

## Op codes

### Ready
When a client connects to the server, Nightingale will send a `ready` event, as its name says,
this event corresponds to the `ready` opcode. The structure of this event is the following:

| Field     | Data type                                | Explanation                                 |
|-----------|------------------------------------------|---------------------------------------------|
| `session` | `Uuid`                                   | The identifier assigned to this session     |
| `resumed` | `Boolean`                                | Whether the session has been resumed or not |
| `players` | [Player](#getting-player-information)[ ] | Players present on the server               |

<details>
<summary>Example payload</summary>

````json
{
  "op": "ready",
  "data": {
    "session": "ad13c35f-7bf4-413b-997d-eef2fe009f98",
    "resumed": false
  }
}
````
</details>

### Forward
Nightingale forwards payloads to the client that should be forwarded to discord gateway,
these payloads are used to connect/disconnect to voice channels and to update microphone activity of
the bot. These messages have the `forward` opcode, the structure is the following:

| Field     | Data type | Explanation                               |
|-----------|-----------|-------------------------------------------|
| `shard`   | `Integer` | The shard that should forward the payload |
| `payload` | `Object`  | The payload that should be forwarder      |


<details>
<summary>Example forward payload</summary>

````json
{
  "op": "forward",
  "data": {
    "shard": 1,
    "payload": {
      "op": 4,
      "d": {
        "channel_id": <Channel_id>,
        "guild_id": <Guild_id>,
        "self_deaf": true,
        "self_mute": false
      }
    }
  }
}
````
</details>

### Update State
Update state events are sent when Nightingale successfully connects, disconnects or reconnects to a voice channel,
these correspond to the `update_state` opcode.

Update state payloads have the following structure:
````json
{
  "type": <State Update Type>,
  "data": object
}
````

Update state types are the following have the following fields:

- Connect Gateway (type: `connect_gateway`) and Reconnect Gateway (type: `reconnect_gateway`):

| Field        | Data type  |
|--------------|------------|
| `channel_id` | `Integer?` |
| `guild_id`   | `Integer`  |
| `session_id` | `String`   |
| `server`     | `String`   |
| `ssrc`       | `Integer`  |

<details>
<summary>Example payload</summary>

```json
{
  "op": "update_state",
  "data": {
    "type": "connect_gateway",
    "data": {
      "channel_id": <Channel_id>,
      "guild_id": <Guild_id>,
      "session_id": <Session>,
      "server": <Server>,
      "ssrc": <Ssrc>
    }
  }
}
```
</details>


- Disconnect Gateway (type: `disconnect_gateway`):

| Field        | Data type  |
|--------------|------------|
| `channel_id` | `Integer?` |
| `guild_id`   | `Integer`  |
| `session_id` | `String`   |

<details>
<summary>Example payload</summary>

```json
{
  "op": "update_state",
  "data": {
    "type": "disconnect_gateway",
    "data": {
      "channel_id": <Channel_id>,
      "guild_id": <Guild_id>,
      "session_id": <Session>
    }
  }
}
```
</details>

### Event
Nightingale sends track related events under the opcode `event`.

All events have the following structure:

| Field      | Data type     | Explanation                     |
|------------|---------------|---------------------------------|
| `guild_id` | `Integer`     | The guild the event occurred on |
| `event`    | `EventObject` | The event object                |

Where `EventObject` is:

| Field  | Data type |
|--------|-----------|
| `type` | `String`  |
| `data` | `object`  |

<details>
<summary>Example Payload</summary>

````json
{
  "op": "event",
  "data": {
    "guild_id": <Guild Id>,
    "event": {
      "type": "track_start",
      "data": <Track Object>
    }
  }
}
````
</details>

There are 3 different track events:

- Track Start(type: `track_start`)

| Field  | Data type |
|--------|-----------|
| `data` | `Track`   |

<details>
<summary>Example payload</summary>

```json
{
  "op": "event",
  "data": {
    "guild_id": <Guild Id>,
    "event": {
      "type": "track_start",
      "data": <Track Object>
    }
  }
}
```
</details>

- Track End (type: `track_end`)

| Field     | Data type | Explanation                                 |
|-----------|-----------|---------------------------------------------|
| `stopped` | `Boolean` | Whether the track has been manually stopped |
| `track`   | `Track`   |                                             |

<details>
<summary>Example payload</summary>

```json
{
  "op": "event",
  "data": {
    "guild_id": <Guild Id>,
    "event": {
      "type": "track_end",
      "data": {
        "stopped": false,
        "track": <Track object>
      }
    }
  }
}
```
</details>

- Track Errored (type: `track_errored`)

| Field   | Data type | Explanation             |
|---------|-----------|-------------------------|
| `error` | `String`  | The error that occurred |
| `track` | `Track`   |                         |

<details>
<summary>Example payload</summary>

```json
{
  "op": "event",
  "data": {
    "guild_id": <Guild_Id>,
    "event": {
      "type": "track_errored",
      "data": {
        "error": "Something failed",
        "track": <Track object>
      }
    }
  }
}
```
</details>

### Track object
The track object has the following fields:

| Field        | Data type  |
|--------------|------------|
| `track`      | `String?`  |
| `artist`     | `String?`  |
| `album`      | `String?`  |
| `channel`    | `String?`  |
| `duration`   | `Integer?` |
| `source_url` | `String?`  |
| `title`      | `String?`  |
| `thumbnail`  | `String?`  |

> [!WARNING]
> `duration` field is in milliseconds


# Outgoing Events
Most interaction with Nightingale is done through the REST API, however, **voice state update** and
**voice server update** events are forwarded using the gateway.

To forward those events to Nightingale we will use the following opcodes and structures:

- Voice state update (opcode: `update_voice_state`)

| Field        | Data type               |
|--------------|-------------------------|
| `guild_id`   | `Integer?` or `String?` |
| `user_id`    | `Integer` or `String`   |
| `session_id` | `String`                |
| `channel_id` | `Integer?` or `String?` |

<details>
<summary>Example Payload</summary>

```json
{
  "op": "update_voice_state",
  "data": {
    "guild_id": <Guild_Id>,
    "user_id": <User_id>,
    "session_id": <Session_id>,
    "channel_id": <Channel_id>
  }
}
```
</details>

- Update voice server (opcode: `update_voice_server`)

| Field      | Data type             |
|------------|-----------------------|
| `endpoint` | `String?`             |
| `guild_id` | `Integer` or `String` |
| `token`    | `String`              |

<details>
<summary>Example Payload</summary>

```json
{
  "op": "update_voice_server",
  "data": {
    "endpoint": <Endpoint>,
    "guild_id": <Guild_id>,
    "token": <Token>,
  }
}
```
</details>

# REST API
Most interactions(such as managing playback) with Nightingale are done through the REST API.

Before making any requests, you must connect to the gateway and receive the [Ready](#ready) event,
because all requests need you to provide the session given on that payload.

# Non-session Specific API
The routes described on this section don't need the session received on the [Ready](#ready)

## Getting system information
To get the current information about the system, a `get` request against the path `/api/v1/info` must be done.
This route also accepts a trailing route path: a session id. If the session is provided, the `playback` field will only
reflect the session players, if not, it will contain all active sessions players.

The server responds with the following json object:

| Field      | Data type      |
|------------|----------------|
| `system`   | `SystemInfo`   |
| `playback` | `PlaybackInfo` |

**SystemInfo:**

| Field    | Data type    |
|----------|--------------|
| `cpu`    | `CpuInfo`    |
| `memory` | `MemoryInfo` |

**CpuInfo:**

| Field           | Data type    | Explanation                                    |
|-----------------|--------------|------------------------------------------------|
| `total_usage`   | `float`      | Total system usage (not process) in percentage |
| `process_usage` | `float`      | System usage of the process in percentage      |
| `cores`         | `CoreInfo[]` | Individual core information                    |

**CoreInfo:**

| Field         | Data type | Explanation                     |
|---------------|-----------|---------------------------------|
| `total_usage` | `float`   | Usage of the core in percentage |
| `frequency`   | `integer` | Core frequency in MHz           |


**Memory Info**

| Field            | Data type | Explanation                 |
|------------------|-----------|-----------------------------|
| `memory`         | `integer` | Memory usage (RSS) in bytes |
| `virtual_memory` | `integer` | Virtual memory in bytes     |

**PlaybackInfo:**

| Field     | Data type | Explanation                         |
|-----------|-----------|-------------------------------------|
| `players` | `integer` | Number of existing players          |
| `playing` | `integer` | Number of players currently playing |

<details>
<summary>Example payload</summary>

```json
{
    "system": {
        "cpu": {
            "total_usage": 11.812126,
            "process_usage": 0.0,
            "cores": [
                {
                    "total_usage": 15.017052,
                    "frequency": 3808
                }, ...
            ]
        },
        "memory": {
            "memory": 22024192,
            "virtual_memory": 10321920
        }
    },
    "playback": {
        "players": 0,
        "playing": 0
    }
}
```

</details>

## Searching from sources
As of today, only searching from youtube is supported

### Searching from youtube

### Searching for query results
To search for results on youtube, make a `get` request against the path `/api/v1/search/youtube/search` providing a
`query` query on the URL.

This route returns a list of Youtube specific track objects, which have the following fields:

### Youtube track
| Field       | Data type | Explanation                             |
|-------------|-----------|-----------------------------------------|
| `title`     | `String`  | The title of the track or video         |
| `author`    | `String?` | The author, if available                |
| `length`    | `Integer` | The length of the track in milliseconds |
| `video_id`  | `String`  | The Id of the video                     |
| `is_stream` | `Boolean` | Whether the video is a stream or not    |
| `url`       | `String`  | The URL of the video                    |
| `thumbnail` | `String`  | URL to the thumbnail of the video       |

<details>
<summary>Usage example</summary>

get request to `<HOST>/api/v1/search/youtube/search?query=never%20gonna%20give%20you%20up`
with the authorization header.

Response:
```json
[
    {
        "title": "Rick Astley - Never Gonna Give You Up (Official Music Video)",
        "author": "Rick Astley",
        "length": 213000,
        "video_id": "dQw4w9WgXcQ",
        "is_stream": false,
        "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        "thumbnail": "https://i.ytimg.com/vi/dQw4w9WgXcQ/maxresdefault.jpg"
    },
    {
        "title": "Rick Astley - Never Gonna Give You Up [Lyrics]",
        "author": "GlyphoricVibes",
        "length": 214000,
        "video_id": "QdezFxHfatw",
        "is_stream": false,
        "url": "https://www.youtube.com/watch?v=QdezFxHfatw",
        "thumbnail": "https://i.ytimg.com/vi/QdezFxHfatw/maxresdefault.jpg"
    },
    ...
]
```

</details>

### Getting tracks from a playlist
To get all the tracks from a playlist, make a `get` request against the path `/api/v1/search/youtube/playlist` providing
a `playlist` query with the playlist url or id.

This route returns a playlist object with the following fields:

| Field    | Data type                         | Explanation            |
|----------|-----------------------------------|------------------------|
| `name`   | `String`                          | Name of the playlist   |
| `tracks` | [YoutubeTrack](#Youtube-track)[ ] | Tracks of the playlist |


# Session Specific API
This section covers the part of the api that is session specific, all routes must be prefixed with `/api/v1/<session>`
where `<session>` is the session id received in the [Ready](#ready) event.

## Player related routes

### Updating player state
This endpoint is used both to connect and disconnect to voice channels. To use this route, a `patch` request must
be done against the path `/players/<guild_id>/update`. This route accepts an optional JSON body with the following fields:

| Field        | Data type               | Explanation                             |
|--------------|-------------------------|-----------------------------------------|
| `channel_id` | `String?` or `Integer?` | The channel id to connect to, if known. |
| `endpoint`   | `String`                | The endpoint to connect voice to.       |
| `session_id` | `String`                | The session if of the connection        |
| `token`      | `String`                | The token of the connection             |

The json body is optional, and if not provided, Nightingale will disconnect from an existing voice connection, or do
nothing if not connected.


### Playing tracks
As of now, Nightingale supports playing from an HTTP Stream, Youtube or file bytes natively. Otherwise, it will use
[yt-dlp] as a backend to retrieve the actual url of the stream, and then stream it, please note that this is slower since
a new [yt-dlp] process needs to be started each time, and it takes some more time than the natively supported sources.

To start playing from sources, a `post` request must be done against the path `/players/<guild_id>/play`,
providing the following JSON body:

| Field        | Data type    | Explanation                                                                          |
|--------------|--------------|--------------------------------------------------------------------------------------|
| `force_play` | `Boolean`    | Whether to force play the track, if set to `true`, it will start playing immediately |
| `source`     | `PlaySource` | The source of the track                                                              |

`PlaySource` has the following fields:

| Field  | Options                                                                                                   | Explanation                 |
|--------|-----------------------------------------------------------------------------------------------------------|-----------------------------|
| `type` | `"link"`, `http` or `"bytes"`                                                                             | The type of source provided |
| `data` | `PlayLink` if `type` is `"link"`, `PlayHttp` if `type` is `"http"` and `PlayBytes` if `type` is `"bytes"` | The actual source           |


`PlayLink` is a json object with the following fields:

| Field         | Data type                        | Explanation                                                                          |
|---------------|----------------------------------|--------------------------------------------------------------------------------------|
| `force_ytdlp` | `Boolean?` (defaults to `false`) | Whether to force Nightingale to use yt-dlp to play, if not, Nightingale will decide. |
| `link`        | `String`                         | Source link to play from.                                                            |

`PlayHttp` is a json object with the following fields:

| Field   | Data type                                              | Explanation            |
|---------|--------------------------------------------------------|------------------------|
| `track` | [Track?](#track-object) described at track start event | The track object       |
| `link`  | `String`                                               | The link of the stream |

`PlayBytes` is a json object with the following fields:

| Field   | Data type                                              | Explanation            |
|---------|--------------------------------------------------------|------------------------|
| `track` | [Track?](#track-object) described at track start event | The track object       |
| `bytes` | `ByteArray`                                            | The bytes of the track |


This endpoint returns a [Track](#track-object) object, the same as described at track start event.

### Pausing and resuming playback
To pause or resume playback, a `patch` request against the paths 
`/players/<guild_id>/pause` and `/players/<guild_id>/resume` respectively must be done.

### Modifying playback volume
To modify the volume, a `patch` request must be done against the path `/players/<guild_id>/volume/<new_volume>`
where `<new_volume>` is the new volume to set as a `8 bit integer`, and will accept values from 0 to 254. 
Please take into account that a value of 100 means a 100% volume, so be careful with the values used since it can lead to
some type of distortion.

### Getting player information
To get information about a player, make a `get` request against the path `/players/<guild_id>/info`. This route returns a
player object that represents the state of a player. The object has the following fields:

| Field               | Data type                 |
|---------------------|---------------------------|
| `guild_id`          | `Integer`                 |
| `channel_id`        | `Integer?`                |
| `paused`            | `Boolean`                 |
| `volume`            | `Integer` (from 0 to 254) |
| `currently_playing` | [Track](#track-object)?   |
| `queue`             | [Track](#track-object)[ ] |
