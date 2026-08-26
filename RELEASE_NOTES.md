# konnect-codex plugin v0.9.0 — companion revision 1

This release is reviewed for
[Konnect v0.9.0](https://github.com/mixelpixx/Konnect/releases/tag/v0.9.0) at
commit `8648fe2573377eac78525907cbdd16216986f08e`.

Konnect 0.9.0 is the upstream “truth-in-responses” release. It adds atomic,
revision-gated placed-footprint refresh; atomic batch placement with live
readback; junction reconciliation after ordinary schematic moves; multi-unit
schematic mutations; live pad readback; transformed symbol geometry; and
correct bus-entry endpoint reporting.

## Guidance compatibility review

The upstream inventory remains 17 files. Two skills changed and no upstream
agent changed:

- `kicad-library` adds the dry-run/apply contract for
  `update_footprints_from_library`;
- `kicad-pcb` places linked-library refresh between schematic transfer and
  placement.

The reviewed Codex guidance carries those changes and adds the server's current
limits. Issue #331 means most official footprints containing `fp_text user`
are safely refused by refresh; issue #326 still protects the complete Default
netclass; issue #328 keeps bus connectivity checks advisory; and #315 means
`move_connected` still refuses even though ordinary moves now reconcile
junction dots. The PCB builder prefers `set_component_placements` for an
already reviewed batch so it is one atomic operation with post-commit readback.

The companion Freerouting bridge remains separate from Konnect's public tool
surface. It now passes the documented `--gui.enabled=false` setting and the
explicit router pass limit, so a complete route is headless and does not depend
on desktop control. DSN export and SES import still use KiCad 10's offline
Python API, preserve the source board, and require the existing acceptance
gate.

## Included

- Nine reviewed Codex-native workflow skills, including execution routing,
  BOM qualification, and controlled firmware/bring-up handoff.
- Five specialist Codex agents for libraries, schematics, PCB construction,
  independent review, and bring-up planning.
- Exact Konnect v0.9.0 version, commit, guidance, hook, and 17-file upstream
  baseline verification before plugin state changes.
- Eager access to the released 206 registered tools across 19 toolsets.
- Scoped MCP session inspection and cleanup with Windows child-process
  ownership, preventing stale Konnect processes from locking upgrades.
- Freerouting-first, explicitly headless, non-overwriting whole-board routing
  with live/offline PCB ownership preflight.
- Custom-part physical pin-map acceptance, schematic readability gates,
  placement checkpoints, ECO and power-layout branches, evidence-grounded
  review, legacy/manual assembly guidance, and manufacturing verification.
- Downloadable Windows, Linux, and macOS release archives with SHA-256 sums.

For the easiest setup, use the README's
[Install with Codex](https://github.com/neusse/konnect-codex#install-with-codex)
request. Konnect and its original hardware workflows are created and maintained
by [mixelpixx](https://github.com/mixelpixx); this repository is an independent,
version-matched Codex plugin.
