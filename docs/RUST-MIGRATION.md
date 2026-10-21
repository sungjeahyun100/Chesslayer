# Chesslayer Rust migration: architecture contract

Status: **Phase 1 (shared core only)**. This document is intended to be
architecture-readable; implementation details live in Rust crates and tests.

## Decision

Implement a Rust engine that can run natively and compile to WebAssembly.
Keep the original C++ executable and tests as a reference until the parity
gates pass. Do not rewrite the reference in-place.

### Boundaries

| Component | Owns | Must NOT own |
| --- | --- | --- |
| chesslayer-core | Board geometry, piece identity, action effects, module interface | Checkmate, cards, UI |
| chessembly (next) | Parsing and evaluating the existing DSL into action candidates | Victory rules, mutation of input board |
| standard-chess (next) | FEN, orthodox movement/pawn rules, legal move filtering, game result, undo | Chesslayer roguelike |
| chesslayer-rules (next) | Royal capture, battle-specific piece behavior, card/encounter effects | Web hosting/UI |
| chesslayer-wasm (next) | Narrow JS/TS interface, serializable commands and events | Business rules |
| web (next) | UI, map, store, run flow, visual effects | Reimplementation of engine rules |

This is a **dependency direction**, not a requirement to write all crates now:
UI -> WASM adapter -> rule modules -> Chessembly and shared core.
The rules engine must not import or depend on React, GCP or DOM APIs.

## Frozen cross-module contracts

- A board has arbitrary dimensions and an optional playable-cell mask.
- A piece has a numeric kind, owner and stable instance ID. Each module may
  define additional piece data with its own schema.
- Action effects have fixed meaning: Move, TakeMove, Take, TakeJump, Catch, Shift.
  A rule module can filter out an action; it cannot change its meaning.
- Multistep actions are ordered and **atomic**. Failed continuation rolls back
  all changed squares.
- The DSL is a candidate generator, not a legal-move judge; the selected ruleset
  decides legality and victory.
- The DSL color predicate addresses the piece executing the program, not the
  player whose turn it is.
- For Chesslayer specifically: starting player pieces are the orthodox initial
  chess set, the player's king is lost by **direct capture**, and an encounter is
  won by **direct capture** of its designated enemy royal piece. Do not inherit
  checkmate/stalemate as automatic Chesslayer victory rules.
- Slay-the-Spire-style map, shop, events, rewards and persistent run progression
  sit **above** a single battle; they are not generic movement-engine concepts.
- Random effects require explicit injected RNG state to enable reproducible runs.
  Do not hide randomness in process-global state.

## Integration plan

1. **Core contract (current PR).** Match board, identity and shared action effects.
   No special chess rules in the shared core.
2. **Chessembly port.** Migrate all documented expressions and their original
   intentional no-op behavior. Preserve emitted action order. Maintain bounds on
   instruction count and block depth.
3. **Standard-chess parity.** Match existing FEN, move legality, castling,
   en passant, promotions, draw rules, stable IDs and make/unmake behavior.
   Gate: initial-position perft depths 1-5 and existing Kiwipete/position-3 tests.
4. **Chesslayer battle rules.** Implement royal capture, battle setup and
   special-piece/card effects as a separate module. Keep standard-chess as a
   regression reference, not a gameplay dependency.
5. **WASM and front-end.** Export a small typed command/event interface.
   Keep both native and wasm builds; native is needed for server-side verification.
6. **Deployment.** Host static React assets and wasm on Firebase Hosting.
   Use Cloud Run only if cloud saves, authoritative validation or later
   multiplayer requires it. Never require a server round trip for every
   single-player move.

## Quality gates for AI-driven development

Every new rule/DSL feature requires at least one named deterministic test.
For migrations, compare both **generated actions and resulting state** against
the original C++ program on identical positions. Passing only a compiler check
does not establish semantic parity.

On every PR: formatting, Clippy, native tests, wasm compilation, changed-files
summary, passed/failed parity counters and explicitly unsupported behaviors.
Keep PRs small and scoped to one contract or rules subsystem. Do not silently
merge or replace the original C++ implementation.

## Open architecture decisions (not silently inferred)

- In-battle player action economy and deck/card timing.
- Enemy AI and encounter data schema.
- Precise interaction between the DSL and newly invented status effects.
- Whether server authority is needed for ranking or multiplayer.

These decisions belong to the project architect, not an implementation agent.
