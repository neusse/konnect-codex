---
name: kicad-bringup
description: "Create firmware handoff, GPIO maps, test-point plans, startup behavior, and controlled hardware bring-up procedures from a KiCad design. Use for programming, proof-of-life, rail checks, boot/reset, and first power-up planning."
---

# KiCad firmware and bring-up handoff

Use Konnect read-only queries to derive the handoff. Do not modify the design or
energize hardware as part of this skill.

1. Confirm the reviewed schematic and PCB revisions.
2. Inventory controllers, rails, regulators, interfaces, buttons, indicators,
   programming headers, boot/reset controls, and test points.
3. Build a GPIO table: firmware name, MCU pin, schematic net, direction, active
   level, reset/pull state, peripheral, voltage domain, external load, and safe
   startup state.
4. Build a test-point table: reference/pad, net, expected unpowered resistance,
   expected powered voltage or waveform, probe ground, and acceptance range.
5. Define staged bring-up: visual/unpowered checks; current-limited input;
   primary rails; reset/boot/programming; proof-of-life; interfaces; then loads.
6. State stop conditions, current limits, expected evidence, and recovery path
   for each stage. Never infer an unsafe current or voltage limit.
7. Record ambiguous pins, missing test access, and firmware-dependent safety
   assumptions as blockers.

For a substantial handoff, delegate this read-only phase to
`konnect_bringup_planner` after the independent design review. Keep it separate
from live KiCad mutation ownership.
