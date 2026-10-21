# Chesslayer Rust engine status

The Rust workspace now contains the shared core, Chessembly and Chesslayer battle rules. The C++ sources remain as a behavior reference. StandardChess is not a Rust migration target, and no standard chess victory condition is used by Chesslayer.

Dependency direction: future UI or WASM adapter → Chesslayer rules → Chessembly → core. The battle module exposes setup inputs, ordered movement candidates, action submission, result, and persistent player-piece output. It does not depend on a browser runtime.

The rules that are already fixed are direct capture of the player's king for defeat and direct capture of the encounter's designated enemy royal piece for victory, even if it is not a king. Checkmate and stalemate do not end a battle. Enemy placement overlapping a player piece removes that player piece during battle setup. Card effects have a separate entry point and do not change the piece turn.

Undefined systems remain outside this implementation: deck/mana/draw, individual card effects, enemy AI, encounter progression, rewards, map, shops, rest choices, and exact status/upgrades behavior. The battle input/output allows these systems to be connected later. See `rust/README.md` for current basic movement and open special-rule decisions.
