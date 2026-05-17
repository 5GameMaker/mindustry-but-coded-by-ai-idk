# Rust Mindustry

Rust rewrite workspace for `D:\MDT\mindustry-upstream-v157.4`.

Rules:

- Source-of-truth baseline: `D:\MDT\mindustry-upstream-v157.4` only.
- `D:\MDT\mindustry-rust` is not used as a reference.
- Directory names intentionally mirror the Java upstream modules: `core`, `desktop`, `server`, `android`, `ios`, `annotations`, `tools`, `tests`.

Current first milestone: compile a minimal Rust workspace and begin with the network compatibility core (`core/src/mindustry/net`).

## Verify

```bash
cargo check --workspace --manifest-path "D:/MDT/rust mindustry/Cargo.toml"
cargo test --workspace --manifest-path "D:/MDT/rust mindustry/Cargo.toml"
```
