# Custom part acceptance

A custom part is accepted only when the physical datasheet lead, symbol pin,
and footprint pad are proven to represent the same electrical node.

## Required evidence table

Record one row per physical lead:

| Datasheet lead | Function | Symbol pin/name/type | Footprint pad | X/Y mm | Physical order and view | Evidence |
|---|---|---|---|---|---|---|

State whether the package drawing is a top view, bottom view, pin side, or
component side. Starting at the documented key or pin-one mark, walk every lead
in the documented direction. For circular parts, tubes, displays, connectors,
and sockets, never mirror the drawing mentally; write the viewing direction in
the table and verify the coordinate sequence explicitly.

## Gates

- Exact manufacturer part and package suffix match the source.
- Datasheet lead count equals symbol pin count and footprint electrical-pad
  count, except documented mechanical or stacked pins.
- Every lead number occurs exactly once in the symbol and footprint unless the
  datasheet explicitly joins duplicated leads.
- Electrical pin type, polarity, common connection, and no-connect status match
  the datasheet.
- Pad X/Y, drill, diameter, annular ring, pitch, body outline, and pin-one mark
  match the stated package tolerance and view.
- Courtyard and fabrication layers represent the physical part and intended
  socket/replaceability constraints.
- A queried-back symbol and footprint reproduce the evidence table.
- A visible disposable placement or footprint render agrees with the drawing.

Any unexplained reversal, mirror, missing pad, duplicate number, or view
ambiguity is BLOCKED. Use an independent library-builder or reviewer pass for
unusual or custom parts before schematic use.
