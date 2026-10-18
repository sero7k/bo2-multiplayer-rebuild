# A small dedicated server

`iw4l serve <zone>` runs a match with no local player (`IW4L_HEADLESS=1`).
A **server pack** is one map's gameplay half in a small file, so a server
hosts a map from a game it does not have installed.

## What a headless match reads

* **The map** (in the pack): `ClipCollision` (brushes, BSP, mesh tables,
  brush models, trigger hulls, static models' collision), spawns, entity
  string, intermission view, script brush models and structs, flag
  descriptors, bounds. Not its drawing, lighting, fog or effects.
* **Black Ops II's shared `.ff` zones** (no `.ipak` headless): scripts,
  tables, weapons, skeletons and hit boxes, animations, sound alias lengths,
  game settings, vehicles. Foreign maps already run on these through
  `attach_t6_rules`, so the map's own game is not needed.
* **Black Ops II's map zones** for its own maps (they are not packed).

## The pack

`<folder>/<game>/<map>.iw4lmap` (`server_pack`): `IW4LMAP\0`, version,
payload size (u32 LE), then the payload in LZ4. A cut-short or damaged file
is refused, never misread. Built from the operator's own installs, never
shipped:

```
iw4l pack <folder>
```

packs every listed multiplayer map from the other games, as a match reads
it, and prints each size and the total. Then, on the server:
`IW4L_GAMES="D:/games/Black Ops II" IW4L_SERVER_PACKS="D:/iw4l/packs"
iw4l serve iw4:mp_highrise`. A map with no installed zone loads from its
pack; an installed zone still wins.

## Size (estimates; `iw4l pack` measures)

About 1–6 MB per map after LZ4 (mesh tables dominate), against a zone of tens
to hundreds of MB plus the game's common zones. Black Ops II: `zone/all/*.ff`
only.

## Not done

* Guest guns whose game is missing on the server; MW2-native rules from a pack.
* Infinite Warfare and Black Ops III zombies maps (multiplayer only).
* Client stand-ins for guest guns; lobbies asking only for the map's game.
* `serve` hosting a room on `iw4l-master` (rotation, modes, bots, trickshot).
* Not yet run on a PC: `iw4l pack` over real installs, `serve` from a pack.
