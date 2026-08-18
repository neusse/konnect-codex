# Freerouting workflow

Use this branch for a complete board or any layout with interacting nets where
independent L-bends would cross copper, pads, courtyards, or board features.
Freerouting is the default whole-board router. Konnect's segment tools remain
useful for intentional short links and repairs after the imported route passes
the acceptance gate.

## Route gate

Enter autorouting only after all of these are true:

- The intended `.kicad_pcb` is open in exactly one responsive PCB Editor and a
  live Konnect query returns a plausible component and pad inventory.
- The board is saved through Konnect. Record component positions, footprint and
  pad counts, current trace count, unrouted count, and direct DRC summary.
- Placement has no pad-to-pad shorts, hole or copper overlaps, courtyard
  conflicts that block assembly, or connector and mounting interference.
- Board outline, keepouts, stackup, net classes, differential-pair constraints,
  and required mechanical features are final enough to route.
- Copper zones are absent or intentionally left unfilled until routing is
  accepted. Stale filled zones are not a routing baseline.

The recorded inventory is the checkpoint. A missing or implausible value keeps
the route gate closed.

## Select the bridge

1. Call `check_freerouting` and record the detected JAR, version, and diagnostic.
2. If Konnect's `autoroute` reports an operational DSN/SES bridge, use it and
   preserve its output paths and status as evidence.
3. If `autoroute` reports that DSN export or SES import is unavailable, use the
   KiCad Freerouting ActionPlugin through an available desktop-control
   capability. The ActionPlugin owns KiCad's DSN export, starts Freerouting, and
   imports the returned session.
4. If no automated bridge is available, pause at the route gate and give the
   user the smallest manual KiCad step: run **Tools > External Plugins >
   Freerouting**, complete routing, save, and return for verification.

A standalone Freerouting JAR is an engine, not a complete KiCad integration.
It becomes usable only when a working path exports Specctra DSN and imports the
resulting SES session.

Do not substitute repeated `route_pad_to_pad` or unconstrained grid-generated
segments for a missing whole-board bridge. Those tools do not perform obstacle
avoidance, rip-up and retry, or global congestion management.

## Import acceptance gate

After the route returns:

1. Save through KiCad/Konnect, then re-query component positions, footprint and
   pad inventory, traces by net and layer, and the unrouted count.
2. Compare component positions and inventory with the checkpoint. Routing must
   not alter footprints, pad counts, graphics, models, board outline, or rules.
3. Treat a zero trace result on a visibly routed board, a large unexplained
   segment increase, no-net copper, or disagreement between a live query and
   direct DRC as stale state. Stop, reopen the target board once, and re-query.
4. Run direct DRC and short detection before repairing anything. Inspect exact
   items and nets for every short, clearance, edge, hole, and unrouted finding.
5. Accept the route only when no required connection is unrouted, no unwaived
   DRC error or short remains, trace counts are plausible for the design, and
   the checkpoint inventory is unchanged.
6. Add or refill zones only after route acceptance, run DRC again, save, and
   re-query the final state.

If acceptance fails broadly, reject the imported session and restore the saved
checkpoint or reverse the import in KiCad. Repair only a small, understood set
of local violations with `route_trace`, `route_pad_to_pad`, `add_via`, or the
interactive router.

## IPC loss

Once a live PCB session begins, every mutation must continue against that same
live board. If KiCad closes, crashes, IPC refuses a connection, or a mutator
reports file fallback, stop the sequence. Reopen the board, confirm the active
path and inventory, and resume from the last saved gate. Never mix live IPC and
closed-file fallback within one placement or routing phase.
