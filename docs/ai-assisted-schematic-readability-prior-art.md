# AI-Assisted Schematic Readability Prior Art

Date: 2026-08-21

## Scope

This note looks for prior art in AI-assisted KiCad or electronic schematic creation where the output is meant to be human-readable, not merely electrically connected. Sources are primary sources: GitHub repositories, official project documentation, KiCad documentation, and papers/project pages.

## Executive Summary

The strongest prior art does not rely on prompt guidance alone. The repeated pattern is:

1. Represent the schematic as structured geometry and connectivity.
2. Plan or infer functional blocks/substructures before detailed placement.
3. Use hierarchy, grouping, or graph structure to control page organization.
4. Compute or render geometry, including bounding boxes or SVG/PDF output.
5. Verify the result with a deterministic check, a visual inspection loop, or both.

For Konnect, the practical conclusion is that schematic creation should add a layout/readability acceptance gate, not just more prose. The gate should combine functional block planning, hierarchical sheet decisions, overlap/bounding-box checks, rendered schematic inspection, and a re-layout loop that moves blocks rather than isolated parts.

## Findings

### 1. KiCad's Own Documentation Treats Hierarchy as a Readability Tool

Source: [KiCad Schematic Editor documentation](https://docs.kicad.org/master/en/eeschema/eeschema.html)

Directly observed facts:

- KiCad documents that schematics can be split across a root sheet and sub-sheets, with each sheet stored as its own `.kicad_sch` file.
- The documentation says hierarchical design improves schematic legibility and reduces repetitive drawing.
- KiCad defines hierarchical labels and sheet pins as the parent-child sheet interface.
- KiCad includes a Sync Sheet Pins tool that checks whether hierarchical labels and sheet pins match.
- KiCad supports graphical elements, rule areas, and component classes that can be used to express visual or logical grouping.
- KiCad design blocks can save a selected circuit with a library link, and the documentation describes drawing a repeated channel circuit on a dedicated hierarchical sheet.

Inference:

- KiCad's native model already gives agents the right abstraction for real grouping: hierarchical sheets and sheet pins. For any nontrivial generated design, using hierarchy is more robust than relying on nearby placement alone.

### 2. EEschematic Uses Multimodal Iteration for Visual Clarity

Sources:

- [EEschematic GitHub repository](https://github.com/eelab-dev/EEschematic)
- [EEschematic arXiv paper](https://arxiv.org/abs/2510.17002)

Directly observed facts:

- EEschematic is an AI agent for automatic analog schematic generation from SPICE netlists.
- The repository says it targets human-readable, editable schematics and uses textual, visual, and symbolic modalities.
- The paper describes few-shot analog substructure examples for placement.
- The paper's initial placement step identifies substructures, spatial relationships, and orientations before detailed refinement.
- Wiring is generated from SPICE connectivity and terminal/node locations.
- The optimization loop sends rendered schematic images plus JSON placement/wiring data back to the multimodal model for iterative refinement.
- The paper evaluates both correctness and aesthetics. Correctness includes structural validity and no overlaps in components and wiring. Aesthetics are manually assessed for symmetry, alignment, compact wiring, and clarity.
- The paper reports that a more complex telescopic cascode remains harder: correctness can be high while aesthetic quality is lower.

Inference:

- This is the closest research pattern to the problem we are seeing: it explicitly separates electrical correctness from visual quality and uses a rendered-image feedback loop. The approach is analog-IC oriented, not KiCad-board-schematic oriented, but the workflow maps well: generate layout, render, critique visually/geometrically, revise.

### 3. Weave Uses Graph Layout Plus Round-Trip Verification

Sources:

- [Weave GitHub repository](https://github.com/senolgulgonul/weave)
- [Weave arXiv paper](https://arxiv.org/abs/2607.03835)

Directly observed facts:

- Weave converts a SPICE netlist into an LTspice `.asc` schematic.
- It parses the netlist into components and nets, classifies nets, and uses `elkjs` layered/Sugiyama graph layout to produce left-to-right signal flow and orthogonal routing.
- It handles some circuit patterns outside the main graph, including feedback loops, divider legs, hanging shunts, and supply corners.
- It stores symbol pin offsets and bounding boxes in an embedded symbol table.
- It performs a round-trip verifier: generated `.asc` is parsed back into a netlist and compared net-by-net against the original.
- It has a safe-mode ladder that disables layout patterns progressively until the verifier accepts a result.
- It explicitly admits cosmetic limits: long value texts can overlap nearby wires or symbols in crowded regions.

Inference:

- Weave shows the value of separating deterministic correctness from visual polish. A Konnect schematic layout gate should similarly have a binary connectivity check, but should add a visual/bounding-box gate for the cosmetic failures Weave still classifies separately.

### 4. Schemato Uses Human Examples and Image-Based Metrics

Sources:

- [Schemato arXiv page](https://arxiv.org/abs/2411.13899)
- [Schemato arXiv HTML](https://arxiv.org/html/2411.13899v1)
- [Sony AI publication page](https://ai.sony/publications/schemato-an-llm-for-netlist-to-schematic-conversion)

Directly observed facts:

- Schemato is an LLM approach for netlist-to-schematic conversion.
- It targets LTspice `.asc` and CircuiTikz output rather than KiCad.
- It uses prompts, few-shot examples, and fine-tuning on human-created netlist-to-schematic pairs.
- The paper evaluates compilation success and image similarity to reference schematics.
- The paper notes an example where GPT-4o produced correct RC network topology but overlapping components, while Schemato produced a cleaner, well-spaced schematic.
- The authors note that image similarity is sensitive to absolute component locations and may be unfair; they propose future graph-based metrics.

Inference:

- Schemato supports using human-designed examples as training or prompt references, but it also shows that syntactic validity and topological correctness do not guarantee readability. For our use case, examples alone are insufficient unless paired with geometry checks.

### 5. Circuit-Synth and kicad-sch-api Provide KiCad Geometry Primitives

Sources:

- [circuit-synth GitHub repository](https://github.com/circuit-synth/circuit-synth)
- [kicad-sch-api GitHub repository](https://github.com/circuit-synth/kicad-sch-api)
- [mcp-kicad-sch-api GitHub repository](https://github.com/circuit-synth/mcp-kicad-sch-api)

Directly observed facts:

- Circuit-Synth is a Python-based circuit design system with KiCad integration and AI acceleration.
- `kicad-sch-api` reads and writes KiCad `.kicad_sch` files and emphasizes exact format preservation.
- `kicad-sch-api` lists connectivity analysis through wires, labels, and hierarchy as a core feature.
- `kicad-sch-api` lists component bounding boxes, Manhattan-style orthogonal routing, and basic obstacle avoidance as core features.
- Its README includes functions to calculate and draw component bounding boxes.
- The MCP wrapper exposes the API as tools for AI agents, including schematic creation, component placement, connections, and hierarchical design support.

Inference:

- This is directly relevant implementation prior art for a KiCad-specific gate. Even if Konnect does not use this library, the feature set confirms that bounding boxes, obstacle-aware wiring, and hierarchy-aware connectivity are practical primitives for AI schematic creation.

### 6. nl2sch Explicitly Uses Bounding Boxes to Avoid Symbol Overlap

Source: [nl2sch GitHub repository](https://github.com/tpecar/nl2sch)

Directly observed facts:

- `nl2sch` converts a netlist to a KiCad schematic.
- Its README describes using bounding boxes, including labels, so the engine can place symbols without overlapping.
- The same note says those bounding boxes can support future vertical/horizontal symbol packing.
- The README also advises users to move generated symbols into new sheets or standalone schematics for viewing/editing.

Inference:

- Even older/non-AI KiCad netlist-to-schematic tools hit the same issue: overlap prevention is a geometric requirement, not a language-model preference. Label-inclusive bounding boxes are especially relevant because our generated sheets often fail through text/label clutter, not only symbol-body overlap.

### 7. d3-hwschematic Shows a Hardware-Schematic Graph Layout Pattern

Source: [d3-hwschematic GitHub repository](https://github.com/Nic30/d3-hwschematic)

Directly observed facts:

- `d3-hwschematic` is a D3.js and ELK-based hardware schematic visualizer.
- Its README lists automatic layout with layered graph layout and orthogonal routing via `elkjs`.
- It supports hierarchical components that can be expanded interactively.
- It supports net selection, highlighting, zoom, drag, custom renderers, and CSS.

Inference:

- This is not a KiCad authoring tool, but it is strong evidence that layered graph layout plus explicit hierarchy is a well-trodden path for readable hardware schematics.

### 8. kicad_monkey and kicad-actions Show Rendered-Schematic Inspection Is Automatable

Sources:

- [kicad_monkey GitHub repository](https://github.com/wavenumber-eng/kicad_monkey)
- [actions-for-kicad/kicad-actions GitHub repository](https://github.com/actions-for-kicad/kicad-actions)

Directly observed facts:

- `kicad_monkey` can render every schematic sheet instance to SVG, including concrete instances of hierarchical sheets.
- `kicad-actions` can run ERC and export schematic PDF, SVG, DXF, PS, BOM, and netlist artifacts in GitHub Actions.

Inference:

- Rendered inspection can be part of CI or agent completion. For Konnect, a schematic generation task should not finish until there is a rendered view suitable for human or multimodal inspection.

### 9. Existing KiCad AI Assistants Expose Editing Tools, But Do Not Clearly Solve Readability

Sources:

- [KiCad AI Assistant GitHub repository](https://github.com/paul356/KiCad-AI-Assistant)
- [Konnect upstream GitHub repository](https://github.com/mixelpixx/Konnect)
- [Konnect tool directory](https://github.com/mixelpixx/Konnect/blob/main/tool-directory.md)
- [kicad-happy GitHub repository](https://github.com/aklofas/kicad-happy)

Directly observed facts:

- KiCad AI Assistant embeds an LLM chat panel inside KiCad and exposes MCP tools for reading and editing schematics and PCBs.
- Its README lists many PCB placement/grouping tools, including group scoring and placement, but the visible README does not show an equivalent schematic readability gate.
- Konnect exposes schematic component tools such as add, move, move-connected, move-region, group-components, reset field positions, and `get_schematic_view`.
- Konnect's tool directory warns that content outside a too-small schematic page still exports and nets up, making a too-small page a silent defect.
- kicad-happy focuses on review and analysis, with release notes and validation around schematic connectivity, buses, hierarchy, and analyzer correctness. Its visible README describes fabrication gates and validation over a large KiCad corpus, but not schematic layout generation.

Inference:

- The AI-KiCad ecosystem has many mechanisms for editing and checking designs, but public docs do not show a mature, standard schematic-readability gate. Konnect already has enough low-level operations to build one: page sizing, region movement, grouping metadata, rendered views, and connected moves.

### 10. MCPkicad Documents Label Geometry and Overlap Checking Details

Source: [MCPkicad CLAUDE.md](https://github.com/Bov27/MCPkicad/blob/beta-test/CLAUDE.md)

Directly observed facts:

- MCPkicad documents `.kicad_sch` writing behavior for wires, labels, junctions, no-connects, and wire splitting.
- It documents label bounding-box geometry, including label direction and connection-point behavior.
- It references `check_schematic_overlaps` and notes a `suppressPinLabels` option to filter normal pin-endpoint labels.
- It states that label justification and angle must both be correct for proper rendering.

Inference:

- This is direct evidence that overlap checking for KiCad schematics needs KiCad-specific label geometry, not only generic rectangle collision detection. It also suggests a useful suppression rule: ignore intended pin-stub labels while still catching unrelated label/symbol clutter.

## Design Patterns Worth Copying

### Functional-block planning before placement

Observed in EEschematic's substructure-first placement and KiCad's hierarchy model.

Apply to Konnect:

- Require a block inventory before symbols are placed.
- Assign each block to a sheet or bounded region.
- Place support parts with their parent device, not from a flat component list.

### Hierarchical sheets as the real grouping mechanism

Observed in KiCad documentation, Circuit-Synth/kicad-sch-api, and d3-hwschematic.

Apply to Konnect:

- Use hierarchical sheets for substantial subsystems.
- Use sheet pins as explicit interfaces.
- Treat global labels as an exception, not the default way to connect every distant signal.

### Bounding boxes that include labels and fields

Observed in `nl2sch`, `kicad-sch-api`, Weave, and MCPkicad.

Apply to Konnect:

- Check overlap between symbol bodies, fields, labels, wire text, sheet pins, and title/page boundaries.
- Include field positions and label flags, not just component anchors.
- Suppress known-intent label stubs only when they exactly match an allowed pattern.

### Rendered inspection

Observed in EEschematic, kicad_monkey, kicad-actions, Konnect's `get_schematic_view`, and prior KiCad workflow guidance.

Apply to Konnect:

- Export/render every sheet after generation.
- Store the rendered artifact or at least report that it was inspected.
- Use a visual/multimodal critique loop when text geometry checks are inconclusive.

### Deterministic correctness separate from readability

Observed in Weave's round-trip verifier, kicad-actions ERC/netlist export, and kicad-happy validation emphasis.

Apply to Konnect:

- ERC/netlist equivalence should gate electrical correctness.
- Overlap/render inspection should gate human readability.
- Passing one gate must not imply passing the other.

## Suggested Konnect Acceptance Gate

A schematic creation task should not be complete until all of these are true:

1. The agent reports a functional block inventory and sheet/region plan.
2. Any nontrivial block is either on its own hierarchical sheet or inside an explicit bounded region.
3. Symbol and label geometry checks find no unacceptable overlap.
4. No content extends outside the selected page frame.
5. ERC passes or all ERC findings are explicitly waived.
6. Exported netlist/connectivity matches intended connections.
7. Every sheet has been rendered to PNG/SVG/PDF and inspected.
8. If readability fails, the repair step moves blocks/regions first, then local support parts, then labels/wires.

## Gaps in the Prior Art

- I did not find a public KiCad AI system that clearly advertises a complete schematic-readability acceptance gate combining hierarchy planning, overlap detection, rendered inspection, and re-layout.
- AI papers tend to target SPICE-to-LTspice, CircuiTikz, or analog-IC schematics rather than board-level KiCad schematics.
- Deterministic layout systems such as Weave have strong connectivity guarantees, but still acknowledge cosmetic overlaps.
- Public KiCad AI assistants expose useful editing tools, but their documentation does not show rigorous schematic layout QA comparable to PCB DRC.

## Recommendation

For this repository, the best path is to add explicit schematic layout acceptance guidance and then back it with tool-level checks where possible. The guidance should not say "make it readable"; it should require measurable or inspectable outputs:

- a block plan;
- hierarchy/sheet decisions;
- label-inclusive bounding-box checks;
- page-boundary checks;
- rendered schematic artifacts;
- ERC/netlist checks;
- a block-level re-layout loop.

This matches the strongest prior art while staying realistic about what current KiCad AI tooling appears to provide.
