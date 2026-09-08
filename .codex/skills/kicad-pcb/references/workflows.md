# PCB Workflows

## Schematic to PCB

1. Require a saved schematic with resolved footprints and acceptable ERC.
2. Record schematic symbol/net counts and existing board inventory.
3. Perform the supported transfer once, then re-query component, pad, net, and
   3D-model coverage. Do not repair serialized KiCad files by hand.
4. Reconcile missing, duplicated, or changed items before placement.

## PCB constraint setup

1. Establish board outline, mounting constraints, stackup, fabrication rules,
   net classes, clearances, vias, and current-carrying requirements.
2. Read [power-layout.md](power-layout.md), [design-rules.md](design-rules.md),
   [trace-width-table.md](trace-width-table.md), and
   [pcb-layout-physics-acceptance.md](pcb-layout-physics-acceptance.md) as
   applicable.
3. Start the PCB-physics applicability matrix and identify the stackup,
   interface, thermal/current, RF, test-access, and special-process evidence
   that must be closed after placement or routing.
4. Save and query the resulting constraints before placement or routing.

## Component placement

1. Lock board-edge, connector, display, mounting, and other mechanical anchors.
2. Place by functional block and signal flow, keeping decoupling and critical
   loops local.
3. Use score-first dry runs and apply [placement-acceptance.md](placement-acceptance.md).
4. Apply the placement-dependent checks in
   [pcb-layout-physics-acceptance.md](pcb-layout-physics-acceptance.md), including
   hot loops, thermal paths, antenna keepouts, and planned test access.
5. Render and inspect both board faces, silkscreen, courtyards, test points, and
   3D-model coverage. Resolve overlaps before routing.

Routing is blocked until the visible placement gate passes.

## PCB routing

1. Preserve the accepted placement and route constraints.
2. Follow [freerouting-workflow.md](freerouting-workflow.md) for a complete board
   unless the board or user requires a justified manual strategy.
3. Validate route import inventory, unrouted count, shorts, clearances, and
   direct DRC before zones or manufacturing output.
4. Complete [pcb-layout-physics-acceptance.md](pcb-layout-physics-acceptance.md)
   against the actual routed layers and filled reference planes. Preserve its
   applicability matrix, direct evidence, and waivers.
5. Add/refill zones only after routing, then run final DRC and visual inspection.
6. Save the accepted board, then close the shared
   [process-lifecycle gate](../../kicad-workflows/references/process-lifecycle.md)
   for PCB Editor, Freerouting, Java, and any routing helper started by the task.

An autorouter success message is not routing acceptance.

## Engineering change

Follow [eco-workflow.md](eco-workflow.md). Establish a before-state, classify
the change, preserve unaffected placement and routing, apply the smallest safe
change, then compare inventory, connectivity, unrouted count, and DRC to the
baseline. Close the shared process-lifecycle gate for any editor or helper the
task started. Unexpected collateral change makes the result `INCOMPLETE`.
