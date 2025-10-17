# Android Emulator-Based Janggi Bot System Development Plan
> Codename: Minerva  
> Objective: Reach at least amateur 1-dan strength and scale up to professional 9-dan via iterative upgrades. Deliver a “local bot vs. remote human” live service.

--------------------------------------------------------------------------------

## 1. Project Overview

1) Project Name  
- Codename: Minerva

2) Final Deliverables  
- (A) Autonomous match-playing agent that controls the emulator Janggi app via ADB  
- (B) Real-time spectating/playing/review front-end (web/mobile)  
- (C) Analytics pipeline (logs/replays/performance dashboard)

3) Key Performance Indicators (KPI)  
- Elo/Win Rate: Surpass stable amateur 1-dan → top amateur tier (>75%) → scalable to pro level  
- Latency: Observation → decision → input roundtrip between 250–1500 ms depending on mode  
- Recognition Accuracy: ≥99% on board/piece state with production UI & animations  
- Scalability: Expandable to 9-dan via NNUE, distributed search, endgame tablebases, opening books

--------------------------------------------------------------------------------

## 2. Top-Level Architecture

- Emulator (Janggi app) ←ADB I/O→ Controller (input injection & screen capture)  
- Controller → Vision (board alignment & piece recognition) → Engine (search & evaluation)  
- Orchestrator (turn loop, synchronization, time management) ↔ Net (Server/WebSocket) ↔ Client (UI)  
- Ops (logging, replay, dashboard, CI/CD)

--------------------------------------------------------------------------------

## 3. ADB Connection/Input Protocol (default: 127.0.0.1:5555)

1) Device Setup  
- `adb kill-server && adb start-server`  
- `adb connect 127.0.0.1:5555`  
- `adb devices -l` → expect `127.0.0.1:5555` or `emulator-5554`

2) Screen Capture (choose one)  
- Single capture: `adb -s 127.0.0.1:5555 exec-out screencap -p > frame.png`  
- High speed: minicap socket stream (optional), optionally piped through ZSTD

3) Input Injection  
- Tap:  `adb -s 127.0.0.1:5555 shell input tap {x} {y}`  
- Drag: `adb -s ... shell input swipe {x1} {y1} {x2} {y2} {ms}`  
- Key:  `adb -s ... shell input keyevent {CODE}`

4) Stabilization Tips  
- Fixed-resolution AVD profile (stable navigation/status-bar layout)  
- Detect template animations/ads → auto-dismiss  
- Log FPS/latency from capture to input for average/peak monitoring

▶ Reference snippet (BlueStacks ADB AppleScript on the same port)
-- BlueStacks ADB automatic input AppleScript  
-- Command + . to stop  
```
property serial : "127.0.0.1:5555"
on sh(cmd)
  set envPATH to "PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
  do shell script envPATH & " " & cmd
end sh
on adb(cmd)
  sh("adb " & cmd)
end adb
on tapXY(x, y)
  adb("-s " & serial & " shell input tap " & x & " " & y)
end tapXY
```

--------------------------------------------------------------------------------

## 4. Vision (Board State Recognition) Design

1) Board Alignment (initial or automatic)  
- Anchor templates (logos/menu) → compute correction offsets  
- Or manual four-corner calibration → compute homography (H) for perspective correction  
- 9×10 grid mapping table: cached per file/profile

2) Piece Recognition Pipeline  
- V0 (fast PoC): tile-level template matching (characters/colors) + color histogram  
- V1 (robust): small CNN/ViT classifier (handles compression, blur, fonts, FX)  
- Incremental updates: re-evaluate only changed tiles between frames; re-capture if confidence < τ

3) Output Format  
- Internal state: bitboards/compact arrays  
- Exchange format: FEN-like string + side-to-move + castling/privileges aligned with Janggi rules

--------------------------------------------------------------------------------

## 5. Engine (Search & Evaluation) Design — scalable to pro 9-dan

1) Core Search  
- Iterative Deepening (1→N ply), Alpha-Beta + PVS  
- Transposition Table (TT, Zobrist hashing, N-way replacement)  
- Quiescence Search (checks/core captures), Aspiration Window  
- Move Ordering: PV → TT → checks/captures → history/killer → sorted  
- LMR/LMP, SEE, Null-Move Pruning (NMP), IID, Re-search

