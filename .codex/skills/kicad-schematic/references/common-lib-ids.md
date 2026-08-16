# Common KiCAD Library Identifiers

## Passive Components (Device library)
| lib_id | Description | Reference prefix |
|--------|-------------|-----------------|
| `Device:R` | Resistor | R |
| `Device:R_Small` | Resistor (compact symbol) | R |
| `Device:C` | Capacitor (unpolarized) | C |
| `Device:C_Polarized` | Electrolytic/tantalum cap | C |
| `Device:C_Small` | Capacitor (compact) | C |
| `Device:L` | Inductor | L |
| `Device:L_Small` | Inductor (compact) | L |
| `Device:D` | Diode | D |
| `Device:D_Zener` | Zener diode | D |
| `Device:D_Schottky` | Schottky diode | D |
| `Device:D_TVS` | TVS protection diode | D |
| `Device:LED` | Light-emitting diode | D |
| `Device:LED_Small` | LED (compact) | D |
| `Device:Q_NPN_BEC` | NPN transistor (B-E-C pinout) | Q |
| `Device:Q_PNP_BEC` | PNP transistor (B-E-C pinout) | Q |
| `Device:Q_NMOS_GDS` | N-channel MOSFET (G-D-S) | Q |
| `Device:Q_PMOS_GDS` | P-channel MOSFET (G-D-S) | Q |
| `Device:Crystal` | Crystal oscillator (2-pin) | Y |
| `Device:Crystal_GND24` | Crystal with ground pins 2,4 | Y |
| `Device:Fuse` | Fuse | F |
| `Device:Ferrite_Bead` | Ferrite bead | FB |
| `Device:Thermistor_NTC` | NTC thermistor | TH |

## Connectors (Connector_Generic library)
| lib_id | Description |
|--------|-------------|
| `Connector_Generic:Conn_01x02` | 1x2 pin header |
| `Connector_Generic:Conn_01x03` | 1x3 pin header |
| `Connector_Generic:Conn_01x04` | 1x4 pin header |
| `Connector_Generic:Conn_01x06` | 1x6 pin header |
| `Connector_Generic:Conn_01x08` | 1x8 pin header |
| `Connector_Generic:Conn_02x03_Odd_Even` | 2x3 pin header |
| `Connector_Generic:Conn_02x05_Odd_Even` | 2x5 pin header (JTAG/SWD) |
| `Connector_Generic:Conn_02x10_Odd_Even` | 2x10 pin header |

## Power Symbols (power library)
| lib_id | Net created | Notes |
|--------|-------------|-------|
| `power:GND` | GND | Main ground |
| `power:GNDREF` | GNDREF | Signal ground reference |
| `power:GNDA` | GNDA | Analog ground |
| `power:GNDD` | GNDD | Digital ground |
| `power:+3V3` | +3V3 | 3.3V rail |
| `power:+5V` | +5V | 5V rail |
| `power:+12V` | +12V | 12V rail |
| `power:VCC` | VCC | Generic positive supply |
| `power:VDD` | VDD | Generic positive supply (CMOS) |
| `power:VBUS` | VBUS | USB bus voltage (5V) |
| `power:+3.3VA` | +3.3VA | Analog 3.3V |
| `power:PWR_FLAG` | (none) | Power flag for ERC compliance |

## Voltage Regulators (Regulator_Linear library)
| lib_id | Description |
|--------|-------------|
| `Regulator_Linear:AMS1117-3.3` | 3.3V LDO, 1A |
| `Regulator_Linear:AP2112K-3.3` | 3.3V LDO, 600mA |
| `Regulator_Linear:MCP1700-3302E_SOT23` | 3.3V LDO, 250mA, low Iq |
| `Regulator_Linear:LP5907MFX-3.3` | 3.3V LDO, ultra-low noise |

## Interface ICs (Interface library)
| lib_id | Description |
|--------|-------------|
| `Interface_USB:CH340G` | USB-UART bridge |
| `Interface_USB:CP2102N-A02-GQFN24` | USB-UART bridge |
| `Interface_CAN_LIN:MCP2551-I-SN` | CAN transceiver |

## Common MCUs (MCU_ST library)
| lib_id | Description |
|--------|-------------|
| `MCU_ST_STM32F1:STM32F103C8Tx` | STM32 "Blue Pill" MCU |
| `MCU_ST_STM32F4:STM32F411CEUx` | STM32F4, 100MHz |

## Usage Notes

- Always verify a lib_id exists with `search_symbols` before using it
- Power symbols create their net automatically — no manual net label needed
- `PWR_FLAG` is needed on power nets that don't connect to a power output pin (fixes ERC warnings)
- For device variants (e.g., specific resistor values), set the `value` parameter when placing
- The `_Small` variants use compact symbols better suited for dense schematics
