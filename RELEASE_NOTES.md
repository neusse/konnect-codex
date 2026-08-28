# konnect-codex plugin v0.10.0-codex.2 — companion revision 2

This release is reviewed for
[Konnect v0.10.0](https://github.com/mixelpixx/Konnect/releases/tag/v0.10.0) at
commit `866933e8aca1c115479963463cc7e34370b5822b`.

Konnect 0.10.0 adds a deterministic placement-analysis toolset and schematic
visual-baseline tools. The release exposes 214 registered domain tools across
20 toolsets, plus six router/meta tools.

Companion revision 2 is a full Codex-guidance audit against upstream issues
#356, #357, and #358. It preserves all prior companion enhancements while
making their release-to-release treatment explicit and executable.

## Codex guidance audit

- Added a living guidance delta register and standards document. Every active
  enhancement now has a retain/revise/retire/add review path for the next
  Konnect release, and tests require every policy ID to remain represented.
- Corrected the seven known invalid KiCad 10 shortcut IDs, fixed the standard
  `Device:LED` polarity example (`1=K`, `2=A`), and removed universal diode,
  transistor, MOSFET, connector, crystal, and IC physical pin-number rules.
- Linked every bundled reference from its parent skill with a read trigger, so
  progressive disclosure can actually reach all 18 references.
- Added an explicit evidence contract to all five agents and the router: no
  check, render, inspection, export, or measurement may be reported as run
  without its result; missing or contradictory required evidence is
  `BLOCKED`/`INCOMPLETE`.
- Corrected manufacturing guidance to the v0.10.0 handler contract, made
  partial export/artifact verification explicit, labeled `estimate_cost` a
  rough heuristic, and demoted time-sensitive vendor/trace tables to sourced
  starting references.
- Replaced the single inaccurate PCB IPC hook warning with four reviewed
  runtime classes: live-only, live-or-closed fallback, closed-board-only, and
  dry-run/apply. A machine-readable tool contract is checked against every
  matcher, including real v0.10.0 `refill_zones` coverage.
- Refactored hook output behind a testable structured Codex response and added
  tests for runtime messages, reference reachability, unsafe shortcut
  regression, and living-register coverage.

## Guidance compatibility review

The upstream inventory remains 17 files. Two skills changed and no upstream
agent changed:

- `kicad-pcb` adds score-first placement automation with dry-run planners for
  schematic clustering, force-directed refinement, decoupling placement, and
  BGA fanout;
- `kicad-schematic` adds inline PNG rendering plus persistent visual baseline
  capture and comparison.

The reviewed Codex guidance integrates those additions into stronger acceptance
loops. Placement starts with an independent score, locks mechanically
constrained references, reviews a dry-run plan, re-scores after apply, and still
requires a human-readable 2D checkpoint. Schematic work captures a baseline
before an established-sheet edit, renders inline for actual inspection, and
treats visual drift as review evidence rather than an automatic failure.

The existing safety guidance remains active. Issue #331 still prevents most
official footprints containing `fp_text user` from being refreshed; issue #326
still protects the complete Default netclass; issue #328 keeps bus connectivity
checks advisory; and #315 means `move_connected` still refuses.

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
- Exact Konnect v0.10.0 version, commit, guidance, hook, and 17-file upstream
  baseline verification before plugin state changes.
- Eager access to the released 214 registered domain tools across 20 toolsets.
- Score-first placement and visual schematic feedback integrated into the
  companion's existing transfer, readability, placement, and review gates.
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