2) Evaluation Function  
- Material weights + positional/activity + king safety (palace control) + pawn/cannon/chariot/horse interactions  
- Phase interpolation (opening/midgame/endgame) with pattern/structure bonuses  
- Parameter tuning via SPSA/black-box optimization

3) Scaling Extensions (pro level)  
- NNUE/lightweight ViT evaluation (ONNX, INT8/AVX2/AVX-512)  
- Opening book (manual/self-play statistics), EGTB (priority on simple endgames)  
- Multi-core parallelism (YBWC), distributed search (cluster PV-split)  
- Time management: remaining time + increment-based adaptive depth + PV stability feedback

--------------------------------------------------------------------------------

## 6. Orchestration/Synchronization

1) Game Loop State Machine  
- READY → OUR_TURN → THINK → ACT (ADB) → VERIFY → OPP_TURN  
- VERIFY: capture after input → re-recognize → compare with expected state → retry/tap again if mismatch

2) Exception Recovery  
- Pop-up/advertisements: template detection → auto-close  
- Animations: wait until frame difference (norm) drops below threshold  
- Coordinate offset drift: re-detect anchors → rebuild grid

3) Time/Latency Management  
- Mode budgets: blitz (≤250 ms) / rapid (≤1000 ms) / classic (flexible)  
- Early stop signal during search → immediately commit best PV

--------------------------------------------------------------------------------

## 7. Networking/Client

1) Server  
- WebSocket real-time board/moves/clocks/metadata  
- Session/auth/spectating/replay APIs

2) Client (UI)  
- Board view, candidate moves, variation tree, commentary text, win-rate graph  
- Review: automatic tagging (blunder/mistake/inaccuracy), auto-generated commentary

--------------------------------------------------------------------------------

## 8. Operations/Quality/Security

1) Logging/Replay  
- Capture frame/alignment H/state/engine stats (NPS, depth, nodes) + input coordinates + latency (ms)  
- Replay engine: reproduce with same seeds/frames

2) Testing  
- Rule/legal move unit tests, recognition regression set (varied skins/resolutions), engine answer set  
- Self-play leagues (A/B/C settings), regression performance gating

3) Security  
- Restrict local ADB, firewall ports, authentication tokens (remote UI)

--------------------------------------------------------------------------------

## 9. Development Timeline (Milestones)

- M0 (Week 1): Kickoff, requirement freeze, environment setup, verify ADB connection (127.0.0.1:5555)  
- M1 (Weeks 2–3): Capture/input loop, fixed-coordinate tap success >99%, latency measurement  
- M2 (Weeks 4–5): Board alignment + template recognition V0 (accuracy >98%), state synchronization  
- M3 (Weeks 6–7): Engine V0 (AB+ID+TT+QSearch), reach amateur 1–3 dan  
- M4 (Weeks 8–9): LMR/NMP/SEE/ordering + time management, opening book V0 → top amateur tier  
- M5 (Weeks 10–12): NNUE/multi-threading/partial EGTB → near pro strength  
- M6 (Week 13+): Distributed search + expanded book/EGTB + large-scale self-play → pro 9-dan scale  

*Duration may vary depending on resources/data/app UI updates.*

--------------------------------------------------------------------------------

## 10. Environment/Standards

- ADB Port: 127.0.0.1:5555 (fixed), scale to 5556+ for multiple sessions  
- Resolution: 1080×1920 or 1440×2560  
- OS: Linux (engine), macOS/Windows (development), Docker support  
- Stack: C++ (engine) + Python (vision/orchestration) + TypeScript (front-end)  
- Inference: ONNX Runtime (optional), AVX2/AVX-512 optimizations

--------------------------------------------------------------------------------

## 11. Risks/Mitigations

- App UI updates → maintain multiple anchor templates/automated retraining pipeline  
- Frame drops/latency spikes → minicap/minitouch, threading/buffering  
- Mis-taps/pop-ups → template library, re-tap/re-capture/rollback scenarios

--------------------------------------------------------------------------------

## 12. Maintenance/Expansion

- Rule/app portability: isolate templates/rule tables  
- Performance upgrades: staged rollout from NNUE → distributed search → EGTB → self-learning  
- Continuous tuning via self-play data  
