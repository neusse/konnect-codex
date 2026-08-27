---
name: kicad-schematic
description: "Design and modify KiCad schematics through Konnect MCP tools. Use for circuit implementation, component placement, wiring, pin connections, symbols, net labels, and power rails."
---

# KiCAD Schematic Design Workflow

This skill guides Codex to design schematics using Konnect MCP tools.
ALL modifications go through MCP tools — never edit .kicad_sch files directly.

---

## Tool availability

The plugin exposes the complete Konnect catalogue eagerly. Call visible
schematic tools directly. When running against a lazy Konnect server and the
tools are absent, load these toolsets:

```
load_toolset('sch_components')   # place, move, rotate, delete symbols
load_toolset('sch_wiring')       # wires, net labels, power symbols, connections
```

On a lazy server, load these additional toolsets as needed:

```
load_toolset('library')          # search_symbols, get_symbol_info, list_symbol_libraries
load_toolset('sch_batch')        # batch operations for 3+ items
load_toolset('sch_analysis')     # find components, get pin info, inspect nets
```

Use `get_active_toolsets()` only to diagnose a missing tool on a lazy server.

---

## Component Placement

For complete builds or substantial rearrangement, follow the schematic layout
gate in [references/schematic-layout-acceptance.md](references/schematic-layout-acceptance.md):
plan functional blocks, use hierarchy or bounded regions, group and tag each
block for human movement, and reject unresolved symbol, label, field, wire, or
page-frame overlap before completion.
Use available standard KiCad symbols for generic parts before creating local
symbols. Component-only grouping is not full grouping: labels, wires,
no-connects, text notes, and support parts must stay in the same movable block
region or the missing grouping capability must be reported as a blocker/waiver.

### Workflow

1. Search the library first: use `search_symbols` to find the correct lib_id
2. Get pin info: use `get_symbol_info` to see pin names, numbers, and positions
3. Register missing standard KiCad libraries into the project before creating
   any local generic symbol
4. Assign each part, label, note, and support component to a functional block
   before placement
5. Place on the 1.27mm grid (KiCAD default schematic grid)
6. Verify placement with `list_schematic_components`

### Common Library IDs

| Component        | lib_id                          |
|------------------|---------------------------------|
| Resistor         | `Device:R`                      |
| Capacitor        | `Device:C`                      |
| Capacitor Polar  | `Device:C_Polarized`            |
| Inductor         | `Device:L`                      |
| LED              | `Device:LED`                    |
| Diode            | `Device:D`                      |
| Zener            | `Device:D_Zener`                |
| NPN Transistor   | `Device:Q_NPN_BCE`              |
| PNP Transistor   | `Device:Q_PNP_BCE`              |
| N-MOSFET         | `Device:Q_NMOS_GDS`             |
| P-MOSFET         | `Device:Q_PMOS_GDS`             |
| 2-pin Connector  | `Connector_Generic:Conn_01x02`  |
| 4-pin Connector  | `Connector_Generic:Conn_01x04`  |
| Ground           | `power:GND`                     |
| +3.3V            | `power:+3V3`                    |
| +5V              | `power:+5V`                     |
| VCC              | `power:VCC`                     |
| VDD              | `power:VDD`                     |

### Rotation Conventions

- 0 degrees: default orientation (pins left/right)
- 90 degrees: rotated CCW (useful for vertical components)
- 180 degrees: flipped horizontally
- 270 degrees: rotated CW

Power symbols: GND uses 0 (arrow points down), VCC/VDD/+3V3/+5V use 0 (arrow points up).

### Spacing Guidelines

- Between ICs: 30-50mm horizontal, 20-30mm vertical
- Between passive components: 10-15mm
- Between a decoupling cap and its IC: 5-10mm
- Leave room for wiring: minimum 5mm between component pins and other elements
- Keep grouped functional blocks separated enough that references, values, net
  labels, sheet pins, and text fields do not overlap after rendering.

---

## Wiring

### Connection Methods — Decision Table

| Scenario                                | Method                  | Why                                      |
|-----------------------------------------|-------------------------|------------------------------------------|
| Two pins physically close (<30mm)       | `connect_pins`          | Direct wire, auto-routed                 |
| Named signal (SDA, MOSI, EN, etc.)      | `connect_to_net`        | Stub wire + net label, cleaner           |
| Power rail (VCC, GND, +3V3)             | `add_power_symbol`      | Proper power symbol, global net          |
| Bus signals (D0-D7)                     | `connect_to_net`        | Net labels with bus naming               |
| Cross-sheet signal                      | Global label            | Connects across schematic sheets         |
| Multiple pins to same net (3+)          | `batch_connect_to_net`  | Efficient bulk operation                 |

