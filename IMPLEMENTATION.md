# Nightingale client implementation

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
# Events
Nightingale sends events to the clients via the WebSocket gateway, all events have the following

