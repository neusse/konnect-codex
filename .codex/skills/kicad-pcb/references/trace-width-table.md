# Trace Width Reference

This file is a rough screening aid, not design proof. Trace/via current and
impedance depend on actual stackup, copper thickness, temperature rise,
ambient, length, allowable voltage drop, plating, and fabrication tolerances.
Use a recognized current calculation and the board fabricator's field solver
or impedance service for production constraints. The project netclass must
record the resulting value and assumptions.

## Current Capacity (1oz/ft² copper, 10°C rise, external layer)

| Current (A) | Min Width (mm) | Recommended Width (mm) |
|-------------|---------------|----------------------|
| 0.1 | 0.05 | 0.15 |
| 0.25 | 0.10 | 0.20 |
| 0.5 | 0.15 | 0.30 |
| 1.0 | 0.30 | 0.50 |
| 2.0 | 0.70 | 1.00 |
| 3.0 | 1.10 | 1.50 |
| 5.0 | 2.00 | 2.50 |

**For internal layers**: multiply width by 1.5x (less cooling)
**For 2oz copper**: a first-order estimate may multiply the 1oz width by about
0.7; recalculate for the actual process instead of using that factor as proof.

## Standard Trace Widths by Application

| Application | Width (mm) | Netclass name |
|-------------|-----------|--------------|
| Signal (general) | 0.15–0.25 | Default |
| High-speed digital | 0.10–0.15 | HighSpeed |
| Power (< 1A) | 0.30–0.50 | Power |
| Power (1–3A) | 0.50–1.50 | PowerHigh |
| USB 2.0 differential | Calculate for actual stackup and target impedance | USB |
| USB 3.0 differential | Calculate for actual stackup and target impedance | USB3 |
| Antenna / RF | per impedance calc | RF |

## Via Sizing

| Application | Drill (mm) | Pad (mm) | Current capacity |
|-------------|-----------|---------|-----------------|
| Signal via | 0.30 | 0.60 | ~0.5A |
| Standard via | 0.40 | 0.80 | ~1A |
| Power via | 0.50 | 1.00 | ~1.5A |
| Thermal via | 0.30 | 0.60 | Array of 4-9 for heat |

## Clearance Rules

| Item pair | Minimum (mm) | Recommended (mm) |
|-----------|-------------|-----------------|
| Trace-to-trace | 0.15 | 0.20 |
| Trace-to-pad | 0.15 | 0.20 |
| Trace-to-edge | 0.25 | 0.50 |
| Via-to-via | 0.20 | 0.30 |
| Via-to-trace | 0.15 | 0.20 |
| Component-to-edge | 1.00 | 2.00 |

## JLCPCB Minimums (standard process)

| Parameter | Minimum |
|-----------|---------|
| Trace width | 0.127mm (5mil) |
| Trace spacing | 0.127mm (5mil) |
| Via drill | 0.30mm |
| Via annular ring | 0.15mm |
| Hole-to-hole | 0.50mm |
| Board edge clearance | 0.30mm |

## Impedance reference

Do not publish a fixed width/gap without dielectric thickness, dielectric
constant, copper thickness, solder mask, reference-plane geometry, and the
fabricator's tolerances. External routing is generally microstrip; routing
between reference planes is stripline. Calculate the geometry for the selected
stackup and confirm it with the fabricator.