### connect_pins

Use for direct pin-to-pin connections. The tool auto-routes with L-bends.

```
connect_pins(schematic, ref1, pin1, ref2, pin2)
```

- Specify pins by pin number (from get_schematic_pin_locations)
- Works best when pins are nearby and facing each other
- Automatically creates wire segments with proper bends

### connect_to_net

Use for named nets. Creates a short stub wire and attaches a net label.

```
connect_to_net(schematic, reference, pin_number, net)
```

- Preferred for signals that connect to 3+ pins
- Preferred for named buses and control signals
- Keeps schematic clean and readable
- Net name must be consistent across all connections
- Name the pin rather than passing `pin_x`/`pin_y`: the stub then points away
  from the symbol body on its own, instead of the label text running back
  across the pin names. Override with `direction` only to fix a layout clash.
- `batch_connect_to_net` does the same for many pins in one read/write, and
  places its labels directly on the pin endpoints without stubs.
- Placing a label by hand with `add_schematic_net_label` instead? Take its
  rotation from `orientation_degrees` in `get_schematic_pin_locations`, or the
  text reads back across the symbol's pin names.
- Before creating stubs for nearby pins, inspect their endpoints and choose
  directions that cannot meet or cross. Adjacent vertical passive pins can
  otherwise produce coincident stub endpoints and merge different nets.
- Run `find_shorted_nets` immediately after each related stub group. A clean
  result is the completion criterion for that wiring step.
- A label placed directly on a pin endpoint is electrically valid. Do not rewire
  an electrically valid direct label solely to clear an orphan finding. Confirm
  it with ERC, exported connectivity or netlist evidence, and the short detector;
  record a contradictory orphan result as a verifier limitation.
