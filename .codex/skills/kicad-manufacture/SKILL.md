---
name: kicad-manufacture
description: "Prepare KiCad projects for fabrication through Konnect MCP tools. Use for Gerbers, drill files, assembly outputs, manufacturing BOMs, pick-and-place files, JLCPCB checks, and production packages."
---

# KiCAD Manufacturing & Fabrication Workflow

This skill guides Codex through preparing a KiCAD design for manufacturing using Konnect MCP tools.
ALL modifications go through MCP tools — never edit project files directly.

---

## Tool availability

The plugin exposes the complete Konnect catalogue eagerly. Call visible
manufacturing tools directly. When running against a lazy Konnect server and
the tools are absent, load these toolsets:

```
load_toolset('pcb_export')       # export_gerber, export_bom, export_position_file, get_drc_violations
load_toolset('manufacturing')    # export_manufacturing_package, validate_for_manufacturing, estimate_cost
```

On a lazy server, load these additional toolsets as needed:

```
load_toolset('sch_analysis')     # inspect nets, verify footprints assigned
load_toolset('integration')      # search_jlcpcb_parts, suggest_jlcpcb_alternatives
load_toolset('pcb_export')       # export_3d for visual verification
```

Use `get_active_toolsets()` only to diagnose a missing tool on a lazy server.

### Reference routing

- Read [references/gerber-layers.md](references/gerber-layers.md) when choosing
  fabrication layers or checking a Gerber/drill inventory.
- Read [references/jlcpcb-rules.md](references/jlcpcb-rules.md) only when JLCPCB
  is the selected fabricator, then verify every time-sensitive field, fee,
  capability, and format against JLCPCB's current official order contract.
- Read [references/legacy-through-hole.md](references/legacy-through-hole.md)
  for legacy, surplus, socketed, or manually assembled parts.

---

## Pre-Flight Checklist

Run these checks BEFORE generating any manufacturing outputs. Stop and fix issues at each stage.

### 1. DRC — Zero Errors Required

```
get_drc_violations()
```

- All errors must be resolved. Warnings should be reviewed but may be waived.
- Common blockers: unrouted nets, clearance violations, minimum width violations.
- Do NOT proceed to export if any DRC errors remain.
- After Freerouting or any SES import, save and verify unchanged component/pad
  inventory, plausible traces by net and layer, and a fresh direct DRC. Imported
  routing is not manufacturing evidence until this gate passes.

### 2. Manufacturing Validation

```
validate_for_manufacturing()
```

In Konnect v0.11.0 this aggregate check confirms that Edge.Cuts content and
footprints exist, evaluates configured minimum trace width, reports a coarse
no-tracks heuristic, and incorporates direct KiCad DRC evidence. It does **not**
prove that the outline is closed, every pad has copper, drills satisfy the
selected fabricator, or silkscreen clears pads. Collect those results with
direct DRC, artifact/viewer inspection, and the current fab contract.

This aggregate result cannot override DRC errors, unrouted connections,
implausible board inventory, transfer mismatches, or missing artifacts. Any such
contradiction blocks manufacturing readiness.

### 3. Verify Footprints Assigned

Every schematic symbol must have a footprint assigned. Check for:
- Missing footprint assignments (shows as empty Footprint field)
- Mismatched footprints (wrong pad count for the symbol)
- Non-existent footprint references (library not found)

### 4. Legacy and hand-assembly branch

When the design uses legacy, surplus, socketed, or manually assembled parts,
follow [references/legacy-through-hole.md](references/legacy-through-hole.md).
Record lifecycle uncertainty, exact suffix, socket/replaceability intent,
height, orientation, hand-solder access, attrition, and alternate risk.

---

## Export Workflow

### One-Shot Export (Convenience)

```
export_manufacturing_package(board, output_dir, fab_house?, schematic?)
```

`fab_house` selects the house profile (there is no `format` argument). Pass
`schematic` when you want the BOM generated as part of the package.

Attempts the requested manufacturing files in one call:
- Gerbers (all copper layers + mask + silkscreen + edge cuts)
- Drill files (Excellon format)
- BOM (CSV)
- Pick-and-place / component position file (CPL)
- Job file (optional, fab-house specific)

The handler may return after partial failures and report them in warnings.
Inspect `warnings`, `files_generated`, and the output directory rather than
trusting request success alone. Verify every requested file exists and is
non-empty, the Gerber
set contains only the intended production layers, drill counts are plausible,
and assembly CSV files state or unambiguously use the intended units and origin.
Exclude mounting holes, fiducials, and other non-placeable footprints from the
CPL unless the assembly contract explicitly requires them.

### Manual Export (When You Need Control)

Use individual tools when you need specific settings per file:

