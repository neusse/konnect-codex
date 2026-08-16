# KiCAD PCB Layer Reference

## Copper Layers

| Layer Name | Purpose | Notes |
|-----------|---------|-------|
| `F.Cu` | Front copper | Primary component side |
| `B.Cu` | Back copper | Secondary/ground plane |
| `In1.Cu` | Inner copper 1 | 4+ layer boards |
| `In2.Cu` | Inner copper 2 | 4+ layer boards |
| `In3.Cu`–`In6.Cu` | Inner copper 3-6 | 6+ layer boards |

## Silkscreen Layers

| Layer Name | Purpose |
|-----------|---------|
| `F.SilkS` | Front silkscreen (component outlines, labels) |
| `B.SilkS` | Back silkscreen |

## Mask Layers

| Layer Name | Purpose |
|-----------|---------|
| `F.Mask` | Front solder mask openings (pads exposed) |
| `B.Mask` | Back solder mask openings |
| `F.Paste` | Front solder paste (stencil) |
| `B.Paste` | Back solder paste (stencil) |

## Fabrication Layers

| Layer Name | Purpose |
|-----------|---------|
| `F.Fab` | Front fabrication (assembly drawings) |
| `B.Fab` | Back fabrication |
| `F.CrtYd` | Front courtyard (component keepout) |
| `B.CrtYd` | Back courtyard |

## Mechanical Layers

| Layer Name | Purpose |
|-----------|---------|
| `Edge.Cuts` | Board outline (REQUIRED for fabrication) |
| `Margin` | Board margin/keepout |
| `Dwgs.User` | User drawings (dimensions, notes) |
| `Cmts.User` | User comments |
| `Eco1.User` | User eco layer 1 |
| `Eco2.User` | User eco layer 2 |

## Common Operations by Layer

| Task | Layer to use |
|------|-------------|
| Board outline | `Edge.Cuts` |
| Traces/routing | `F.Cu`, `B.Cu`, `In*.Cu` |
| Component placement text | `F.SilkS` |
| Board text/logos | `F.SilkS` or `F.Cu` |
| Mounting holes | `Edge.Cuts` (outline) + all copper (pad) |
| Copper pour/zones | `F.Cu`, `B.Cu` (typically GND) |
| Test points | `F.Cu` or `B.Cu` (exposed pad) |

## Standard 2-Layer Stackup

```
┌─────────────────┐
│  F.SilkS        │  Silkscreen (white ink)
│  F.Mask         │  Solder mask (green)
│  F.Cu           │  Copper (35µm / 1oz)
│  Substrate      │  FR4 core (1.6mm)
│  B.Cu           │  Copper (35µm / 1oz)
│  B.Mask         │  Solder mask (green)
│  B.SilkS        │  Silkscreen (white ink)
└─────────────────┘
```

## Standard 4-Layer Stackup

```
┌─────────────────┐
│  F.Cu           │  Signal + components
│  In1.Cu         │  GND plane
│  In2.Cu         │  Power plane
│  B.Cu           │  Signal + components
└─────────────────┘
```
