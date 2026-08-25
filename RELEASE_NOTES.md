# konnect-codex plugin v0.8.0 — companion revision 1

This release is reviewed for
[Konnect v0.8.0](https://github.com/mixelpixx/Konnect/releases/tag/v0.8.0) at
commit `dee8a27ce606f644cef0220deac98e88640d9b16`.

Konnect 0.8.0 is a large correctness release. It adds live IPC board reads and
zone creation, repairs footprints damaged by legacy synchronization, replaces
stacked rectangular board outlines, preserves complete ERC/DRC violation item
sets, validates schema parameters in CI, and fixes the schematic corruption,
UUID, library-table, view-return, datasheet-catalog, and idempotency defects
tracked in the v0.7.0 improvement backlog.

## Guidance compatibility review

The upstream guidance inventory remains 17 files. Three skills changed:

- `kicad-manufacture` removed two parameters that the implementation did not
  consume: `quantity` from `export_manufacturing_package` and `schematic` from
  `estimate_cost`;
- `kicad-pcb` documents the expanded `add_zone` contract, including name,
  priority, pad connection, live IPC behavior, and the warning attached to
  closed-file fallback;
- `konnect` now describes Freerouting as installation detection rather than an
  available autorouting tool.

The reviewed Codex assets carry each change. The stricter companion behavior is
unchanged: a live PCB phase stops on file fallback, whole-board routing uses the
companion's non-overwriting Freerouting DSN/SES bridge, and imported routing is
accepted only after inventory, unrouted, short, and direct DRC checks.

No Codex enhancement is retired solely because a server fix landed. Transfer
integrity, contradictory-verdict, placement, and live-board gates remain active
until a v0.8.0 end-to-end benchmark proves their recorded retirement conditions.
The native-guidance suppression remains necessary because upstream issue #242
is still open and MCP startup can still reinstall explicitly removed native
Codex skills.

## Included

- Nine reviewed Codex-native workflow skills, including the execution router,
  BOM qualification, and controlled firmware/bring-up handoff.
- Five specialist Codex agents for libraries, schematics, PCB construction,
  independent review, and bring-up planning.
- Exact Konnect v0.8.0 version, commit, guidance, hook, and per-file baseline
  verification before plugin state changes.
- Eager discovery of Konnect's 210-tool catalogue and Codex-native lifecycle
  hooks.
- Scoped MCP session inspection and cleanup with Windows child-process
  ownership, preventing stale Konnect processes from locking upgrades.
- Freerouting-first, non-overwriting whole-board routing with live/offline PCB
  ownership preflight.
- Custom-part physical pin-map acceptance, schematic readability gates,
  placement checkpoints, ECO and power-layout branches, evidence-grounded
  review, legacy/manual assembly guidance, and manufacturing verification.
- Downloadable Windows, Linux, and macOS release archives with SHA-256 sums.

For the easiest setup, use the README's
[Install with Codex](https://github.com/neusse/konnect-codex#install-with-codex)
request. Konnect and its original hardware workflows are created and maintained
by [mixelpixx](https://github.com/mixelpixx); this repository is an independent,
version-matched Codex plugin.
