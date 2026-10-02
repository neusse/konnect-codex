# Functional-block relocation

Konnect 0.13.0 does not provide native schematic grouping or editor-style
connected dragging. Treat block movement as a managed reconstruction, not as a
symbol move with an optimistic success report.

Use this procedure when moving an existing functional block. Do not use it for
ordinary first-pass placement where the block has not been wired yet.

## Closure manifest

Before changing geometry, record one closure manifest for the block:

- exact symbol references and UUIDs, including support parts;
- source bounds, destination bounds, and one grid-aligned translation vector;
- every internal wire segment and junction;
- every local, global, and hierarchical label owned by the block;
- pin-owned no-connect markers and power symbols;
- notes, text boxes, and boundary graphics;
- interface nets that intentionally cross the block boundary;
- pre-move ERC, short, orphan, and exported-connectivity evidence.

The manifest is incomplete if ownership is ambiguous. A wire crossing the
boundary is an interface, not an internal wire. A shared label or rail is not
owned by one block merely because it is rendered nearby.

## Preflight

1. Render the sheet and inspect the proposed source and destination regions.
2. Expand the destination bounds to include fields, labels, wire bends, notes,
   and editing margin. Reject any collision with another block or the page
   frame before mutation.
3. Confirm that every closure item has a safe exact move or delete-and-recreate
   operation in the active Konnect catalogue. If a note, graphic, label, or
   wire cannot be selected without affecting unrelated content, stop
   `INCOMPLETE`; do not leave it behind.
4. Save the project and retain the closure manifest and pre-move evidence as
   the rollback and comparison baseline.

## Relocate and reconstruct

1. Remove only the internal wires, labels, junctions, notes, or graphics that
   must be reconstructed. Preserve boundary interfaces and unrelated items.
2. Move the exact symbol-reference set by the same translation vector. Prefer
   an exact batch move. Use `move_region` only after proving its bounding box
   selects exactly the manifest's symbols.
3. Konnect 0.13.0 placement operations reconcile junctions and carry
   pin-owned no-connect intent. Verify their reported counts and read back the
   new pin locations; do not recreate duplicate no-connect markers.
4. Recreate every owned closure item at the translated coordinates. Rebuild
   local wires as short orthogonal segments, preserve label scope, and preserve
   the topology of each interface net.
5. Search the old bounds for remnants and the new bounds for missing or
   duplicated items. A clean-looking destination does not excuse abandoned
   geometry at the source.

Do not call `move_connected`. Do not use net-wide label movement when the same
net has labels outside the block. Do not treat `group_components` metadata as a
geometry operation.

## Equivalence and layout gate

After reconstruction:

- compare symbol and support-part membership with the manifest;
- compare each pin-to-net assignment and block interface net;
- run ERC, wire validation, component validation, short detection, orphan
  reconciliation, and exported-connectivity checks that are available;
- run overlap checks and inspect a fresh inline render for labels, fields,
  wires, notes, junctions, page bounds, and local passive topology;
- compare against the visual baseline and explain the intended translated
  region plus any other changed region.

Any unintended connectivity difference, remnant, missing closure item,
overlap, or unrelated visual change fails the move. Restore the saved state or
repair from the manifest before doing other work.

## Closure status

Report one status for every block:

- `NATIVE_GROUP`: a real KiCad group contains and moves the complete closure.
- `MANAGED_CLOSURE`: no native group exists, but every closure item is recorded,
  safely addressable, and a requested relocation completed with equivalent
  electrical and visual evidence.
- `BOUNDED_LAYOUT_ONLY`: the block is coherent and readable, but complete
  relocation has not been demonstrated or one closure item cannot be moved
  safely.

Only `NATIVE_GROUP` is directly movable by a human as one KiCad GUI object.
`MANAGED_CLOSURE` is movable through this Konnect reconstruction workflow.
`BOUNDED_LAYOUT_ONLY` is `INCOMPLETE` when movable grouping was requested.

For an executed relocation, add `relocation` to the schematic acceptance
record with `block`, `manifest`, `source_bounds`, `destination_bounds`,
`translation`, `preflight`, `items_removed`, `symbols_moved`,
`items_recreated`, `old_region_clear`, `connectivity_equivalent`,
`render_inspected`, and `verdict`.
