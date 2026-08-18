---
name: kicad-pcb
description: "Lay out and route KiCad PCBs through Konnect MCP tools. Use for board setup, footprint placement, Freerouting or autoroute, traces, vias, differential pairs, zones, stackups, design rules, and silkscreen."
---

# KiCAD PCB Layout Workflow

This skill guides Codex to perform PCB layout using Konnect MCP tools.
ALL modifications go through MCP tools — never edit .kicad_pcb files directly.

---

## Prerequisites

Most PCB layout operations require KiCAD to be running with the board file open. The IPC
connection communicates with the running KiCAD instance in real-time.

`place_component` is the narrow exception: when IPC is unreachable, it can place one
front-side footprint into a closed `.kicad_pcb` file through a revision-aware atomic
fallback. It preserves pads, graphics, attributes, and models, and rejects duplicate
references. If KiCAD is reachable but rejects the request, the fallback stays disabled
to avoid racing a live editor.

If connection fails, open the target `.kicad_pcb` in one PCB Editor and retry
the readiness query once. After a live PCB phase begins, an IPC failure or an
unexpected `source: file`/fallback result closes the phase: stop mutations,
reopen the board, confirm its active path and inventory, and resume from the
last saved gate. Never mix live IPC and closed-file fallback in one placement
or routing sequence.

---

## Tool availability

The plugin exposes the complete Konnect catalogue eagerly. Call visible PCB
tools directly. When running against a lazy Konnect server and the tools are
absent, load these toolsets:

```
load_toolset('pcb_board')        # board outline, layers, setup, stackup
load_toolset('pcb_components')   # place, move, rotate, align footprints
load_toolset('pcb_routing')      # traces, vias, differential pairs
load_toolset('sch_export')       # update PCB from the saved schematic hierarchy
load_toolset('integration')      # check_freerouting, autoroute capability
```

Zones (`pcb_board`: add_zone; `pcb_routing`: add_copper_pour), component/net queries (`pcb_components`: find_component, get_component_list; `pcb_board`: get_board_info), and bulk placement (`pcb_components`: place_component_array, align_components, duplicate_component) are already covered by the toolsets loaded above.

On a lazy server, load these additional toolsets as needed:

```
load_toolset('config')           # design rule storage: add_design_rule, list_design_rules
load_toolset('verification')     # run_drc, set_design_rules, get_design_rules, check_clearance
```

Use `get_active_toolsets()` only to diagnose a missing tool on a lazy server.

---

## Layout Order

Follow this sequence for a clean PCB workflow:

1. **Board outline** — `set_board_size` or draw Edge.Cuts geometry
2. **PCB transfer integrity gate** — before transfer, inventory every selected
   footprint's reference, expected pad count, supported graphic layers, graphic
   count, and model associations with the available library/footprint queries.
   Reject or replace a footprint whose child items use an undefined or currently
   unsupported layer. Preserve this inventory as the post-apply invariant.
3. **Update from schematic** — call `update_pcb_from_schematic` first with
   `dry_run: true`. Review `status`, `coverage`, `diagnostics`, and staged positions.
   Apply only with `dry_run: false` and the exact returned
   `expected_plan_revision` value. The saved schematic hierarchy must be closed in the
   schematic editor, and the target board must be open in KiCad. A conflict is
   non-mutating; resolve it and rerun the dry run. A successful apply is one KiCad
   undo entry, so Ctrl-Z reverses the whole update. Treat the dry run as necessary
   but not sufficient: diagnostics must also agree with the inventory.
4. **Verify transfer invariants** — before placement or routing, re-query the
   board and compare footprint references, pad counts, graphic counts, layers,
   and models with the inventory. Phantom unnamed pads, missing graphics, changed
   pad counts, undefined layers, or missing models are transfer corruption. Stop,
   roll back the single update, and report the exact mismatch.
5. **Place components** — position all footprints
6. **Placement gate** — save; verify pad, hole, courtyard, edge, connector, and
   mounting clearances; record component positions, pad inventory, trace count,
   unrouted count, and direct DRC baseline
7. **Select the router** — use Freerouting by default for a complete board or
   interacting nets; read [references/freerouting-workflow.md](references/freerouting-workflow.md)
8. **Route acceptance gate** — verify unchanged placement/inventory, plausible
   traces, no shorts, clean direct DRC, and no required unrouted connection
