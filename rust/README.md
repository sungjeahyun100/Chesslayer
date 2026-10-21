# Chesslayer Rust engine

The original C++ files remain at the repository root as a behavior reference.
The Rust workspace provides three crates:

- `chesslayer-core`: board, piece identity, shared atomic action effects and rule interface.
- `chessembly`: reusable parser and immutable-board candidate generator.
- `chesslayer-rules`: 8×8 battle setup, basic piece movement, turn changes, direct royal capture and persistent player pieces.

Run from this directory:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace --target wasm32-unknown-unknown
```

The battle API starts with `Battle::new(BattleStart { players, encounter })`. `Battle::standard_roster()` supplies the initial 16 player pieces. `legal_actions` returns ordered movement candidates, `apply_piece_action` validates and applies a selected candidate, and `outcome` returns surviving piece positions and persistent data for the next encounter. `apply_resolved_card_effect` accepts an effect validated by a future card system and does not advance the piece turn. It does not define card costs, decks or targeting.

Current basic movement includes one-square king moves, sliding rook/bishop/queen moves, knight jumps, pawn forward/diagonal capture and an initial two-square pawn move when clear. Castling, en passant, promotion, check restrictions and checkmate are not implemented as Chesslayer rules. See `parity/README.md` for the C++ differential check.

Open architecture decisions: enemy AI and encounter format beyond placements, card timing/costs/effects, status/upgrades' combat meaning, promotion and other special move rules, action economy beyond alternating piece moves, and whether a transformed king retains royal identity across encounters.
