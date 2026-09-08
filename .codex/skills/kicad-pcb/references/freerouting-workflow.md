# Native Freerouting workflow

Use this branch for a complete board or any layout with interacting nets where
independent L-bends would cross copper, pads, courtyards, or board features.
Konnect v0.11.1 owns the complete Specctra pipeline; the companion supplies
workflow guidance and acceptance gates, not a second router implementation.

## Route gate

Enter autorouting only after all of these are true:

- The intended `.kicad_pcb` is open in exactly one responsive PCB Editor and a
  live Konnect query returns a plausible component and pad inventory.
- The board is saved through Konnect. Record component positions, footprint and
  pad counts, trace count, unrouted count, and direct DRC summary.
- Placement has no pad-to-pad shorts, hole or copper overlaps, blocking
  courtyard conflicts, connector interference, or mounting interference.
- Board outline, keepouts, stackup, net classes, differential-pair constraints,
  locked routes, and mechanical features are final enough to route.
- Copper zones are absent or intentionally unfilled until routing is accepted.

The recorded inventory and board revision are the checkpoint. A missing or
implausible value keeps the route gate closed.

## Native pipeline

Run these Konnect MCP tools in order:

1. `check_freerouting` — record the detected local Freerouting MCP endpoint,
   version, capabilities, and diagnostics. This is a readiness check only.
2. `export_specctra_dsn` — export the saved live board. Use Konnect's Rust
   exporter by default. Use `native_bridge_mode: prefer` only when the installed
   KiCad 10 ActionPlugin bridge is authenticated and its native exporter is
   useful; use `require` only for an explicit compatibility test. Record the
   returned board identity, revision, DSN path, manifest, warnings, and preserved
   locked routing.
3. `route_specctra_dsn` — submit that exact DSN to the detected local
   Freerouting MCP server with explicit limits. Require a successful `.ses`
   result tied to the exported artifact.
4. `plan_specctra_ses_import` — validate the SES against the exact live board
   and export manifest. Review the proposed operations, supported geometry,
   rejected items, warnings, and expected revision. Planning must not mutate the
   board.
5. `apply_specctra_ses` — apply only the reviewed plan to the unchanged board
   revision. Stop on revision drift, unsupported geometry, unlocked pre-existing
   routing, arcs, zones, or any structured conflict.

The first supported native profile is intentionally narrow: straight tracks and
through vias, with locked existing straight routing preserved. Treat a rejected
construct as a boundary to resolve, not permission to bypass validation or edit
KiCad files as text.

Do not fall back to the removed `konnect-codex freerouting` command. Do not
substitute repeated `route_pad_to_pad` calls or unconstrained generated segments
for a failed whole-board route. The KiCad Freerouting ActionPlugin remains a
manual fallback only when the native Konnect pipeline reports a documented
unsupported case.

## Import acceptance gate

After apply:

1. Save through Konnect and re-query component positions, footprint and pad
   inventory, traces by net and layer, and unrouted count.
2. Compare placement, inventory, graphics, models, outline, rules, and locked
   routing with the checkpoint and export manifest.
3. Treat zero traces on a visibly routed board, a large unexplained segment
   increase, no-net copper, or disagreement between a live query and direct DRC
   as stale state. Stop, reopen the target board once, and re-query.
4. Run direct DRC and short detection before repairing anything. Inspect exact
   items and nets for every short, clearance, edge, hole, and unrouted finding.
5. Accept the route only when no required connection is unrouted, no unwaived
   DRC error or short remains, trace counts are plausible, and checkpoint
   inventory and locked geometry are unchanged.
6. Add or refill zones only after route acceptance, run DRC again, save, and
   re-query the final state.

If acceptance fails broadly, reverse the single apply in KiCad when safe or
restore the saved checkpoint. Repair only a small, understood set of local
violations with Konnect segment tools or KiCad's interactive router.

## IPC loss

Every board-sensitive stage must remain tied to the same exact live board. If
KiCad closes, crashes, IPC refuses a connection, the active path changes, or the
board revision drifts between export, plan, and apply, stop. Reopen one target
board, verify identity and inventory, and restart from the saved route gate.

## Process cleanup gate

Apply the shared
[process-lifecycle gate](../../kicad-workflows/references/process-lifecycle.md)
to PCB Editor, Konnect, Freerouting, Java, and bridge/helper processes. Capture
the baseline before readiness or routing launches anything and use bounded
timeouts.

After route acceptance is saved, close task-owned secondary windows and router
processes normally. Verify exact children exited. Terminate only a verified
task-owned orphan and preserve the Konnect MCP/companion process serving the
current Codex task.
