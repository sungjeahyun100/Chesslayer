# C++ / Rust Chessembly differential check

From the repository root:

```sh
g++ -std=c++20 -I. rust/parity/cpp_dump.cpp chessembly.cpp -o /tmp/chesslayer-cpp-parity
/tmp/chesslayer-cpp-parity > /tmp/chesslayer-cpp-parity.out
(cd rust && cargo run --quiet -p chessembly --example parity_dump) > /tmp/chesslayer-rust-parity.out
diff -u /tmp/chesslayer-cpp-parity.out /tmp/chesslayer-rust-parity.out
```

The 14 fixtures compare ordered action effect, source, capture and destination coordinates, transition name, action application success, and every resulting board cell's piece ID, type, and owner. Empty C++ cells use owner -1; the Rust dump normalizes `None` to this format for comparison. The original C++ `interpreter_tests` and Rust integration tests cover additional conditions, bounds, mask behavior, rollback and determinism.
