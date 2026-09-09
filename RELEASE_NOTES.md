# konnect-codex plugin v0.11.1 - companion revision 2

This release is reviewed specifically for
[Konnect v0.11.1](https://github.com/mixelpixx/Konnect/releases/tag/v0.11.1) at
commit `eadbe451bb50eb51c6e42abcf6e0152f62bc1e13`.

## Revision 2 corrective release

- Restored the non-overwriting KiCad-native Python/JAR Freerouting bridge after
  the representative v0.11.1 benchmark proved that the native Rust DSN
  preflight rejects standard roundrect and unnumbered NPTH pads before its
  optional ActionPlugin bridge can be selected.
- Changed routing selection from an assumed native default to an executable
  actual-board preflight, native route when supported, companion route fallback
  when needed, and explicit `INCOMPLETE` when neither path works.
- Made closure-capable schematic grouping a hard completion requirement.
  Component metadata, bounded proximity, and symbol-only `move_region` no longer
  satisfy movable grouping.
- Added regression assertions for route fallback availability, known v0.11.1
  compatibility boundaries, grouping truthfulness, and contradictory guidance.
- Made benchmark ledgers durable completion evidence: every gate, failure,
  recovery, missing artifact, cleanup result, and terminal verdict is required.

## Revision 1 native integration

This historical section describes revision 1. Its router-retirement decisions
are superseded by the corrective release above.

- Replaced the companion-owned Python/JAR router with Konnect's native sequence:
  `check_freerouting`, `export_specctra_dsn`, `route_specctra_dsn`,
  `plan_specctra_ses_import`, and `apply_specctra_ses`.
- Retired the `offline-freerouting-bridge` companion delta and removed its CLI,
  process discovery, KiCad Python scripts, and tests.
- Retained Freerouting-first Codex guidance and strengthened its board-revision,
  manifest, supported-geometry, checkpoint, DRC, and cleanup gates.
- Added the live-board hook classification for native DSN export, SES planning,
  and SES apply.

## Workflow and guidance improvements

- Added the discoverable `kicad-workflows` router and domain-owned workflows for
  custom libraries, schematic construction and cleanup, PCB transfer,
  constraints, placement, routing, ECOs, recovery, simulation readiness,
  manufacturing release, and bring-up.
- Added an owned-process lifecycle gate for KiCad editors, Freerouting, Java,
  simulators, and external helpers. It distinguishes pre-existing processes from
  task-owned children and requires verified cleanup without broad termination.
- Added a routed-board physics acceptance gate covering return planes, critical
  loops, power and thermal paths, differential and RF constraints, stitching,
  via process, edge clearance, and production test access.
- Preserved the existing Codex-specific delegation, schematic readability,
  transfer integrity, evidence honesty, placement, BOM, manufacturing, review,
  and bring-up controls.

## Upstream v0.11.1 review

- Rebased the exact upstream skill and agent baseline on the v0.11.1 tag and
  recorded all normalized file hashes plus aggregate guidance and hook
  fingerprints.
- Integrated upstream physical pin-map, schematic evidence, manufacturing, and
  native Freerouting guidance without copying Claude-only frontmatter or model
  settings into Codex agents.
- Retained the documented #315 connected-move and #328 bus-connectivity safety
  boundaries where the upstream release still requires them.

## Compatibility evidence

- Exact Konnect version, tag commit, per-file upstream asset hashes, aggregate
  guidance fingerprint, and hook fingerprint are pinned.
- Policy assertions cover 26 active enhancements and three retired decisions.
- Release validation includes source audit, formatting, tests, Clippy, plugin
  sync/doctor, local process cleanup, and platform packaging.
