---
name: kicad-workflows
description: "Select and execute complete multi-stage KiCad workflows through Konnect. Use when a request spans requirements, libraries, schematic, PCB, review, manufacturing, recovery, simulation readiness, or bring-up and needs ordered phases, stop conditions, evidence, and a defined outcome."
---

# KiCad Workflows

Use this skill as the multi-stage workflow router. It does not replace the
domain skills or their command and acceptance rules.

Read [references/workflow-catalog.md](references/workflow-catalog.md), select
one primary workflow before the first mutation, then read the matching domain
skill and the exact workflow reference it owns.

Read [references/process-lifecycle.md](references/process-lifecycle.md) before
launching, restarting, or taking ownership of a KiCad editor, Freerouting,
Java, simulator, or other external helper. Its ownership baseline and cleanup
gate apply across every workflow in the catalog.

## Workflow selection gate

Before work begins:

1. State the requested outcome and select one primary workflow.
2. Collect required inputs or record assumptions that are safe and reversible.
3. Identify the ordered phases, their stop conditions, and required evidence.
4. Give one agent or current task ownership of the KiCad project at a time.
5. When external applications or helpers are involved, record their
   pre-existing/task-owned lifecycle baseline before the first launch.
6. Complete each phase before handing the project to the next domain.

Do not declare a workflow complete unless its requested artifact exists and its
required direct evidence was observed. A missing, skipped, failed, unavailable,
or contradictory required step makes the outcome `INCOMPLETE` with the smallest
safe next action stated. A workflow that started an application or helper is
also incomplete until the process-lifecycle cleanup gate is evidenced.

Keep a simple one-tool inspection or edit in its domain skill. Do not turn every
KiCad action into a multi-stage workflow.
