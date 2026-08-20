# Pre-Fabrication Design Checklist

This is a prompt list, not a universal acceptance specification. Calibrate each
item to the confirmed design context, component datasheets, operating
conditions, and fabrication or assembly requirements. Values below are common
starting points only.

## Schematic Review

### Power
- [ ] Every IC has datasheet-appropriate decoupling close to its supply pins
- [ ] Bulk capacitance matches source impedance and load-transient requirements
- [ ] Power indicator LED (optional but recommended for debug)
- [ ] Reverse polarity protection on external power input
- [ ] Voltage regulator output cap per datasheet recommendation
- [ ] PWR_FLAG on power nets without power-output pins (prevents ERC error)

### Signal Integrity
- [ ] I2C pull-ups satisfy bus voltage, capacitance, device, and speed limits
- [ ] SPI chip select lines have pull-ups (prevent floating during boot)
- [ ] Reset pins use the device-recommended biasing or supervisor circuit
- [ ] Unused op-amp inputs tied to known state
- [ ] Crystal load caps match crystal specification
- [ ] ADC reference has dedicated decoupling

### Protection
- [ ] ESD protection on external-facing interfaces (USB, Ethernet, GPIO headers)
- [ ] TVS diodes on power inputs (if external power)
- [ ] Current limiting resistors on LEDs
- [ ] Gate resistors on MOSFET drivers (prevent ringing)

### Connectivity
- [ ] No unconnected pins (except NC pins marked with no-connect)
- [ ] No floating inputs on logic ICs
- [ ] All nets have at least 2 connections (no single-pin nets)
- [ ] No shorted nets (distinct nets accidentally merged)

## PCB Review

### Mechanical
- [ ] Board outline is closed (no gaps)
- [ ] Mounting holes placed and correct diameter
- [ ] Connector positions accessible from enclosure
- [ ] Keep-out zones around antennas/RF sections
- [ ] Board dimensions match enclosure

### Routing
- [ ] No unrouted nets (ratsnest clear)
- [ ] Power traces adequately sized for current
- [ ] Differential pairs length-matched (USB, Ethernet)
- [ ] No acute angles on traces (acid traps)
- [ ] Via-in-pad only where needed (adds cost)
- [ ] Ground pour on back (or both sides)

### DFM (Design for Manufacturing)
- [ ] All traces/spaces meet fab house minimums
- [ ] All holes meet minimum drill size
- [ ] Annular rings adequate
- [ ] Silkscreen not overlapping pads
- [ ] Component courtyard no overlaps
- [ ] Thermal relief on ground pour connections
- [ ] Fiducial markers (for assembly, 3 minimum)

### Assembly
- [ ] All components have correct footprints
- [ ] Polarity markings visible (caps, diodes, ICs)
- [ ] Reference designators readable
- [ ] Component values on silkscreen (or fab layer)
- [ ] Test points accessible for debug

## Using Konnect for Review

| Check | Tool |
|-------|------|
| Unconnected pins | `find_orphan_items` |
| Shorted nets | `find_shorted_nets` |
| Single-pin nets | `find_single_pin_nets` |
| ERC violations | `run_erc` |
| DRC violations | `get_drc_violations` |
| Decoupling audit | `audit_decoupling` |
| Connection audit | `audit_connections` |
| Power rail audit | `audit_power_rails` |
| DFM audit | `audit_manufacturing` |
| Full review | `run_design_review` |
