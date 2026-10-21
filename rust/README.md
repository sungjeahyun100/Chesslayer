# Chesslayer: Rust migration

The old C++ files at the repository root remain untouched as a behavior reference.
The Rust workspace is under this directory so parity can be checked before cutover.

## Local commands

    cd rust
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    rustup target add wasm32-unknown-unknown
    cargo check --workspace --target wasm32-unknown-unknown

## Scope of this initial commit

The chesslayer-core crate implements the shared board/identity representation,
the module interface and the six common default-action effects, including
atomic rollback. It does NOT yet implement a Chessembly parser/interpreter,
ordinary chess rules, Chesslayer rules, WebAssembly bindings or a web UI.

See ../docs/RUST-MIGRATION.md for the interfaces, parity gates and build targets.
