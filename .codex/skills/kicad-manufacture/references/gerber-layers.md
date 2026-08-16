# Gerber Layer Mapping

## Standard Gerber File Extensions

| KiCAD Layer | Gerber Extension | Purpose |
|-------------|-----------------|---------|
| `F.Cu` | `.gtl` or `-F_Cu.gbr` | Front copper |
| `B.Cu` | `.gbl` or `-B_Cu.gbr` | Back copper |
| `In1.Cu` | `.g2` or `-In1_Cu.gbr` | Inner layer 1 |
| `In2.Cu` | `.g3` or `-In2_Cu.gbr` | Inner layer 2 |
| `F.Mask` | `.gts` or `-F_Mask.gbr` | Front solder mask |
| `B.Mask` | `.gbs` or `-B_Mask.gbr` | Back solder mask |
| `F.SilkS` | `.gto` or `-F_Silkscreen.gbr` | Front silkscreen |
| `B.SilkS` | `.gbo` or `-B_Silkscreen.gbr` | Back silkscreen |
| `F.Paste` | `.gtp` or `-F_Paste.gbr` | Front paste (stencil) |
| `B.Paste` | `.gbp` or `-B_Paste.gbr` | Back paste (stencil) |
| `Edge.Cuts` | `.gm1` or `-Edge_Cuts.gbr` | Board outline |

## Drill Files

| File | Extension | Purpose |
|------|-----------|---------|
| Plated through-holes | `.drl` or `-PTH.drl` | Component holes + vias |
| Non-plated holes | `-NPTH.drl` | Mounting holes, slots |
| Drill map | `.drl.map` | Visual drill reference |

## What `export_gerber` Produces

The `export_gerber` tool generates all required files in one call:
- All copper layers present in the design
- Both mask layers
- Both silkscreen layers
- Both paste layers
- Edge.Cuts (board outline)
- Drill file(s)

## What Fab Houses Expect

### JLCPCB Upload
Upload a single `.zip` containing all Gerber + drill files.
JLCPCB auto-detects layers by extension or content.

### PCBWay Upload
Same as JLCPCB — single zip with all Gerbers.

### OSH Park Upload
Upload the `.kicad_pcb` file directly (they parse it themselves).
Or upload Gerber zip.

## Verification Checklist

Before uploading Gerbers:
1. `get_drc_violations` — zero errors
2. `export_3d` — visual check of the 3D model
3. Open Gerbers in a viewer (KiCAD's built-in, or gerbv)
4. Verify board outline is closed (no gaps in Edge.Cuts)
5. Verify drill file has correct hole count
6. Verify silkscreen doesn't overlap pads