9. **Copper pour** — add ground/power zones after route acceptance
10. **Final DRC** — refill zones, run direct DRC, and reconcile live queries
11. **Save** — `save_project`, then re-query final state

Do NOT add copper pours before routing is complete — they interfere with interactive routing.

---

## Placement

### Strategy

- Group components by functional block (power, digital, analog, connectors)
- Place ICs first, then their associated passives
- Decoupling caps: within 2mm of their IC power pins, on same layer
- Connectors: at board edges, accessible for cables
- High-frequency components: minimize trace lengths between them
- Thermal considerations: power components away from sensitive analog

### Placement Tools

| Tool                      | Use Case                                    |
|---------------------------|---------------------------------------------|
| `place_component`         | Position one footprint via IPC or safe file fallback |
| `move_component`          | Relocate an existing footprint              |
| `rotate_component`        | Rotate footprint (0/90/180/270)             |
| `align_components`        | Align multiple components (top/bottom/left/right/center) |
| `place_component_array`   | Grid placement for repeated elements        |

### Placement Tips

- Use mm coordinates (KiCAD default for PCB)
- Standard grid: 0.5mm for placement, 0.25mm for fine adjustment
- Check component courtyard overlaps after placement
- Check pad-to-pad and hole clearances before routing; a visually separated body
  can still have overlapping pads
- Reference designator text: F.SilkS layer, 1mm height default

---

## Routing

### Default route policy

Use Freerouting for whole-board routing. It provides global obstacle avoidance,
rip-up and retry, and congestion management that independent L-bends do not.
Run the placement gate first, then follow
[references/freerouting-workflow.md](references/freerouting-workflow.md).

Use the segment tools below for deliberate local work: a trivial isolated net,
a controlled power connection, a differential segment, or a small repair after
the imported route passes its acceptance gate. They are not a fallback
whole-board autorouter.

### Routing Tools

| Tool                      | Use Case                                    |
|---------------------------|---------------------------------------------|
| `route_pad_to_pad`        | One unobstructed local L-bend or small repair |
| `route_trace`             | Manual segment-by-segment routing           |
| `route_differential_pair` | Matched-length USB/LVDS/Ethernet pairs      |
| `add_via`                 | Layer transition                            |
| `create_netclass`         | Define width/clearance rules for net groups |

### route_pad_to_pad

A narrow local routing tool. It looks up both pad positions and lays an
L-shaped trace between them without global obstacle or congestion planning.

```
route_pad_to_pad(board, net_name, ref1, pad1, ref2, pad2, layer?, width?)
```

- Emits one segment when the pads already share an X or Y, two otherwise
- Specify width in mm (e.g., 0.25 for signal, 0.5 for power)
- Routes entirely on `layer` (default `F.Cu`) — it does not add a via. To
  change layer mid-route, place the via yourself with `add_via` and route each
  side separately

### route_trace

One straight segment between two explicit points, for when you want to control
the path yourself.

```
route_trace(board, net_name, layer, x1, y1, x2, y2, width?)
```

- Use for controlled local repairs after inspecting nearby copper and pads
- There is no waypoint list: call it once per segment to build a polyline
- Coordinates are board-space mm

### route_differential_pair

For differential signals (USB, HDMI, Ethernet, LVDS).

```
route_differential_pair(board, net_pos, net_neg, x1, y1, x2, y2, gap?, layer?, width?)
```

- Lays two straight traces parallel to the given line, offset `(gap + width)/2`
  either side, so spacing is constant along the segment
- Not a length-matching router: it adds no serpentine tuning, and equal length
  only follows from the two traces being parallel segments. Skew introduced
  before or after this call is yours to correct
- Common pairs: USB_D+/USB_D-, LVDS_P/LVDS_N

### Netclasses

Define routing rules for groups of nets:

```
create_netclass(board, name, trace_width?, clearance?, via_drill?, via_diameter?)
```

The class is written to the project's `.kicad_pro` file, which is where KiCad
has kept netclasses since v7 — the board file is not modified.

Common netclass configurations:
- Signal: 0.25mm track, 0.2mm clearance
- Power: 0.5-1.0mm track, 0.3mm clearance
- USB: 0.3mm track, 0.15mm spacing (90 ohm differential)

### Via Defaults

