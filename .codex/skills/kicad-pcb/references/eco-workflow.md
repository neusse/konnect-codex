# Incremental PCB and ECO workflow

Use this workflow when a saved board already contains approved placement,
routing, zones, or mechanical work.

1. Save a baseline inventory: schematic references and footprints; board
   references, positions, pads, traces by net/layer, vias, zones, and DRC.
2. Run `update_pcb_from_schematic` with `dry_run: true`. Record the exact adds,
   removes, footprint changes, net changes, staged positions, diagnostics, and
   plan revision.
3. Identify affected nets and components. Treat an unexplained move, deletion,
   replacement, or broad connectivity change as a conflict.
4. Apply only the reviewed plan revision. Immediately compare unaffected
   placement and routing with the baseline.
5. Re-place only new or intentionally changed components. Preserve locked and
   accepted mechanical positions.
6. Reroute only affected nets when possible. Use whole-board Freerouting only
   when the change invalidates the global route, and then re-run the full route
   acceptance gate.
7. Refill affected zones and run direct DRC, shorts, unrouted, inventory, and
   changed-net checks.

Report a before/after delta. “Update succeeded” is not evidence that existing
layout was preserved.
