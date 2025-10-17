# Minerva (Rust Workspace)

This repository contains the Rust implementation scaffold for the Minerva project—an Android emulator–driven Janggi bot.  
The workspace mirrors the architectural breakdown from `Minerva_Project_Details.md` using dedicated crates per subsystem.

## Workspace Layout

```
apps/minerva-cli           # Developer CLI entry point
crates/minerva-controller  # Emulator/ADB interaction layer
crates/minerva-vision      # Board alignment and recognition pipeline
crates/minerva-engine      # Search/evaluation engine abstraction
crates/minerva-orchestrator# Turn/state orchestration
crates/minerva-network     # Realtime publish/subscribe facade
crates/minerva-ops         # Logging, telemetry, operational helpers
crates/minerva-types       # Shared domain types/config/errors
docs/architecture.md       # Extended crate responsibilities
```

## Getting Started

1. Ensure Rust (1.75+) and Cargo are installed.
2. Fetch dependencies and verify the workspace compiles:

   ```bash
   cargo check
   ```

3. Run the unit tests:

   ```bash
   cargo test
   ```

4. Run the development CLI (currently a mock pipeline). Configuration is read from TOML; by default `configs/dev.toml` is used, or pass a different path / set `MINERVA_CONFIG`.

   ```bash
   cargo run -p minerva-cli -- configs/dev.toml
   ```

   The CLI boots the orchestrator with mock components, providing a starting point for integrating real controller, vision, and engine backends.

## Next Steps

- Replace `MockController`, `TemplateMatchingRecognizer`, and `NullEngine` with production implementations.
- Flesh out telemetry persistence and networking protocols.
- Expand validation and error handling around configuration I/O.
- Add unit/integration tests per crate as subsystems mature.
