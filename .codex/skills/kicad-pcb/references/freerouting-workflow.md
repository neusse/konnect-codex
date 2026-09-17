# Whole-board Freerouting workflow

Use this branch for a complete board or interacting nets that require global
obstacle avoidance, rip-up/retry, and congestion management. Konnect v0.12.0's
native Specctra path is preferred when it accepts the actual board. The
companion KiCad-native bridge is the compatibility fallback for ordinary KiCad
geometry outside that first native profile.

## Route gate

Enter routing only after all of these are true:

- The intended `.kicad_pcb` is saved and its component, pad, trace, unrouted,
  rule, and direct-DRC inventory is recorded.
- Placement has no pad, hole, copper, courtyard, connector, mounting, or edge
  conflict that blocks routing or assembly.
- Board outline, keepouts, stackup, net classes, locked routing, and mechanical
  features are final enough to route.
- Copper zones are absent or intentionally unfilled until routing is accepted.
- The visible placement and applicable pre-route physics gates are closed.

This inventory and board revision are the checkpoint. A missing or implausible
value keeps the gate closed.

## Select the route path

1. With exactly one responsive PCB Editor owning the target board, call
   `check_freerouting`. Record the JAR, Java, MCP capability, version result,
   and diagnostics. A timed-out or unverified version is a warning, not proof
   that routing works.
2. Try `export_specctra_dsn` to new DSN and manifest paths. Use the default Rust
   exporter. This is the executable compatibility preflight for the board.
3. When export succeeds, complete the revision-bound native sequence:
   `route_specctra_dsn` -> `plan_specctra_ses_import` ->
   `apply_specctra_ses`. Review the manifest, plan, rejected items, warnings,
   and exact board revision at every boundary.
4. When native export rejects supported KiCad content, preserve the rejection
   as evidence, save the checkpoint, close PCB Editor, verify offline ownership,
   and run `konnect-codex freerouting status`. If ready, run
   `konnect-codex freerouting route --board <path> --passes <n>`. This uses
   KiCad's own DSN/SES APIs and writes a separate
   `<name>.freerouted.kicad_pcb`; it never overwrites the checkpoint.
5. If both paths are unavailable, return `INCOMPLETE` with both diagnostics and
   the smallest manual option. A whole board is not rerouted with local segment
   tools.

Konnect v0.12.0's Rust exporter has an intentionally narrow first profile. It
rejects common constructs including unnumbered NPTH pads and `roundrect` pads,
as well as unsupported layer counts, zones/rule areas, custom DRC rules, arcs,
and unlocked existing routing. `native_bridge_mode` does not widen this profile
in v0.12.0 because the restricted Rust baseline is constructed before the
optional ActionPlugin export is selected. Treat these as compatibility results,
not defects to remove from the board merely to satisfy the exporter.

`route_pad_to_pad`, `route_trace`, and generated segments are limited to an
intentional isolated connection or a small understood repair after a global
route. They are not a whole-board fallback.

## Route acceptance

After either route path:

1. For native apply, save and re-query the live board. For companion output,
   open only the generated board in one PCB Editor, then save and query it.
2. Compare component positions, footprints, pads, graphics, models, outline,
   rules, and locked geometry with the checkpoint.
3. Record traces by net and layer, unrouted count, shorts, and direct DRC.
   Contradictory live and file evidence invalidates the route.
4. Inspect the rendered routed board. Reject no-net copper, dangling vias,
   implausible segment growth, broken planes, and geometry not represented by
   aggregate DRC counts.
5. Accept only with no required unrouted connection, no unwaived DRC error or
   short, plausible route inventory, unchanged placement, and completed
   layout-physics evidence.
6. Add/refill zones only after route acceptance, rerun DRC and inventory, and
   save the accepted board.

If acceptance fails broadly, reject the generated board or reverse the single
native apply. Preserve the checkpoint; repair only a small, understood set of
local violations.

## Process ownership

Apply the shared
[process-lifecycle gate](../../kicad-workflows/references/process-lifecycle.md)
to PCB Editor, Konnect, Freerouting, Java, and bridge/helper processes. A
Konnect server started before KiCad may retain an unresolved v0.12.0 IPC
endpoint. If a live query still cannot see a newly opened editor, preserve the
current task server, report the startup-order limitation, and restart only a
verified task-owned secondary server or task from the saved checkpoint.

After acceptance, close task-owned secondary windows and router children.
Terminate only verified task-owned orphans and report every process deliberately
left running.
