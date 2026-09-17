# konnect-codex plugin v0.12.0 - companion revision 1

This release is reviewed specifically for
[Konnect v0.12.0](https://github.com/mixelpixx/Konnect/releases/tag/v0.12.0) at
commit `37cfa434848120321906e87bd30553c4f0f08244`.

## Upstream v0.12.0 integration

- Reviewed all nine changed upstream skill and agent assets and rebased the
  exact 17-file baseline, aggregate guidance fingerprint, and hook contract.
- Added Codex guidance for `get_footprint_info(include_pads=true)` and
  `get_component_pads` so library-local and board-space pad geometry are not
  confused during routing.
- Integrated DRC `owner` and `ownership_status` evidence so footprint-owned
  edge geometry is not incorrectly "fixed" by moving a component.
- Integrated repeated-sheet local/global/power-net scope and the structured
  `annotate_schematic` partial/unresolved contract.
- Integrated the verified manufacturing artifact manifest and JLCPCB BOM/CPL
  correction and mandatory placement-preview workflow. Structural completion
  is never presented as physical-orientation approval.

## Companion behavior retained

- Preserved the non-overwriting KiCad-native Python/JAR Freerouting bridge.
  Konnect's Specctra exporter and bridge-selection code did not change in
  v0.12.0, so the representative roundrect/unnumbered-NPTH failure boundary
  remains. The workflow uses native routing when the actual-board preflight
  passes and the companion fallback only when that profile rejects legal KiCad
  geometry.
- Preserved deterministic specialist routing, schematic readability and group
  closure, transfer integrity, visual placement, PCB physics, BOM,
  manufacturing, review, bring-up, process-lifecycle, and benchmark-ledger
  controls.
- Preserved separate Claude and Codex guidance paths. Upstream behavior is
  translated and enhanced for Codex rather than copied with incompatible
  frontmatter or model settings.

## Known upstream packaging gap

Konnect v0.12.0's released `konnect` skill and two agents link
`references/reliability-contract.md`, but that file is absent from the release
asset tree. This companion continues to ship its independent evidence contract
and does not claim the missing upstream reference was loaded.

## Compatibility evidence

- Exact supported Konnect version and tag commit are pinned in
  `compatibility.json`.
- All upstream baseline files, the aggregate guidance fingerprint, and the hook
  fingerprint are pinned and checked by `konnect-codex audit`.
- The complete Konnect catalogue remains exposed through the plugin: 226
  registered tools, 233 total tools, and 21 toolsets for the supported release.
- Release validation covers formatting, unit and integration tests, Clippy,
  source audit, dry-run publication, plugin sync/doctor, and local activation.

## Installation

Install Konnect v0.12.0 first, then install this matching companion release.
Run `konnect-codex sync` followed by `konnect-codex doctor`. Start a new Codex
task after installation so the refreshed skills and agents are loaded.
