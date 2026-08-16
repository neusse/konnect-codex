# Design Rules by Fab House

## JLCPCB (Standard Process)

| Parameter | Minimum | Notes |
|-----------|---------|-------|
| Trace width | 0.127mm (5mil) | 0.09mm available with "advanced" |
| Trace spacing | 0.127mm (5mil) | |
| Via drill | 0.30mm | |
| Via pad diameter | 0.50mm min | Annular ring ≥ 0.15mm |
| Min hole size | 0.30mm | |
| Board thickness | 0.4–2.4mm | 1.6mm standard |
| Copper weight | 1oz or 2oz | 1oz default |
| Min board size | 10x10mm | |
| Max board size | 400x500mm | |
| Slots min width | 0.80mm | |
| Castellated holes | 0.60mm min | Edge-plated |
| Silkscreen min | 0.15mm width, 0.80mm height | |
| Board edge clearance | 0.30mm | Copper to edge |

## PCBWay (Standard Process)

| Parameter | Minimum |
|-----------|---------|
| Trace width | 0.10mm (4mil) |
| Trace spacing | 0.10mm (4mil) |
| Via drill | 0.20mm |
| Via pad diameter | 0.40mm min |
| Board thickness | 0.2–6.0mm |
| Copper weight | 0.5–13oz |

## OSH Park (2-layer)

| Parameter | Minimum |
|-----------|---------|
| Trace width | 0.152mm (6mil) |
| Trace spacing | 0.152mm (6mil) |
| Via drill | 0.254mm (10mil) |
| Annular ring | 0.102mm (4mil) |
| Board thickness | 1.6mm (fixed) |

## Netclass Setup for Design Rules

Use `create_netclass` to apply different rules per net type:

```
Default:    clearance 0.20mm, trace_width 0.20mm, via_drill 0.40mm
Power:      clearance 0.25mm, trace_width 0.50mm, via_drill 0.50mm
HighSpeed:  clearance 0.15mm, trace_width 0.15mm, via_drill 0.30mm
USB:        clearance 0.15mm, trace_width 0.15mm, via_drill 0.30mm
```

Then use `assign_net_to_class` to assign nets:
- Power nets (+3V3, +5V, VCC) → "Power"
- USB_DP, USB_DM → "USB"
- Clock nets → "HighSpeed"
