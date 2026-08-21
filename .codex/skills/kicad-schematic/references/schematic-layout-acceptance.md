# Schematic layout acceptance

A schematic build is not complete until the page is electrically correct and
human-reviewable. Passing ERC or netlist checks does not prove readability.

## Block plan

Before placing symbols, define the functional blocks and assign each block to a
hierarchical sheet or a bounded region on the sheet. Typical blocks include
input protection, power conversion, MCU or logic, clocking, programming,
external connectors, sensors, analog front ends, drivers, and status indicators.

Before creating any project-local symbol, resolve the relevant KiCad standard or
project library. Register needed project libraries first. Generic parts such as
resistors, capacitors, power symbols, and standard connectors must use standard
KiCad symbols when available. Custom symbols are accepted only for parts missing
from the libraries or for exact part variants that need explicit pin evidence.

For each block, record:

- block name and purpose;
- sheet name or region bounds;
- member references and support parts;
- block interface nets;
- placement intent, such as left-to-right signal flow, edge connector access,
  power entry, or local analog/noisy separation.

Use hierarchical sheets for nontrivial subsystems, repeated channels, or blocks
that would crowd a single sheet. Use single-sheet bounded regions only when the
full schematic remains readable at normal review zoom.

## Grouping and tags

Component-only grouping is not full grouping.

Components that belong together must remain movable as a coherent block. Build
the complete group closure before accepting a block: parent symbol, support
parts, local labels, local wires, no-connect markers, text notes, and block
boundary graphics. Decoupling capacitors and their power/ground labels are part
of the parent device's group. Pin labels placed on a large IC are part of that
IC's group.

When Konnect exposes grouping tools that include every closure item, create a
real schematic group for all of them. If Konnect exposes only
`group_components`, that is component-only metadata, not full grouping. In that
case the block is accepted only when all non-component closure items are inside
the same bounded region, the report lists the missing label/wire/text grouping
capability, and moving the region with `move_region` would carry the complete
block. Do not report component-only grouping as satisfying full grouping.

Grouping is accepted only when a human can identify and move the complete block
without hunting for scattered support parts. Decoupling capacitors, pull-ups,
crystals, boot straps, programming headers, protection parts, and divider/filter
passives stay with the parent device or interface they support.

## Placement

Place blocks before local details. Keep the primary signal flow left-to-right or
top-to-bottom within each sheet unless the circuit convention says otherwise.
Keep power rails visually consistent, with supply symbols above loads and ground
symbols below where practical.

Place symbols, labels, wires, text, and no-connect markers on the schematic
grid. ERC off-grid warnings are layout defects for generated schematics unless a
specific imported/library symbol forces the exception and the exception is
reported.

Use direct wires for short local relationships. Use named labels for distant or
shared signals. Avoid long cross-page wires through unrelated blocks. Sheet pins
are the preferred interface for hierarchical blocks; global labels should be
reserved for true global rails or intentional cross-sheet signals.

## Overlap and page checks

No accepted layout may contain unresolved overlap among:

- symbol bodies, pins, and pin names;
- reference/value fields and custom fields;
- net labels, global labels, hierarchical labels, and sheet pins;
- wires, junctions, no-connect markers, and power symbols;
- text notes, rule/boundary graphics, and title-block/page edges.

Bounding checks must include labels and fields, not only symbol anchors. Pin-end
labels and intentional stubs may be waived only when ERC, connectivity, and
short checks prove the connection is correct and the rendered label is legible.
The rendered sheet is stronger evidence than the numeric overlap checker for
readability. If the render shows text collisions, clipped notes, labels crossing
pin names, or crowded support parts, the layout gate fails even when
`check_schematic_overlaps` reports zero.

All content must remain inside the selected page frame with enough margin for
the title block and future edits. If the schematic outgrows the page, split it
into hierarchical sheets or enlarge the page before adding more parts.

## Render checkpoint

After annotation, wiring, and local cleanup, render or capture every schematic
sheet. Inspect the render for:

- block identity and member locality;
- overlap or near-overlap of symbols, fields, labels, and wires;
- readable reference/value text;
- clear sheet-pin or label interfaces;
- page-frame fit and title-block clearance.

If readability fails, repair by moving whole blocks or regions first, then local
support components, then labels and wires. Do not finish by nudging isolated
symbols while leaving the block structure broken.

## Acceptance report

Report completion with:

- the block inventory and sheet/region assignment;
- grouping or tag mechanism used for each block;
- group-closure evidence for labels, wires, no-connects, notes, and support
  parts, or the exact Konnect grouping capability that is missing;
- overlap, page-boundary, ERC, short, orphan, and connectivity checks run;
- rendered sheet artifacts or the render/capture method used;
- unresolved readability waivers and why they are acceptable.

The layout gate is blocked if any component, field, label, wire, note, or
support part overlaps or is separated in a way a human would have to untangle
before review, movement, or reuse.
