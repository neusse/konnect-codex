# JLCPCB Manufacturing Reference

This file is a workflow prompt, not a frozen JLCPCB contract. Part categories,
fees, stock, process limits, uploader headers, and assembly capabilities are
time-sensitive. Retrieve and cite the current official JLCPCB requirements on
the order date; when this file disagrees with the current order interface, the
current official contract controls.

## Part categories

Use `search_jlcpcb_parts` and record the category, stock, price, and retrieval
date returned for the exact MPN. Verify the category and any setup fee in the
current order interface before optimizing around it.

## BOM Format Requirements

Export a CSV, then map its fields to the column names accepted by the current
JLCPCB uploader. A commonly encountered shape is:
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

## Design rules

Do not use a static table here as the design authority. Select the current
JLCPCB process, retrieve its official capabilities, record the source/date, and
configure KiCad to those values. Electrical current, voltage-drop, thermal, and
impedance constraints may require stricter rules.

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
6. `estimate_cost` — rough heuristic only; obtain a current vendor quote
7. Upload Gerber zip + BOM CSV + CPL CSV to jlcpcb.com
