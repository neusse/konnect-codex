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

The companion starts Konnect with `eager_toolsets = true`, so the first MCP tool
list contains the complete catalogue. Call the visible domain tools directly.
Router calls such as `load_toolset` remain useful only when this skill is used
against a different, lazy Konnect server.

For a complete schematic build or a comprehensive pre-fabrication review, use
the installed `konnect_schematic_builder` or `konnect_design_reviewer` custom
agent when agent delegation is available and appropriate. Otherwise execute the
same workflow in the current task.

## Capability discipline

- Treat Konnect tool results as the source of truth for the design state.
- Prefer batch tools for cohesive edits, then inspect and validate the result.
- Keep unsupported operations explicit. Report the missing server capability
  and the smallest safe manual step instead of editing KiCad serialization.
- Preserve user intent when electrical requirements conflict with generic
  design defaults; document the decision and validate it against the relevant
  component data.
- A workflow is complete only when requested artifacts exist and the available
  checks have been run or a concrete blocker is reported.
