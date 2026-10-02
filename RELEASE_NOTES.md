# konnect-codex v0.13.0-codex.1 — native power, Codex discipline

Konnect v0.13.0 considerably expands what the server can observe and enforce.
This companion release turns those capabilities into a Codex-native workflow:
small explicit mutations, exact live-board readback, structured evidence, and
clear stop conditions instead of optimistic automation.

It is reviewed specifically for
[Konnect v0.13.0](https://github.com/mixelpixx/Konnect/releases/tag/v0.13.0) at
commit `6bbe3e4f890ba1d37c0e5d5f38ccd03d90958c9e`.

## Highlights

- **Placement that proves what happened.** Whole-board planners are diagnostic.
  Codex now preserves the held set, applies only small named move batches,
  reads the exact live board back, and re-runs placement scoring and DRC.
- **Real stackup evidence.** `get_board_stackup` replaces layer-count guessing
  for layout-physics review while keeping stackup mutation explicitly out of
  scope.
- **Placed 3D models under live IPC.** `set_placed_footprint_models` is exposed
  through guidance and protected by the live-board ownership hook.
- **Assembly coordinates from geometry.** JLCPCB output uses a stable live
  snapshot and native pad-box midpoints. Legacy anchor-offset workarounds must
  be reviewed rather than carried forward blindly.
- **Bus-aware schematics.** The old issue #328 warning is retired. Batch wiring,
  power placement, and field creation adopt the new v0.13 contracts.
- **No regression in complete-board routing.** Native Specctra routing remains
  first. The non-overwriting KiCad-native Freerouting bridge remains available
  until the representative roundrect/unnumbered-NPTH benchmark passes natively.

## Native v0.13.0 behavior integrated

- Added `edit_footprint_pad zone_connect` guidance for solid, thermal, none,
  and inherited zone behavior.
- Added explicit `board_source: "saved"` read-only inspection with source
  disclosure; saved evidence is never presented as live IPC evidence.
- Added actionable schematic-to-PCB diagnostics for unsupported footprint pads
  and symbol/footprint pad mismatches.
- Added exact indexed 3D-model inspection and mutation on placed footprints.
- Adopted the bounded AI-directed placement loop, decoupling associations,
  unproven-cap reporting, containment uncertainty, and blocked diagnostic plans.
- Added batch-connect stub length, direction, and label type.
- Added snapped power-symbol placement with observed-position readback.
- Added `create_missing` for custom fields with per-unit update/create counts.
- Adopted bus-aware connectivity and removed the v0.12.1 bus workaround.

## Companion policy retired or reduced

- Retired `bus-connectivity-workaround`; Konnect issue #328 is fixed.
- Removed duplicate legacy score-first/whole-plan placement instructions now
  owned by Konnect v0.13.0.
- Removed manual stackup inference where `get_board_stackup` supplies evidence.
- Removed JLCPCB anchor compensation as a default workflow.
- Kept issue #315 as an honest architecture boundary: `move_connected` remains
  refused, so a wired block move still requires explicit wire repair or managed
  reconstruction plus ERC/connectivity evidence.

## Codex-native controls retained

- Explicit workflow routing and sequential specialist-agent ownership.
- Schematic readability, local passive wiring, group closure, and visual review.
- Transfer, placement, route-import, manufacturing, and contradiction gates.
- BOM qualification, custom-part physical pin mapping, and bring-up handoff.
- PCB-layout-physics and visible placement acceptance beyond DRC.
- Process ownership and cleanup for KiCad, Java, Freerouting, and helpers.
- Separate Claude and Codex review paths as an intentional defect-finding
  control.

## Compatibility and verification

- Pins the exact 17-file upstream guidance baseline, aggregate guidance
  fingerprint, and native hook fingerprint.
- Targets Konnect's v0.13.0 catalogue of 228 registered tools, 235 total tools,
  and 21 toolsets with eager discovery enabled for Codex.
- Audits the Codex hook matcher against the reviewed v0.13.0 tool contract;
  `set_placed_footprint_models` is classified live-only and
  `get_board_stackup` is read-only.
- Validates formatting, Clippy, unit/integration/doc tests, exact-tag source
  audit, dry-run sync, installation, activation, and doctor output.
- The release remains version-strict: a missing or non-v0.13.0 Konnect binary
  stops before plugin files are changed.

## Installation

Install Konnect v0.13.0 first, then install this matching companion release.
Run `konnect-codex sync` followed by `konnect-codex doctor`, then start a new
Codex task so the refreshed skills, agents, hooks, and MCP configuration load.