#### Step 1: Gerbers

```
export_gerber(board, output_dir, layers?, drill_file?)
```

Standard layers to export:
- F.Cu, B.Cu (and inner layers if present)
- F.Mask, B.Mask
- F.SilkS, B.SilkS
- F.Paste, B.Paste (for stencils)
- Edge.Cuts (board outline)

#### Step 2: Bill of Materials

```
export_bom(schematic, output, format?, fields?, group_by?, labels?, exclude_dnp?)
```

Include fields: Reference, Value, Footprint, LCSC (if targeting JLCPCB).

#### Step 3: Component Position File

```
export_position_file(board, output, format?, side?, units?)
```

Required for SMT assembly. Contains X/Y/Rotation for each component.
Export separately for top and bottom if double-sided assembly.

---

## JLCPCB-Specific Guidance

JLCPCB capabilities, fees, stock categories, order-column names, and design
limits are time-sensitive. Do not use a static count, price, or minimum from
this skill as an order constraint. Read
[references/jlcpcb-rules.md](references/jlcpcb-rules.md), retrieve the current
official assembly/fabrication requirements on the order date, record the
source and date, and configure the project to the selected process.

### Part Sourcing

```
search_jlcpcb_parts(query)                     # Find LCSC part numbers
suggest_jlcpcb_alternatives(value, footprint)  # Find alternatives for OOS parts
```

Prefer currently eligible low-setup-cost parts when that matches the design,
but preserve exact MPN, package, ratings, and lifecycle requirements. Use
`search_jlcpcb_parts` as catalogue evidence, then verify the exact selected
part and the uploaded BOM/CPL preview. Use the field names and units required
by the current uploader rather than normalizing to an old hard-coded spelling.

---

## Cost Estimation

```
estimate_cost(board, quantity?, layers?, fab_house?)
```

In Konnect v0.11.0 this is a fixed rough heuristic, not a live quote. Label its
result as an estimate and obtain a current vendor quote before making a cost or
supplier decision.

Factors that increase cost:
- Layer count (2 vs 4 vs 6+)
- Board size
- Number of unique extended parts
- Double-sided assembly
- Special finishes (ENIG vs HASL)
- Tight tolerances below standard minimums
- Expedited turnaround

---

## 3D Verification

Before submitting to fab, always generate a 3D view:

```
export_3d(board, output, format?, include_unspecified?)
```

Visual checks:
- Component clearance (tall parts near board edges)
- Connector accessibility and orientation
- Mounting hole alignment
- Heatsink/thermal pad clearance
- Enclosure fit (if applicable)

---

## Common Mistakes

1. **Exporting with DRC errors** — Always run DRC first. A clearance violation can short traces on the fab board.
2. **Wrong drill file format** — JLCPCB expects Excellon format. PTH and NPTH in separate files.
3. **Missing board outline** — Edge.Cuts layer must be a closed polygon. Open outlines cause fab rejection.
4. **Silkscreen on pads** — Silkscreen ink on exposed copper pads prevents soldering. Remove overlaps.
5. **Wrong position file origin** — CPL origin must match board origin. Use board center or bottom-left corner consistently.
6. **Forgetting paste layer** — If ordering stencils, F.Paste/B.Paste must be exported.
7. **Out-of-stock parts in BOM** — Always verify availability with `search_jlcpcb_parts` before ordering.
8. **Rotation offsets** — JLCPCB may apply rotation corrections. Review their orientation guide for ICs and polarized components.
9. **Panelization not accounted for** — If panelizing, export from the panel file, not the individual board.
10. **Missing fiducials** — SMT assembly with fine-pitch parts requires at least 2 fiducial marks on each assembly side.

---

## Rules

1. **Never export without passing DRC** — zero errors required
2. **Never overstate validate_for_manufacturing** — report its actual coverage
   and collect direct evidence for checks it does not implement
3. **Always verify part availability** before finalizing BOM for assembly
4. **Export 3D model** before submitting order — visual sanity check
5. **Save project before export** — ensures exported files match current state
6. **Load toolsets first** — check `get_active_toolsets()` and load what you need
7. **Use one-shot export when useful** — then inspect warnings, generated-file
   inventory, and each requested artifact because partial completion is possible
8. **Double-check fab house requirements** — each house has slightly different file format expectations
9. **Verify artifacts directly** — success is incomplete until every requested
   output exists, is non-empty, and has plausible units, origin, layers, and rows
10. **Fail closed on contradictions** — package or validation success cannot
    override DRC errors, unrouted work, transfer corruption, or missing artifacts
11. **Preserve review evidence** — include raw checks, route provenance, renders,
    artifact inventory, and waivers using the kicad-review evidence-package format
