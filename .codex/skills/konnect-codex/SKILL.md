---
name: konnect-codex
description: "Route KiCad schematic, PCB, library, manufacturing, and design-review work through Konnect in Codex. Use whenever a task mentions KiCad, a .kicad_* file, circuit design, footprints, routing, ERC, DRC, Gerbers, or board fabrication."
---

# Konnect for Codex

Use this as the execution router; read the matching bundled domain skill before
performing the work.

## Start

1. Confirm the `konnect` MCP tools are available.
2. Select the domain skill: `konnect`, `kicad-schematic`, `kicad-pcb`,
   `kicad-library`, `kicad-review`, or `kicad-manufacture`.
3. Inspect the project and requirements before changing the design.
4. Perform every KiCad-source mutation through Konnect MCP tools.
5. Validate the result with the strongest available ERC, DRC, connectivity, or
   manufacturing checks before declaring completion.

## Codex execution profile

The plugin starts Konnect with `eager_toolsets = true`, so the first MCP tool
list contains the complete catalogue. Call the visible domain tools directly.
Router calls such as `load_toolset` remain useful only when this skill is used
against a different, lazy Konnect server.

When agent delegation is available, make these handoffs deterministic:

- Delegate a complete schematic build to `konnect_schematic_builder`.
- Delegate complete schematic-to-PCB transfer, board setup, placement, routing,
  and zone work to `konnect_pcb_builder` after the schematic is saved and
  validated. Use it directly for substantial layout work on an existing board.
- Delegate a comprehensive final or pre-fabrication review to
  `konnect_design_reviewer` after all design mutations are complete.
- Run the agents sequentially in schematic -> PCB -> review order when all three
  phases apply. Give one agent ownership of the KiCad project at a time; multiple
  agents must not mutate the same project or live IPC session.

If delegation is unavailable, execute the matching workflow in the current task
and state that no custom agent ran.

## Capability discipline

- Treat Konnect tool results as evidence about design state, not as permission
  to ignore contradictory ERC, DRC, connectivity, inventory, or artifact
  evidence. Reconcile contradictions explicitly and fail closed.
- Prefer batch tools for cohesive edits, then inspect and validate the result.
- Keep unsupported operations explicit. Report the missing server capability
  and the smallest safe manual step instead of editing KiCad serialization.
- Preserve user intent when electrical requirements conflict with generic
  design defaults; document the decision and validate it against the relevant
  component data.
- A workflow is complete only when requested artifacts exist and the available
  checks have been run or a concrete blocker is reported.
- A passing aggregate review or manufacturing verdict cannot override DRC
  errors, unrouted connections, corrupted transfer counts, implausible coverage,
  or missing requested artifacts.
