# konnect-codex plugin v0.11.0 — companion revision 1

This release is reviewed specifically for
[Konnect v0.11.0](https://github.com/mixelpixx/Konnect/releases/tag/v0.11.0) at
commit `a22ad2153dcf45dbcf1cc63b5b0f1e40c93d7956`.

## Upstream integration

- Integrated `delete_graphics` into the PCB outline-replacement workflow and
  classified it as a live-or-revision-aware-closed-board mutation.
- Integrated `set_predefined_sizes` / `get_predefined_sizes` while keeping the
  editor palette distinct from DRC floors and netclass targets.
- Updated netclass guidance for resolved values, `inherits`, and
  `missing_fields`.
- Added explicit placement `held`-set inspection and rejection of unexpected
  held references while retaining independent post-apply scoring.
- Removed the #331 official-footprint-refresh workaround and the #326 incomplete
  Default-netclass workaround; both are fixed in v0.11.0.
- Retained the #328 bus-connectivity and #315 `move_connected` safety gates.

## Retired companion deltas

- `native-auto-install-suppression` is retired. Konnect v0.11.0 MCP startup is
  non-mutating and guidance installation requires explicit `konnect init`.
  Sync migrates old installations by removing the companion-owned guard and
  only the synthetic marker that guard recorded as companion-created.
- `verified-symbol-and-pin-guidance` is retired as a companion-only correction.
  Upstream v0.11.0 corrected unsafe universal pin rules, known invalid library
  IDs, and LED polarity examples and added asset tests. The safe wording remains
  in the Codex translation.

## Deliberately retained

- The offline Freerouting bridge and Freerouting-first workflow remain active.
  The native DSN/SES/MCP stack in Konnect PRs #338, #339, #340, and #342 has not
  shipped in v0.11.0.
- Codex-native delegation, schematic readability, transfer-integrity,
  evidence-honesty, placement, contradictory-verdict, BOM, manufacturing,
  review, and bring-up gates remain active.
- The Codex hook contract remains active. It now covers `delete_graphics` and
  `set_predefined_sizes`; upstream Claude issues #357 and #358 remain open.

## Compatibility evidence

- Exact Konnect version, tag commit, aggregate guidance fingerprint, unchanged
  hook fingerprint, and all 17 normalized upstream asset hashes are pinned.
- Policy assertions cover 24 active enhancements and record two retired
  decisions in the living delta register.
- Release validation includes source audit, formatting, tests, Clippy,
  lifecycle migration, plugin sync/doctor, and platform packaging.
