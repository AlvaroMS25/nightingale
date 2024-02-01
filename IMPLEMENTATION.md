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

After a connection is established, the server will send a [Ready](#ready) event.

## Resuming a connection
If for a reason, the client disconnects from the server, the session can be resumed
within a timespan, to do this, open a connection against the path `/ws/resume`

This request only requires a single header:
``
session: Integer. The session id of the session to resume, this is the id received in the Ready event
``

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
When a client connects to the server, nightingale will send a `ready` event, as its name says,
this event corresponds to the `ready` opcode. The structure of this event is the following:

| Field     | Data type | Explanation                                 |
|-----------|-----------|---------------------------------------------|
| `session` | `Uuid`    | The identifier assigned to this session     |
| `resumed` | `Boolean` | Whether the session has been resumed or not |

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
the bot. These messages have the `forward` opcode, and the part to be forwarded is under the `data` field.

<details>
<summary>Example forward payload</summary>

````json
{
  "op": "forward",
  "data": {
    "op": 4,
    "d": {
      "channel_id": <Channel_id>,
      "guild_id": <Guild_id>,
      "self_deaf": true,
      "self_mute": false
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

There are 3 different track events:

- Track Start(type: `track_start`)

| Field   | Data type |
|---------|-----------|
| `track` | `Track`   |

<details>
<summary>Example payload</summary>

```json
{
  "op": "event",
  "data": {
    "type": "track_start",
    "data": <Track object>
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
    "type": "track_end",
    "data": {
      "stopped": false,
      "track": <Track object>
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
    "type": "track_errored",
    "data": {
      "error": "Something failed",
      "track": <Track object>
    }
  }
}
```
</details>

The track object has the following fields:

| Field        | Data type   |
|--------------|-------------|
| `track`      | `String?`   |
| `artist`     | `String?`   |
| `album`      | `String?`   |
| `channel`    | `String?`   |
| `duration`   | `Duration?` |
| `source_url` | `String?`   |
| `title`      | `String?`   |
| `thumbnail`  | `String?`   |

Where `Duration`:

| Field   | Data type |
|---------|-----------|
| `secs`  | Integer   |
| `nanos` | Integer   |

> [!WARNING]
> Total duration is: `secs + to_secs(nanos)`


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
