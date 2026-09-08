# Schematic Workflows

## Project bootstrap

1. Capture functional blocks, interfaces, power rails, mechanical constraints,
   preferred parts, and manufacturing assumptions.
2. Create the project non-destructively and establish block ownership and sheet
   structure before placing components.
3. Identify custom-library and BOM qualification work that must finish first.
4. Record the initial validation and expected handoff.

Stop if project creation conflicts with existing KiCad source files.

## Requirements to schematic

1. Convert requirements into named functional blocks and interface contracts.
2. Resolve and verify library IDs, then place by block with room for readable
   wiring, labels, notes, and later movement.
3. Wire locally where connectivity aids understanding; use hierarchical or net
   labels at block boundaries and for repeated/global signals.
4. Apply [schematic-layout-acceptance.md](schematic-layout-acceptance.md), render
   the schematic, and inspect labels and wires for overlap.
5. Run ERC and connectivity checks and reconcile every exception.

Outcome: a saved, human-editable schematic whose functional structure is
visible and whose electrical evidence is reported directly.

## Schematic cleanup

1. Inventory the current connectivity before moving anything.
2. Define functional blocks and preserve net identity while moving one block at
   a time.
3. Replace long cross-page wiring with appropriate labels only when meaning
   remains clear. Preserve local wires that communicate circuit behavior.
4. Re-render and apply the full readability gate, including label extents.
5. Compare connectivity and ERC evidence to the pre-cleanup baseline.

Stop and restore the last known-good block if intended edits also drift or
connectivity changes unexpectedly. Visual improvement alone is not completion.

