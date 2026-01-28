# Offline English Dictionary Implementation Log

## Session: 2026-01-08

### Compilation Checks
- Started `cargo check`.
- **FAILED**: Dependency conflict `libsqlite3-sys` resolved by downgrading `stardict` to `=0.2.0`.
- **FAILED**: `stardict 0.2.0` API obscure (trait vs struct).
    - **Action**: Stubbed `StarDict` loading with `Option<Box<dyn StarDict>>` to allow compilation.
    - **Note**: Runtime dictionary loading currently disabled. Requires further research or custom parsing implementation.
- **FAILED**: `CommentId` type mismatch.
    - **FIXED**: Updated `comment.rs` to use `Json::<CommentId>`.
