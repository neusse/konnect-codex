# Saved Codex Exchange

- Saved: 2026-08-16T20:50:02.670755+00:00
- Source: Codex conversation
- Filename: `skills-enhancments.md`

## Prompt:

are there some simple improvements we should make to skills or agents?  also I did not see that any agents were active?

## Response:

Yes—there are several straightforward improvements worth making.

No subagents were active during the benchmark. I confirmed that only the primary `/root` agent was running. The two custom agents are installed correctly:

- `konnect_schematic_builder`
- `konnect_design_reviewer`

They are not background processes and do not activate automatically. A task must explicitly delegate work to them. Our router currently says to use them “when available and appropriate,” which is too vague, so the entire benchmark stayed with the primary agent.

The best simple improvements are:

1. Make agent invocation deterministic.

   - Complete schematic construction → use `konnect_schematic_builder`.
   - Final comprehensive review → use `konnect_design_reviewer`.
   - Run them sequentially, not concurrently, because parallel agents should not modify the same KiCad project or IPC session.

2. Add the benchmark lessons to the schematic skill.

   - Do not replace electrically valid direct labels merely because `find_orphan_items` reports them.
   - Treat ERC, exported connectivity, and short detection as stronger evidence.
   - After automatic stub placement, immediately check for collisions and shorted nets.
   - Record the CC2-to-GND event accurately: agent geometry mistake caught by Konnect; orphan findings were a Konnect verifier defect.

3. Add a PCB-transfer integrity gate.

   Before applying `update_pcb_from_schematic`, record and compare:

   - footprint count;
   - expected pad counts;
   - footprint graphic counts;
   - references and models.

   Abort if the transfer turns graphics into pads, changes expected pad counts, or introduces undefined layers. This would have stopped the 50-to-128-pad corruption before it reached the board.

4. Strengthen contradictory-verifier handling.

   The skills should explicitly say that `run_design_review` or `validate_for_manufacturing` cannot declare success when stronger evidence shows:

   - DRC errors;
   - unrouted connections;
   - zero reported pads on a populated board;
   - missing output artifacts;
   - corrupted transfer counts.

5. Improve the custom reviewer agent.

   Its current “non-negotiable” rules are too universal—for example, requiring ESD protection on every external interface and bulk capacitance on every rail. Those should be engineering defaults that are checked against the actual requirements and datasheets.

6. Clarify `konnect-codex doctor`.

   It reports:

   > `0 recognizable agents`

   even though our two plugin agents are installed. That count intentionally excludes `konnect_*` plugin-managed agents, but the output is confusing. It should say something like:

   > `Plugin agents: 2 installed; upstream-native agents: 0`

A PCB/layout agent would also be valuable because most of the benchmark work happened after schematic creation, but that is a slightly larger addition. I would first make the six changes above, then add a `konnect_pcb_builder` agent once the workflow and transfer safety gate are proven.
