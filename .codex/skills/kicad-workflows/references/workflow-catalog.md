# Workflow Catalog

Choose one primary workflow. Follow its linked domain procedure; this catalog
routes work and does not duplicate domain acceptance rules.

| Workflow | Use when | Owned procedure or skill |
|---|---|---|
| Project bootstrap | Starting a coherent project from requirements | [Schematic workflows](../../kicad-schematic/references/workflows.md) |
| Requirements to schematic | Turning functional requirements into a readable, validated circuit | [Schematic workflows](../../kicad-schematic/references/workflows.md) |
| Schematic cleanup | Electrically correct content is visually poor or hard to maintain | [Schematic workflows](../../kicad-schematic/references/workflows.md) |
| Custom part | A verified symbol, footprint, metadata, or global/private library entry is missing | [Library workflows](../../kicad-library/references/workflows.md) |
| Library-update propagation | A corrected library item must reach one or more projects safely | [Library workflows](../../kicad-library/references/workflows.md) |
| Schematic to PCB | Moving a validated schematic into a new or existing board | [PCB workflows](../../kicad-pcb/references/workflows.md) |
| PCB constraint setup | Establishing stackup, net classes, dimensions, and mechanical constraints | [PCB workflows](../../kicad-pcb/references/workflows.md) |
| Component placement | Producing an intentional, inspectable placement before routing | [PCB workflows](../../kicad-pcb/references/workflows.md) |
| PCB routing | Routing a complete board, normally with Freerouting first | [PCB workflows](../../kicad-pcb/references/workflows.md) |
| Engineering change | Changing a placed or routed board without losing unaffected work | [PCB workflows](../../kicad-pcb/references/workflows.md) |
| BOM qualification | Qualifying MPNs, sourcing, lifecycle, alternates, DNP, or assembly data | [BOM skill](../../kicad-bom/SKILL.md) |
| Existing-project intake | Establishing trustworthy project state before edits | [Review workflows](../../kicad-review/references/workflows.md) |
| Simulation readiness | Determining whether a design and its models can be simulated | [Review workflows](../../kicad-review/references/workflows.md) |
| Design review | Performing a comprehensive schematic or PCB audit | [Review workflows](../../kicad-review/references/workflows.md) |
| Project recovery | Recovering from a crash, stale process, lock, or uncertain saved state | [Review workflows](../../kicad-review/references/workflows.md) |
| Manufacturing release | Creating and verifying a revision-bound fabrication package | [Manufacturing release](../../kicad-manufacture/references/manufacturing-release-workflow.md) |
| Hardware bring-up | Producing firmware handoff and controlled first-power evidence | [Bring-up skill](../../kicad-bringup/SKILL.md) |

A single lookup, inspection, or narrowly scoped edit is not a workflow. Route it
directly to the owning domain skill.

The [process-lifecycle gate](process-lifecycle.md) is cross-cutting rather than
a separate design workflow. Apply it to any catalog path that launches or
restarts a KiCad editor, Freerouting, Java, simulator, or external helper.
