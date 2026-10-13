# Online play on Windows (bo2mp)

Everyone plays through one **server** (`iw4l-master`): a relay and lobby list,
not a game server. It needs no game files and no GPU. The host's own game runs
the match; the server forwards packets and answers where friends are.

## Pick where the server runs

| Where | How | Friends need |
|---|---|---|
| **This PC** (port-forward) | `host-online.cmd` in the project folder. It makes the certificates once (`online\certs`) and serves on UDP 4433. Forward UDP 4433 on the router to this PC for friends outside the house. | your public IP, and `online\certs\ca.pem` |
| **A VPS** (no port-forward) | from a machine with the clone: `cargo xtask master install root@IP` ([`MASTER.md`](MASTER.md)) | the VPS IP, and the `iw4l-ca.pem` it minted |
| **A friend's PC** | they run `host-online.cmd` | their IP, and their `ca.pem` |

"P2P" in practice is the first row: one player's PC is the server for
everyone.

## Point the game at it: `online.cfg`

`run-bo2.cmd` writes a template the first time; fill in `server` and run it
again:

```ini
server=127.0.0.1:4433        # the server's address (this PC: 127.0.0.1)
ca=online\certs\ca.pem       # the server's ca.pem (relative to the project)
name=iw4l-prod               # the label its certificate carries (leave as is)
```

`run-bo2.cmd` turns these into `IW4L_MASTER_ADDR`, `IW4L_MASTER_SERVER_NAME` and
`IW4L_MASTER_CA_CERT` in `play\.env`. An empty `server` plays offline.

## Friends

* Each PC has an 8-digit **friend code** (Options > ACCOUNT, and FRIENDS on
  the main menu), made once and saved in `settings.cfg`.
* FRIENDS > ADD FRIEND: type a friend's code. Each lobby-list refresh (every
  15 s) asks the server about them: ONLINE, IN A LOBBY (map, mode, players),
  IN A MATCH or OFFLINE. Del removes one.
* The server knows a code only while its player is connected or browsed in
  the last 45 seconds; nothing is stored on disk.

## Hosting and joining

* **Host:** set up a lobby as usual and press START. With a server in
  `online.cfg`, the game opens a room on it and starts the match there; you
  load the map with everyone in the room. If the server cannot host (down,
  refused, or no room within 15 s) the match starts on this PC offline.
* **Join a friend:** FRIENDS, pick a friend who shows IN A LOBBY, press
  Enter. You load in when they press START.
* Every player needs the map's game installed (Black Ops II, Call of Duty 4,
  World at War, ...); the server only lets in players who have it.

## Not done yet

A lobby browser in the Black Ops II front end (open lobbies of non-friends),
and zombies through the server.