- Konnect v0.10.0 connectivity queries are not bus-aware (#328). On a bus sheet,
  treat floating/orphan results at bus entries and bus labels as candidates and
  use KiCad ERC plus exported connectivity as authority before changing wiring.

### add_power_symbol

Use for all power connections, in preference to labelling a pin with the rail name.

```
add_power_symbol(schematic, power_net, x, y, rotation?)
```

- Takes coordinates, not a reference and pin number. Place it on the pin
  endpoint (from `get_schematic_pin_locations`) — a power symbol carries its
  pin at its own origin, so the two coinciding is the connection.
- `power_net` is loaded as `power:<power_net>`, so it must name a symbol in
  KiCad's power library: `+3V3` and `+12V`, never `3V3` or `12V`. A miss is an
  error and nothing is placed.
- `rotation` defaults to 0 — see Rotation Conventions above.
- A power pin landing mid-segment on a wire gets its junction dot
  automatically, in either order: symbol onto an existing wire, or a wire
  routed across an already-placed symbol.

---

## Batch Operations

Load `sch_batch` toolset when placing 3 or more components or making bulk connections.

### batch_place_components

Place multiple components in one call. Provide `schematic` and a `components` array of `{lib_id, x, y, rotation?, reference?, value?, unit?}` objects. Pass `reference` explicitly for each component -- it is not auto-assigned.

### batch_connect_to_net

Connect multiple pins to the same net in one call. Ideal for:
- Connecting all VCC pins on an IC
- Connecting all GND pins
- Bus signals across multiple ICs

### batch_edit_schematic_components

Bulk-modify component properties (values, footprints, fields) across multiple components.

### When to Use Batch vs Individual

- 1-2 components: individual calls
- 3+ components: batch operations
- Mixed operations (place + wire): do placement batch first, then wiring batch

---

## Common Patterns

### Decoupling Capacitor
Place 100nF cap (Device:C) within 5mm of IC power pin. Connect one pin to VCC via power symbol, other pin to GND via power symbol. One cap per VCC/VDD pin.

### Pull-up Resistor
Place resistor (Device:R) vertically. Connect one pin to the signal net via `connect_to_net`, other pin to VCC via `add_power_symbol`. Typical values: 4.7k for I2C, 10k for general.

### Voltage Divider
Two resistors in series, vertically aligned. Top to input net, middle junction to output net, bottom to GND. Use `connect_to_net` for input/output, `add_power_symbol` for GND.

### LED with Current-Limiting Resistor
Resistor in series with LED. Connect resistor to signal/power, resistor to LED anode, LED cathode to GND. R = (Vsupply - Vf) / If. Typical: 330R for 3.3V, 470R for 5V.

### Bypass/Decoupling Filter
For analog circuits: 100nF ceramic + 10uF electrolytic in parallel, close to power pins. Place ceramic closest to IC.

### Crystal Oscillator
Crystal (Device:Crystal) between XI and XO pins. Two load capacitors from each crystal pin to GND. Typical load caps: 12-22pF. Optional 1M feedback resistor across crystal.

---

## Post-Placement Verification

After placing components and wiring, always run these checks:

### annotate_schematic
Assigns reference designators (R1, C1, U1, etc.) to all unannotated components. Run after all placement is complete.

### validate_wire_connections
Checks that all wires connect properly to pins. Reports:
- Dangling wire ends
- Wires that miss pins
- Overlapping wires

### validate_component_connections
Verifies that components have the expected connections. Reports:
- Unconnected pins that should be connected
- Missing power connections

### find_orphan_items
Finds floating wires, labels, and symbols that are not connected to anything.
Treat its output as a candidate list. Pin-end labels and the pin end of a valid
stub may be false positives, so correlate each finding with ERC and exported
connectivity before changing geometry.

### Verification Workflow

1. Record the functional block inventory and sheet/region plan.
2. For an established sheet, call `set_visual_baseline` before a meaningful
   batch of edits. No stored baseline is a normal state; a baseline from a
   different renderer is stale evidence and must not be silently trusted.
3. Place and group/tag all block members.
4. Complete all wiring.
5. Run `annotate_schematic`.
6. Run `validate_wire_connections`.
7. Run `validate_component_connections`.
8. Run `find_shorted_nets`.
9. Run `find_orphan_items` and reconcile it with ERC/connectivity evidence.
10. Call `render_schematic_png` with `inline: true` and actually inspect the
    returned image. Use `get_schematic_view` or a sheet capture as an additional
    artifact when its structured SVG is useful. Inspect every sheet for symbol,
    label, field, wire, group, and page-frame overlap.
11. After editing an established baseline, call `compare_visual_baseline`.
    Treat DRIFT and its changed-region bounding box as a direction for visual
    review, not as automatic failure: an intended edit should drift.
12. Treat rendered text collisions, clipped notes, labels over pin names,
    off-grid warnings, and support parts outside their parent block as layout
    defects even if the numeric overlap checker is clean
13. Fix confirmed issues; preserve electrically valid geometry reported only by
   a contradictory orphan check
14. Re-render the affected sheet and save with `save_project`.

---

## Rules

1. **Never edit .kicad_sch files directly** — all changes go through MCP tools
2. **Never guess pin numbers** — always use `get_schematic_pin_locations` or `get_symbol_info` to look up pin numbers before connecting
3. **Always verify after changes** — run validation tools after placing and wiring
4. **Use the grid** — all placements on 1.27mm grid
5. **Search before placing** — use `search_symbols` to confirm lib_id exists
6. **Power symbols for power** — use `add_power_symbol` for rails, not net labels
7. **Net labels for named signals** — keeps schematics readable
8. **Save frequently** — call `save_project` after major operations
9. **Load toolsets first** — check `get_active_toolsets()` and load what you need before starting
10. **Batch for bulk** — use batch toolset for 3+ repetitive operations
11. **Gate automatic stubs** — inspect nearby pin geometry and run
    `find_shorted_nets` immediately after placement
12. **Reconcile verifiers** — never change a known-good net solely to satisfy a
    contradictory orphan result
13. **Accept layout visibly** — complete the schematic layout gate before
    declaring a generated or rearranged schematic human-usable
14. **Use real libraries first** — standard KiCad symbols beat local placeholder
    symbols for generic parts, connectors, and power symbols
15. **Do not call `move_connected` in v0.10.0** — it still refuses because wire
    carrying is not implemented (#315). Ordinary component moves now reconcile
    affected junction dots, but they do not carry attached wires. Use a plain
    move, explicitly repair affected wires, then run ERC and connectivity
    checks.
16. **Require full group closure** — a block is not movable until its labels,
    notes, wires, no-connects, and support parts move with it by real grouping
    or a bounded-region workflow
