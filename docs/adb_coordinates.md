# Minerva ADB Interaction Coordinates

This document tracks the emulator tap coordinates supplied for the Minerva onboarding flow and the in-game board grid.  
All coordinates presume a 1080×1920 layout captured in `assets/screenshots/`.

## Start Flow Buttons

| Step | Screenshot | Action | Coordinate (x, y) |
|------|------------|--------|-------------------|
| 1 | `assets/screenshots/start_request.png` | 신청 버튼 | (550, 1180) |
| 2 | `assets/screenshots/start_confirm_yes.png` | 예 버튼 | (280, 710) |
| 3 | `assets/screenshots/start_confirm_ok.png` | 확인 버튼 | (360, 750) |

## Formation Selection

Screenshot: `assets/screenshots/formation_selection.png`

| Formation | Coordinate (x, y) |
|-----------|-------------------|
| 마상마상 | (280, 560) |
| 상마상마 | (450, 560) |
| 마상상마 | (280, 620) |
| 상마마상 | (450, 620) |
| 확인 | (450, 680) |

## Board Grid

Screenshot: `assets/screenshots/board_layout.png`

- **File axis (x, left → right)**:  
  `40, 125, 200, 280, 360, 440, 520, 600, 680`
- **Rank axis (y, bottom → top)**:  
  `880, 800, 740, 670, 600, 530, 450, 380, 300, 240`

The playable intersections are the Cartesian product of the file and rank coordinates.  
For reference, grid labels follow the `xNyM` pattern (e.g., `x1y1 = (40, 880)`, `x9y10 = (680, 240)`).

These values are codified in `minerva-types` for reuse by controller routines that translate board squares into ADB tap targets.
