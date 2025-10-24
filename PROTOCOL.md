# Protocol

> [!NOTE]
> The protocol is still being worked on, so nothing is entirely concrete.
> In addition, the documentation is still a work-in-progress.

The server is just a simple raw TCP server, where packets are all big endian. In addition, because some
packets are fairly complicated, only fields are length deliminated whereas entire packets are not.

Each packet begins with a one byte "tag", followed by the packet body. Because everything is intended
to be as simple as possible, there is no actual "header" to speak of aside from the tag.
If you would like to see a sample implementation of the packets in C, you can take a look at
[shared.h](clients/ti84/src/shared.h) in the TI84 client.

## Overview

| Tag                                | Name               | Direction       | Summary                                              |
| ---------------------------------- | ------------------ | --------------- | ---------------------------------------------------- |
| [`0x00`](#0x00---null)             | `NULL`             | Server → Client | Connection closed or invalid.                        |
| [`0x01`](#0x01---init)             | `INIT`             | Client → Server | Initialize client session.                           |
| [`0x02`](#0x02---confirmed)        | `CONFIRMED`        | Server → Client | Server acknowledgment with client and lobby info.    |
| [`0x03`](#0x03---lobby_event)      | `LOBBY_EVENT`      | Server → Client | Lobby action or status change.                       |
| [`0x04`](#0x04---waiting_status)   | `WAITING_STATUS`   | Client → Server | Updates readiness status.                            |
| [`0x05`](#0x05---round_loading)    | `ROUND_LOADING`    | Server → Client | Indicates round loading, includes lobby info.        |
| [`0x06`](#0x06---round)            | `ROUND`            | Server → Client | Round image and number sent to players.              |
| [`0x07`](#0x07---guess)            | `GUESS`            | Client → Server | Player submits a coordinate guess.                   |
| [`0x08`](#0x08---guessed)          | `GUESSED`          | Server → Client | Indicates a player has made a guess.                 |
| [`0x09`](#0x09---result)           | `RESULT`           | Server → Client | Sends round results and updated scores.              |
| [`0x0a`](#0x0a---request_game_end) | `REQUEST_GAME_END` | Client → Server | Requests to end the current game or return to lobby. |

## `0x00` - `NULL`

This packet signifies that something has gone wrong, or that whatever connection has been closed.
The server will never actually send this, so it's more of a technical safe guard.

## `0x01` - `INIT`

This packet is sent by the client to initialize itself. Until this is
sent, the client will be considered "inactive", and won't receive
anything from the server or be visible to other clients.

### Body

| Field     | Type                              | Size (bytes) | Description                |
| --------- | --------------------------------- | ------------ | -------------------------- |
| `options` | [`ClientOptions`](#clientoptions) | 20           | Client configuration data. |

## `0x02` - `CONFIRMED`

Sent by the server after successful `INIT`.

| Field     | Type                              | Size (bytes) | Description                         |
| --------- | --------------------------------- | ------------ | ----------------------------------- |
| `id`      | `uint32`                          | 4            | Assigned client ID.                 |
| `options` | [`ClientOptions`](#clientoptions) | 20           | Echo of initialized client options. |
| `lobby`   | [`LobbyClients`](#lobbyclients)   | variable     | Current lobby state.                |

## `0x03` - `LOBBY_EVENT`

Notifies all clients when a player joins, leaves, returns, or toggles ready state.

| Field    | Type                            | Size (bytes) | Description                             |
| -------- | ------------------------------- | ------------ | --------------------------------------- |
| `action` | `uint8`                         | 1            | `JOIN` = 0, `RETURN`, `LEAVE`, `READY`. |
| `user`   | `uint32`                        | 4            | Client ID affected.                     |
| `lobby`  | [`LobbyClients`](#lobbyclients) | variable     | Updated lobby snapshot.                 |

## `0x04` - `WAITING_STATUS`

Sent by client to indicate readiness.

| Field   | Type   | Size (bytes) | Description                     |
| ------- | ------ | ------------ | ------------------------------- |
| `ready` | `bool` | 1            | `true` if ready for next round. |

## `0x05` - `ROUND_LOADING`

Server notifies all players that a new round is loading.

| Field   | Type                            | Size (bytes) | Description                          |
| ------- | ------------------------------- | ------------ | ------------------------------------ |
| `lobby` | [`LobbyClients`](#lobbyclients) | variable     | Lobby state before the round begins. |

## `0x06` - `ROUND`

Server sends the round image and metadata.

| Field       | Type              | Size (bytes) | Description                             |
| ----------- | ----------------- | ------------ | --------------------------------------- |
| `number`    | `uint32`          | 4            | Round number.                           |
| `image_len` | `uint32`          | 4            | Length of the image payload.            |
| `image`     | `byte[image_len]` | variable     | Raw image data (width=320, height=240). |

## `0x07` - `GUESS`

Client submits guessed coordinates.

| Field         | Type                        | Size (bytes) | Description                |
| ------------- | --------------------------- | ------------ | -------------------------- |
| `coordinates` | [`Coordinate`](#coordinate) | 8            | Player’s guessed location. |

## `0x08` - `GUESSED`

Server notifies that a specific player has submitted their guess.

| Field    | Type     | Size (bytes) | Description                   |
| -------- | -------- | ------------ | ----------------------------- |
| `player` | `uint32` | 4            | ID of the player who guessed. |

## `0x09` - `RESULT`

Server sends round results.

| Field   | Type                      | Size (bytes) | Description                          |
| ------- | ------------------------- | ------------ | ------------------------------------ |
| `round` | [`RoundData`](#rounddata) | variable     | Results, scores, and player guesses. |

## `0x0A` - `REQUEST_GAME_END`

Sent by client to request returning to lobby or ending the game.  
No body.

## Data Structures

### `Coordinate`

| Field       | Type    | Size (bytes) | Description   |
| ----------- | ------- | ------------ | ------------- |
| `longitude` | `float` | 4            | X coordinate. |
| `latitude`  | `float` | 4            | Y coordinate. |

### `ClientOptions`

| Field   | Type       | Size (bytes) | Description                                                            |
| ------- | ---------- | ------------ | ---------------------------------------------------------------------- |
| `color` | `uint8`    | 1            | Player color (`RED` = 0, `GREEN`, `BLUE`, `CYAN`, `MAGENTA`, `YELLOW`) |
| `user`  | `char[16]` | 16           | Null-padded username string.                                           |

### `LobbyClients`

| Field     | Type                               | Size (bytes) | Description        |
| --------- | ---------------------------------- | ------------ | ------------------ |
| `len`     | `uint32`                           | 4            | Number of clients. |
| `clients` | [`LobbyClient[len]`](#lobbyclient) | variable     | Client list.       |

### `LobbyClient`

| Field     | Type                              | Size (bytes) | Description            |
| --------- | --------------------------------- | ------------ | ---------------------- |
| `id`      | `uint32`                          | 4            | Client ID.             |
| `ready`   | `bool`                            | 1            | Ready state.           |
| `options` | [`ClientOptions`](#clientoptions) | 20           | Client color and name. |

### `RoundData`

| Field         | Type                             | Size (bytes) | Description          |
| ------------- | -------------------------------- | ------------ | -------------------- |
| `number`      | `uint32`                         | 4            | Round number.        |
| `answer`      | [`Coordinate`](#coordinate)      | 8            | Correct coordinates. |
| `players_len` | `uint32`                         | 4            | Number of players.   |
| `players`     | [`Player[players_len]`](#player) | variable     | Player results.      |

### `Player`

| Field       | Type                        | Size (bytes) | Description               |
| ----------- | --------------------------- | ------------ | ------------------------- |
| `has_guess` | `bool`                      | 1            | Whether player guessed.   |
| `guess`     | [`Coordinate`](#coordinate) | 8            | Player’s guess.           |
| `points`    | `uint32`                    | 4            | Total points.             |
| `delta`     | `uint32`                    | 4            | Points gained this round. |
| `id`        | `uint32`                    | 4            | Player ID.                |
