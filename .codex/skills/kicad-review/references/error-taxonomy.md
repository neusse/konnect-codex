# ERC/DRC Error Taxonomy

## ERC Error Severity

### CRITICAL (must fix before fabrication)

| Error | Meaning | Fix |
|-------|---------|-----|
| Pin connected to incompatible pin | Power output driving another power output | Verify net assignments, add diode/resistor |
| Unconnected power pin | IC power pin floating | Connect to appropriate power rail |
| Net with no driver | Signal net has only inputs | Add a driver (output pin, label to source) |
| Conflicting net names | Two labels on same wire segment | Remove duplicate, verify intended net |

### WARNING (investigate, may be intentional)

| Error | Meaning | Fix |
|-------|---------|-----|
| Unconnected pin | Pin without connection or no-connect marker | Add `no_connect` if intentional, wire if not |
| Pin not driven | Input pin without a driver on its net | Verify net has an output pin somewhere |
| Bidirectional pin conflict | Multiple bidirectional pins contending | Usually OK for buses, verify if intentional |
| Power pin not driven | Power input without a power flag | Add `PWR_FLAG` symbol to the net |

### INFO (usually benign)

| Error | Meaning |
|-------|---------|
| Duplicate reference | Two components with same refdes (pre-annotation) |
| Missing value | Component without a value field |
| Unresolved text variable | `${...}` variable without a definition |

## DRC Error Severity

### CRITICAL

| Error | Meaning | Fix |
|-------|---------|-----|
| Clearance violation | Copper-to-copper too close | Move trace or reduce width |
| Short circuit | Two different nets touching | Reroute or fix via placement |
| Unconnected items | Ratsnest not fully routed | Complete routing |
| Missing footprint | Component without a footprint | Assign in schematic, re-sync |
| Pad near edge | Copper pad too close to board edge | Move component inward |

### WARNING

| Error | Meaning | Fix |
|-------|---------|-----|
| Silk over pad | Silkscreen overlapping exposed copper | Move silk text |
| Courtyard overlap | Two components physically overlapping | Move component |
| Via near edge | Via too close to board outline | Move via inward |
| Minimum width | Trace narrower than design rule | Increase width or adjust rule |
| Annular ring | Via/pad ring too thin | Increase pad size or reduce drill |

### INFO

| Error | Meaning |
|-------|---------|
| Isolated copper | Copper island not connected to any net |
| Missing courtyard | Footprint without courtyard layer |
| Duplicate footprint | Two footprints with same reference |

## Interpreting Results

When `run_erc` or `get_drc_violations` returns results:

1. **Count by severity**: errors first, then warnings
2. **Group by type**: often one root cause creates multiple violations
3. **Fix in order**:
   - Shorts/clearances first (fabrication blockers)
   - Unconnected items (design incomplete)
   - Warnings (quality issues)
   - Info (cosmetic)
4. **Re-run after fixes**: one fix may resolve multiple violations
