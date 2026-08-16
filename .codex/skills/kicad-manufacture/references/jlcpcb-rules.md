# JLCPCB Manufacturing Reference

## Part Categories

| Category | Assembly fee | Notes |
|----------|-------------|-------|
| **Basic** | Included in standard fee | ~350 common parts, no setup charge |
| **Extended** | +$3 per unique part | Thousands of parts, requires setup |
| **Consigned** | User-supplied parts | You ship parts to JLCPCB |

Use `search_jlcpcb_parts` and check the `category` field in results.
Prefer **Basic** parts for cost optimization.

## BOM Format Requirements

CSV with columns:
```
Comment, Designator, Footprint, LCSC Part Number
100nF, C1;C2;C3, 0402, C1525
10k, R1;R2, 0402, C25744
```

- Multiple designators separated by `;`
- LCSC part number is the `C######` identifier
- Use `export_bom` then enrich with `search_jlcpcb_parts`

## Component Placement File (CPL)

CSV with columns:
```
Designator, Mid X, Mid Y, Layer, Rotation
C1, 10.5, 20.3, top, 0
U1, 25.0, 15.0, top, 90
```

- Coordinates in mm from board origin
- Layer: "top" or "bottom"
- Rotation: degrees, counter-clockwise from file
- Use `export_position_file` to generate

## Rotation Offsets

JLCPCB may rotate components differently than KiCAD's orientation.
Common offsets (add to KiCAD rotation):

| Package | Offset |
|---------|--------|
| 0402/0603/0805 passives | 0° (usually correct) |
| SOT-23 | 180° |
| SOIC-8 | 0° |
| QFP | 0° |
| QFN | 0° |
| USB-C receptacle | Verify visually |
| Electrolytic caps | Check polarity dot |

*Always verify rotation in JLCPCB's preview tool before confirming order.*

## Design Rules Summary

| Rule | JLCPCB Minimum | Recommended |
|------|---------------|-------------|
| Trace width | 0.127mm | 0.15mm |
| Trace space | 0.127mm | 0.15mm |
| Via drill | 0.30mm | 0.40mm |
| Annular ring | 0.15mm | 0.20mm |
| Hole to hole | 0.50mm | 0.60mm |
| Hole to edge | 0.30mm | 0.50mm |
| Min silkscreen | 0.15mm wide | 0.20mm wide |
| Pad to pad | 0.127mm | 0.15mm |

## Assembly Constraints

### Economic Assembly (cheaper)
- Single-side only (top OR bottom)
- Max 1000 unique parts per board
- No parts in slots or cutouts
- Component size: 0201 to 40x40mm

### Standard Assembly (full capability)
- Both sides
- Fine-pitch down to 0.35mm
- BGA support
- Odd-form components

## Order Workflow with Konnect

1. `get_drc_violations` — ensure zero errors
2. `validate_for_manufacturing` — pre-flight
3. `export_manufacturing_package` — generates all files
4. `search_jlcpcb_parts` for each component — get LCSC numbers
5. `suggest_jlcpcb_alternatives` for any out-of-stock parts
6. `estimate_cost` — get price breakdown
7. Upload Gerber zip + BOM CSV + CPL CSV to jlcpcb.com
