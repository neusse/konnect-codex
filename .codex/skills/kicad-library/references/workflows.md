# Library Workflows

## Custom part

**Inputs:** exact manufacturer part number and suffix, authoritative datasheet,
package/viewing convention, intended library scope, and required metadata.

1. Search installed libraries before creating anything.
2. Extract the physical lead numbering and electrical meaning from the
   datasheet. Resolve top/bottom-view ambiguity explicitly.
3. Build a physical lead -> symbol pin -> footprint pad table and apply
   [custom-part-acceptance.md](custom-part-acceptance.md).
4. Reuse verified assets when they match; otherwise create the symbol and
   footprint through Konnect. Preserve datasheet, manufacturer, MPN, package,
   and 3D-model metadata when available.
5. Query the saved assets back, inspect the footprint, and validate pin/pad
   correspondence before schematic use.
6. Register the library at project or global scope as requested, then prove the
   created identifiers resolve from that scope.

Stop on an ambiguous view, incomplete pin table, mismatched pad count, or
unverified registration. Outcome: `ACCEPTED` with the mapping evidence and
resolvable library IDs, otherwise `INCOMPLETE`.

## Library-update propagation

**Inputs:** changed library identifier, affected projects, reason for the
change, and whether pin or pad meaning changed.

1. Inventory affected symbols, footprints, and projects before mutation.
2. Classify the change as graphical/metadata-only, geometry, or electrical
   pin/pad meaning. Electrical changes require explicit mapping review.
3. Preview the update and conflicts where supported; back up or commit project
   state before applying a broad propagation.
4. Apply the update to one project at a time and query the resulting instances.
5. Re-run relevant ERC, connectivity, footprint, and DRC checks.

Stop rather than overwrite local project edits or silently renumber pins/pads.
Outcome: a per-project applied/conflict report with direct validation evidence.

