# konnect-codex plugin v0.12.1-codex.2 - companion revision 2

This release is reviewed specifically for
[Konnect v0.12.1](https://github.com/mixelpixx/Konnect/releases/tag/v0.12.1) at
commit `fa62e1ccb9eba359519bf8e3eab53a6cffeee33c`.

## Companion revision 2

- Require functional-block passives to be placed with their parent device and
  hand-wired with visible local topology instead of connected only by repeated
  labels.
- Require a structured schematic acceptance record covering blocks, local
  wiring, group closure, overlap/page checks, electrical checks, inspected
  renders, waivers, and verdict.
- Apply the full schematic readability gate independently in
  `konnect_design_reviewer`; clean ERC alone cannot approve a visually unusable
  schematic.
- Extend prompt-hook and release regression coverage so later refreshes cannot
  silently drop local-wiring or schematic-review acceptance.
- Add a managed functional-block relocation protocol for Konnect 0.12.1:
  inventory the complete closure, preflight expanded bounds, move exact symbols,
  reconstruct wires/labels/notes/graphics at one offset, clear the source
  region, and prove electrical and visual equivalence. Native one-object KiCad
  grouping remains unavailable and is reported distinctly.

## Upstream v0.12.1 integration

- Rebased the exact 17-file upstream guidance baseline and reviewed all three
  changed skill assets.
- Reclassified `flip_component` as live-IPC-preferred with a guarded
  closed-board fallback. KiCad 10.0.6+ now performs the native transform,
  including 3D-model offsets and rotations; older live endpoints fail closed
  with `unsupported_capability`.
- Integrated the safer placement contract: the force-directed planner is
  deprecated and diagnostic-only, while decoupling plans require exact
  capacitor references and refuse blocked, out-of-bounds, or non-improving
  application.
- Integrated atomic junction evidence for batch schematic placement through
  `junctions_added_count` and `junctions_pruned_count`.
- Confirmed that the reliability-contract reference missing from v0.12.0 is
  now present in the upstream release package.

## Companion behavior retained

- Preserved the non-overwriting KiCad-native Python/JAR Freerouting bridge.
  Konnect v0.12.1 still rejects representative legal roundrect and unnumbered
  NPTH geometry in its native Specctra preflight, so the actual-board workflow
  continues to prefer native routing and uses the companion fallback only for
  that measured compatibility boundary.
- Preserved deterministic specialist routing, schematic readability and group
  closure, transfer integrity, visual placement, PCB physics, BOM,
  manufacturing, review, bring-up, process-lifecycle, and benchmark-ledger
  controls.
- Preserved separate Claude and Codex guidance paths. Upstream behavior is
  translated and enhanced for Codex rather than copied with incompatible
  frontmatter, hooks, or model settings.

## Compatibility evidence

- Exact supported Konnect version and tag commit are pinned in
  `compatibility.json`.
- All upstream baseline files, the aggregate guidance fingerprint, and the hook
  fingerprint are pinned and checked by `konnect-codex audit`.
- Release validation covers formatting, unit and integration tests, Clippy,
  source audit, dry-run publication, plugin sync/doctor, and local activation.

## Installation

Install Konnect v0.12.1 first, then install this matching companion release.
Run `konnect-codex sync` followed by `konnect-codex doctor`. Start a new Codex
task after installation so the refreshed skills, agents, and hooks are loaded.