- Standard signal via: 0.4mm drill, 0.8mm pad diameter
- Power via: 0.6mm drill, 1.0mm pad diameter
- Micro via (HDI): 0.1mm drill, 0.3mm pad diameter

---

## Copper Pour

Zone tools live in the `pcb_board` toolset.

### add_zone

Creates a copper pour area (polygon fill).

```
add_zone(board, net_name, layer, points, clearance?, min_width?)
```

- Almost always GND net on both F.Cu and B.Cu
- `points` is the outline polygon; define it slightly inside the board edge
  (0.5mm inset)
- There is no priority argument, and no `(priority …)` is written, so every
  zone takes KiCad's default. Set priority in KiCad if two pours overlap

### refill_zones

**Must call `refill_zones` after any change that affects copper pour:**
- After adding/moving components
- After routing new traces
- After modifying zone outlines
- After changing design rules

Zones do not auto-update — stale fills cause DRC errors.

### Zone Tips

- GND pour on both layers is standard practice
- Leave spoke thermal reliefs for through-hole pads (easier soldering)
- Use keepout zones to prevent copper in sensitive areas
- Zone clearance typically 0.3-0.5mm from traces

---

## Layer Reference

| Layer    | Name     | Purpose                              |
|----------|----------|--------------------------------------|
| F.Cu     | Front Copper   | Top copper traces and pads     |
| B.Cu     | Back Copper    | Bottom copper traces and pads  |
| F.SilkS  | Front Silk     | Top silkscreen (text, outlines)|
| B.SilkS  | Back Silk      | Bottom silkscreen              |
| F.Mask   | Front Mask     | Top solder mask openings       |
| B.Mask   | Back Mask      | Bottom solder mask openings    |
| Edge.Cuts| Board Outline  | Physical board boundary        |
| F.Fab    | Front Fab      | Top fabrication drawing        |
| B.Fab    | Back Fab       | Bottom fabrication drawing     |
| F.CrtYd  | Front Courtyard| Top component clearance area   |
| B.CrtYd  | Back Courtyard | Bottom component clearance area|
| In1.Cu   | Inner 1        | Internal copper layer 1        |
| In2.Cu   | Inner 2        | Internal copper layer 2        |

### Layer Usage Guidelines

- Route signals on F.Cu and B.Cu (2-layer) or add inner layers for complex boards
- Board outline MUST be on Edge.Cuts (closed polygon or rectangle)
- Silkscreen for reference designators and polarity marks
- Courtyard defines minimum spacing between components
- Use F.Fab/B.Fab for assembly drawings and component outlines

---

## Design Rule Check

After completing layout:

```
run_drc()
```

Common DRC errors and fixes:
- **Clearance violation**: move trace or component further apart
- **Unconnected net**: route missing connection
- **Track too close to edge**: move inward from board outline
- **Courtyard overlap**: increase spacing between components
- **Zone fill error**: run `refill_zones`

If a live trace query and direct DRC disagree, treat the board state as stale.
Reopen the target board once, re-query, and use the direct item-level DRC as the
acceptance gate. A successful tool response does not override contradictory
shorts, unrouted work, or implausible inventory.

---

## Rules

1. **Never edit .kicad_pcb directly** — all changes go through MCP tools
2. **Always verify placement after moves** — components may snap to unexpected positions
3. **Board outline first** — define the physical boundary before placing anything
4. **Refill zones after changes** — stale zone fills cause phantom DRC errors
5. **Check DRC before finishing** — run `run_drc()` and resolve all errors
6. **Use netclasses for consistency** — define track widths per net type, not per trace
7. **Keep one live owner** — once placement or routing begins through IPC, stop
   on editor loss or file fallback and reconnect before any further mutation
8. **Save frequently** — call `save_project` after major operations
9. **Load toolsets first** — check `get_active_toolsets()` and load what you need
10. **Copper pour last** — add zones only after routing is substantially complete
11. **Preserve transfer invariants** — no placement or routing begins until
    footprint, pad, graphic, layer, and model counts match the pre-transfer inventory
12. **Fail closed on implausible queries** — zero pads on a populated board or a
    successful transfer with changed child-item counts is a tool contradiction,
    not proof that the board is clean
13. **Freerouting first for complete boards** — reserve segment tools for
    intentional local work and verified repairs
14. **Gate routing on placement** — no autoroute begins with pad shorts,
    blocking courtyard conflicts, stale zones, or unresolved mechanical overlap
