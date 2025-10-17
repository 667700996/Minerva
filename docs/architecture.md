# Minerva Rust Workspace Overview

This document sketches the initial Rust workspace layout for the Minerva project.  
Each crate encapsulates a distinct subsystem from the project plan so that domain code remains isolated yet composable inside the orchestrator.

## Workspace Structure

```
Cargo.toml          # Workspace manifest
apps/minerva-cli    # Binary entry point for local orchestration/testing
crates/minerva-types
crates/minerva-controller
crates/minerva-vision
crates/minerva-engine
crates/minerva-orchestrator
crates/minerva-network
crates/minerva-ops
docs/architecture.md
```

## Crate Responsibilities

- **minerva-types**  
  Common types: board state, move semantics, configuration, time controls, telemetry, and domain events shared across other crates.

- **minerva-controller**  
  Emulator/ADB bridge. Abstracts device discovery, screen capture, input injection, and latency metrics. Exposes traits so multiple controller backends (emulator, physical device, mock) can coexist.

- **minerva-vision**  
  Board alignment and piece recognition pipeline. Starts with trait-based API for pluggable recognizers (template matching, CNN, remote inference). Produces structured board states compatible with `minerva-types`.

- **minerva-engine**  
  Game engine/search abstraction. Defines interfaces for incremental development from baseline alpha-beta search to NNUE/distributed implementations.

- **minerva-orchestrator**  
  Turn loop, synchronization, time management, and exception handling. Coordinates controller, vision, and engine crates with deterministic state machines.

- **minerva-network**  
  Networking server/client glue (WebSocket transport, event streaming, replay endpoints). Keeps protocol definitions near networking logic.

- **minerva-ops**  
  Logging/tracing, persistent telemetry, replay serialization, and operational tooling hooks.

- **minerva-cli**  
  Developer-facing binary for running the system locally. Loads configuration, wires dependencies, starts orchestrated matches, and now ships with a 터미널 UI(TUI) that streams lifecycle/엔진/텔레메트리 이벤트.

## Design Principles

- **Trait-first APIs** for each subsystem so high-fidelity implementations can replace early stubs without disrupting orchestrator contracts.
- **Message-driven orchestration** with strongly typed events to aid replay, testing, and distributed scaling.
- **Testable boundaries** using mock implementations from example/test modules, enabling incremental progress before full subsystem completion.
- **Config-driven wiring** leveraging `serde`/`toml` to mirror production environment constraints while allowing local overrides.

This skeleton provides the scaffolding needed to build the detailed functionality outlined in the project plan, while keeping compilation fast and the codebase navigable as features grow.
