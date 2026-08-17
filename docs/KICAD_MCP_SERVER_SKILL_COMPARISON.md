# KiCAD-MCP-Server workflow comparison

Date: 2026-08-16

## Scope

This compares the retained checkout at `C:\Users\georg\Documents\Codex\KiCAD-MCP-Server_hold_this_for_later` with the current `konnect-codex` companion. The retained checkout is `mixelpixx/KiCAD-MCP-Server` 2.6.0 at commit `d35dd01342c2ee6adbcd4522c60e0a8ac339f35f`. Its existing `package-lock.json` modification was not changed.

## Main finding

The old project has no `SKILL.md` files and no Codex/Claude agent definitions. It exposes 18 MCP prompt templates, but those templates are mostly ordinary design advice. Its better practical result came from its end-to-end workflow documentation, evidence hierarchy, backend/lifecycle behavior, and mature regression coverage—not from stronger “skills” in the current Codex sense.

## Meaningful differences

1. **One linear delivery workflow.** `docs/PCB_DESIGN_WORKFLOW.md` maps project setup, schematic design, PCB layout, verification, and manufacturing output to exact tools. The current companion divides these activities among domain skills and has no single end-to-end build/manufacturing conductor.

2. **A stronger schematic completion gate.** `docs/HEADLESS_AUTHORING.md` treats `kicad-cli` as ground truth when parsers disagree. A sheet is complete only after ERC, rendered SVG/PDF inspection, and exported-netlist verification. It explicitly warns against trusting a connected-pin count by itself. This would have caught or correctly classified several benchmark contradictions.

3. **Explicit save/reload ownership.** `docs/REALTIME_WORKFLOW.md` tells the user to save before MCP reads and reload after file-based writes. The server also implements board lifecycle tools and backend session pinning. This reduces stale-file and last-save-wins failures.

4. **Hybrid backend behavior.** The old server can use IPC and fall back to SWIG/file operations. The retained Windows config does not force `KICAD_BACKEND`, so automatic selection applies. That likely helped it finish boards while avoiding some IPC-only crash and translation paths encountered during the Konnect 0.6.0 benchmark. Konnect 0.6.1 fixes the reproduced `Dwgs.User` crash by covering every legal KiCad 10 footprint layer and refusing unrepresentable layers before submission. Hybrid fallback remains an implementation difference, not something skill prose can reproduce.

5. **Direct schematic-to-board synchronization.** The old workflow uses `sync_schematic_to_board` as the KiCad F8 equivalent and has targeted regression tests for footprint transfer. It does not use Konnect's Rust child-item translation path that produced phantom pads in the benchmark.

6. **A complete autoroute procedure.** The old workflow formally integrates Freerouting: check availability, export DSN, route, import SES, save, and run DRC. Its tests cover routing, scoring, and netclass export. The companion presently lacks comparable operational guidance, so the benchmark required manual recovery and trace-width repair.

7. **Visual verification is mandatory.** The old guidance repeatedly requires schematic and board inspection, not just aggregate tool results. The current companion has improved contradiction handling but does not yet make rendered schematic inspection a universal completion condition.

8. **Part qualification hierarchy.** The JLCPCB guidance prefers curated local symbols/footprints for known parts, then uses the catalog/API for discovery, and verifies discovered parts against the local library. The companion should additionally require exact symbol, footprint, pad count, supported layers, 3D model/render, manufacturer part number, LCSC number, and datasheet before transfer.

9. **Individually visible manufacturing outputs.** The old workflow exports Gerbers, BOM, position data, 3D, and drawings as distinct operations. This makes false aggregate success harder to hide. The companion's newer direct artifact checks move in the right direction, but controlled individual exports should be preferred whenever the package operation reports contradictory results.

10. **Implementation maturity.** The old checkout includes focused tests for auto-save guards, project preservation, session pinning, connectivity at labels/pins, schematic-to-board transfer, Freerouting, and IPC unit conversion. Skills can add gates and workarounds, but they cannot replace these server-level guarantees.

## Recommended companion improvements

Priority order:

1. Add an end-to-end KiCad delivery skill or router branch that maintains a phase/evidence ledger from requirements through fabrication artifacts.
2. Require the three-evidence schematic gate: ERC, visual render inspection, and exported-netlist pin/net verification.
3. Add a formal part-qualification gate before schematic-to-PCB transfer, including a real 3D render check.
4. Add a backend/project-state handshake: identify the active editor/backend, save or reload intentionally, verify the open file, and prevent competing writers.
5. Add a Freerouting stage with pre-route netclass checks and post-import unrouted-count, width, zone-refill, and DRC verification.
6. Prefer separately verified manufacturing exports when an aggregate packaging tool is known to provide incomplete or contradictory validation.

## What not to copy

Do not import the 18 generic prompt templates wholesale. Do not assume snapshots are valid merely because a tool returns a path—the Konnect benchmark already found a reported snapshot that did not exist. Do not describe hybrid SWIG fallback or direct F8 synchronization as companion capabilities unless Konnect itself provides them.

## Conclusion

The old server's successful manufacturing outcome is credible and explainable. The largest reusable lesson is a disciplined evidence-driven pipeline; the largest non-reusable advantages are its hybrid backend, direct board synchronization, lifecycle controls, and regression-tested implementation. The current companion already adopted useful contradiction, transfer-inventory, and artifact-verification gates. Adding the six recommendations above would capture most of the remaining workflow advantage without pretending that skill text can fix Konnect's server defects.
