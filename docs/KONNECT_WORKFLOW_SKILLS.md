# Konnect-Codex Skills, Agents, Hooks, and Workflows

## Purpose

Konnect provides a large set of useful KiCad operations. A successful hardware
design, however, is not the result of calling those operations in arbitrary
order. It requires requirements, part verification, readable schematic
construction, controlled PCB transfer, intentional placement, routing,
independent review, manufacturing evidence, and often a firmware/bring-up
handoff.

`konnect-codex` is the Codex-native guidance and lifecycle companion for that
work. It does not replace Konnect's Rust MCP server. It supplies the process
layer that tells Codex:

- which skill owns the current phase;
- which work must happen before and after that phase;
- when a specialist agent is useful;
- which direct evidence closes the phase;
- when contradictory or missing evidence makes the result `INCOMPLETE`; and
- how to finish without leaving task-owned KiCad, Freerouting, Java, or helper
  processes behind.

This document is the human-readable handbook. The installed `SKILL.md`, agent
TOML, hook JSON, references, and `policy/enhancements.json` remain the
executable sources of truth.

## What we built

The companion now contains much more than translated upstream prose:

- two router/integrity skills;
- one multi-stage workflow router;
- seven technical domain skills;
- five specialist Codex agents;
- seventeen named end-to-end workflows;
- a shared process lifecycle and cleanup gate;
- a Freerouting-first route and import-acceptance workflow;
- custom-part physical pin-mapping acceptance;
- schematic readability, grouping, and rendered-inspection gates;
- PCB transfer-inventory and visual-placement gates;
- a PCB layout-physics applicability and evidence gate;
- BOM qualification and time-sensitive sourcing evidence;
- independent review, manufacturing, and bring-up evidence contracts;
- prompt and pre-tool ownership hooks; and
- a release-durable enhancement policy with automated assertions.

## Why skills instead of a GUI?

The current failures are mostly workflow failures rather than missing buttons:
the model can call a tool but may omit a prerequisite, choose the wrong order,
accept weak evidence, or stop before the requested artifact is actually ready.
A skill can correct that behavior now while continuing to use Konnect's
supported Rust and KiCad interfaces.

A GUI becomes worthwhile when Konnect needs durable queues, resumable jobs,
cross-task approval state, shared part inventories, or a dashboard that cannot
be represented reliably in a task conversation. Adding a GUI before those
needs are concrete would create another stateful product without fixing the
execution contract.

These workflows therefore belong inside the existing **konnect-codex plugin**.
That plugin is already the installed and versioned Codex integration layer for
skills, agents, hooks, MCP configuration, and companion utilities. A second
plugin would split one contract across two releases and increase drift.

## Execution architecture

```text
User request
    |
    v
konnect-codex router skill
    |
    +-- one-stage task ----------> owning domain skill
    |
    `-- multi-stage outcome -----> kicad-workflows router
                                      |
                                      v
                              ordered domain phases
                    library -> schematic -> BOM -> PCB
                              -> review -> manufacture
                                      -> bring-up
                                      |
                                      v
                         direct evidence and outcome

Cross-cutting controls:
  konnect integrity skill | prompt/tool hooks | one-owner rule
  process lifecycle gate  | policy assertions | release review
```

| Layer | Responsibility | What it does not do |
|---|---|---|
| Konnect MCP server | Performs supported KiCad, library, verification, configuration, and manufacturing operations | It does not infer a complete product workflow from a short prompt |
| `konnect-codex` | Routes a KiCad request to the correct installed guidance | It does not replace the domain skill |
| `kicad-workflows` | Selects and orders multi-stage outcomes, gates, stop conditions, and handoffs | It does not duplicate every tool command |
| Domain skills | Define the technical procedure and acceptance evidence for one domain | They do not authorize unrelated phases |
| Specialist agents | Own a substantial delegated phase and return evidence to the parent task | They are not automatically running merely because a skill exists |
| Hooks | Add context and classify tool ownership requirements before matched calls | They do not replace server-side validation or supervise every child process |
| Companion CLI | Installs guidance, checks compatibility, preflights ownership, and supplies the KiCad-native whole-board compatibility route | It preserves the source board and does not replace Konnect's revision-bound native path |

## How skills and agents are invoked

Skills are guidance packages. Codex selects a model-invoked skill when the
request matches its description, or the user can invoke a skill explicitly.
Reading a skill does **not** start its associated agent.

Agents are separate delegated workers with a narrow ownership contract. The
router or current task invokes one only when delegation is available and the
phase is substantial enough to benefit from a handoff. If no agent is invoked,
the current task follows the same domain skill and states that no specialist
agent ran.

Only one agent or current task may mutate a KiCad project or live IPC session
at a time. The normal sequence is:

```text
konnect_library_builder
    -> konnect_schematic_builder
    -> current task for BOM qualification
    -> konnect_pcb_builder
    -> konnect_design_reviewer (independent, normally read-only)
    -> konnect_bringup_planner (read-only)
```

Parallel agents can research unrelated facts, but two agents must not
concurrently edit the same schematic, board, library, or live KiCad session.

## Skill inventory

| Skill | Use it when | Primary outcome | Specialist agent |
|---|---|---|---|
| `konnect-codex` | Any KiCad/Konnect request needs routing to the correct guidance | Correct domain/workflow selection | Routes to the agents below |
| `konnect` | Any operation touches protected KiCad source or needs live/file ownership discipline | Integrity-preserving execution channel | None |
| `kicad-workflows` | The request spans phases or asks for an end result such as a complete board | Ordered phases, stop conditions, evidence, lifecycle cleanup | Coordinates handoffs |
| `kicad-library` | A symbol, footprint, library entry, pin map, package, model, or registration is missing or wrong | Verified reusable library asset | `konnect_library_builder` |
| `kicad-schematic` | A circuit must be created, modified, rewired, or made human-readable | Saved, readable, electrically checked schematic | `konnect_schematic_builder` |
| `kicad-bom` | MPNs, manufacturers, datasheets, alternates, lifecycle, sourcing, DNP, or assembly BOM data matter | Qualified and inspected BOM | Current task |
| `kicad-pcb` | Board transfer, outline, constraints, placement, routing, zones, or PCB ECO work is requested | Saved and validated PCB layout | `konnect_pcb_builder` |
| `kicad-review` | The design needs review, intake, recovery, or simulation-readiness analysis | Evidence-backed verdict and finding ledger | `konnect_design_reviewer` |
| `kicad-manufacture` | Gerbers, drills, position data, production BOM, or a frozen fabrication package is requested | Revision-bound inspected release package | Current task after review |
| `kicad-bringup` | Firmware mapping, proof-of-life, test points, startup behavior, or first-power planning is requested | Read-only firmware and bring-up handoff | `konnect_bringup_planner` |

## Router and integrity skills

### `konnect-codex`

This is the first Codex-facing router. It confirms that Konnect is available,
selects `kicad-workflows` for multi-stage outcomes, selects the owning domain
skill, and establishes the normal handoff order.

It also carries the evidence contract: a check, render, route, export, or agent
result cannot be reported as performed unless its direct result exists in the
current handoff. Missing, failed, skipped, unavailable, or contradictory
required evidence produces `INCOMPLETE` rather than an inferred pass.

### `konnect`

This is the integrity boundary. KiCad source files are serialized object
graphs, not ordinary text-editing targets. Every mutation of schematics,
boards, projects, symbols, footprints, and library tables goes through
Konnect's supported tools.

It distinguishes three access channels:

1. Konnect MCP for all mutations.
2. Exported netlists, BOMs, reports, and artifacts for normal analysis.
3. Read-only source inspection only for information unavailable through tools
   or exports.

It also documents the KiCad 10 capability boundary:

- PCB item mutation is primarily live IPC, with a small set of explicit safe
  closed-board contracts.
- Schematic item mutation uses Konnect's validated builder rather than KiCad
  item-level IPC.
- Symbol and footprint library mutation uses validated library operations.
- `kicad-cli` provides non-GUI checks and exports.

## Multi-stage workflow router

### `kicad-workflows`

Use this skill when the requested outcome crosses domain boundaries. Before the
first mutation it requires:

1. a named outcome and one primary workflow;
2. required inputs or safe, reversible assumptions;
3. ordered phases and phase stop conditions;
4. one current owner for the KiCad project;
5. a process-ownership baseline when external applications are involved; and
6. direct completion evidence for every required phase.

It does not turn a simple lookup or one-tool edit into a large ceremony. Those
remain direct domain-skill tasks.

### Process lifecycle and cleanup

Before launching or restarting KiCad, Freerouting, Java, a simulator, or
another helper, the workflow records relevant windows and processes and
classifies them as `pre-existing`, `task-owned`, or `ownership unknown`.

During execution it prevents duplicate restarts and gives probes and external
commands bounded timeouts. At completion it:

1. confirms the artifact is saved;
2. closes task-owned secondary windows and parent applications normally;
3. waits a bounded grace period;
4. inspects any survivor by PID, parent, and command line;
5. terminates only a verified task-owned orphan; and
6. reports what was closed, terminated, or deliberately preserved.

Active MCP/companion processes serving the current task and applications that
existed before the task are preserved. Ambiguous ownership makes cleanup
`INCOMPLETE`; it does not authorize broad process killing.

## Domain skills in detail

### `kicad-library`

Use this skill for library search, custom symbols, footprints, metadata, pin
and pad numbering, 3D-model associations, and project or global library
registration.

The custom-part workflow requires the exact manufacturer part and package
suffix, authoritative datasheet, stated drawing view, intended library scope,
and required metadata. It searches installed libraries first and creates a new
asset only when no verified match exists.

The acceptance evidence is a physical mapping table:

```text
datasheet physical lead
    -> electrical function
    -> symbol pin number/name/type
    -> footprint pad number and coordinates
```

Bottom-view packages, tubes, displays, connectors, sockets, and polarity-
sensitive parts receive a second explicit walk around the drawing direction.
The workflow reads the assets back, inspects the footprint, proves the library
identifiers resolve at the requested scope, and stops on ambiguous viewing
direction, mismatched pad count, or unverified registration.

Library-update propagation is separate. It inventories affected projects,
classifies the change as graphical, geometrical, or electrical, previews
conflicts where possible, applies to one project at a time, and reruns relevant
ERC/connectivity/footprint/DRC checks. A corrected global library item does not
silently rewrite embedded project copies.

Agent: `konnect_library_builder` owns a substantial custom-part phase and
returns verified library identifiers and complete mapping evidence.

### `kicad-schematic`

Use this skill for project bootstrap, circuit construction, component
placement, wiring, labels, power rails, hierarchy, and schematic cleanup.

The workflow starts with named functional blocks, interfaces, rail contracts,
sheet or bounded-region ownership, and intended signal flow. It searches
standard libraries and templates before creating local assets.

Readable layout is a hard acceptance gate, not decoration. A complete
schematic must:

- group symbols and their support parts by functional block;
- keep local wires where they communicate circuit behavior;
- use labels at meaningful block, repeated-signal, or hierarchy boundaries;
- keep labels, fields, notes, no-connects, and wires within block closure;
- fit the page frame without visible overlaps; and
- render every affected sheet for actual visual inspection.

Electrical validation includes ERC, short detection, wire and component
connection checks, exported connectivity when needed, and reconciliation of
known orphan-checker false positives. A visually cleaner sheet is not complete
if connectivity drifted; an aggregate pass does not override a confirmed short
or ERC failure.

Agent: `konnect_schematic_builder` owns a complete construction or substantial
cleanup phase and returns block, grouping, render, ERC, connectivity,
contradiction, and waiver evidence.

### `kicad-bom`

Use this skill when part identity and production data matter. The schematic's
component properties remain the maintained source of truth; CSV files and
catalog searches are views, not competing authorities.

The workflow inventories BOM health, establishes required fields, qualifies
critical line items against exact MPN/package datasheets, checks alternates for
electrical and footprint compatibility, applies reviewed property changes, and
inspects the exported BOM.

Stock, price, lead time, and lifecycle are time-sensitive evidence and require
a source and observation date. Distributor inventory does not prove active
manufacturer lifecycle. A family datasheet or wrong package does not qualify
an exact part.

Current limitation: Konnect 0.12.1 does not expose a dedicated mutation for
KiCad's native DNP attribute. A custom text field named `DNP` must not be
treated as equivalent. When native DNP state must change, the workflow reports
the smallest manual KiCad step and verifies the resulting export.

Outcome: `QUALIFIED`, `NOT QUALIFIED`, or `INCOMPLETE` with exact component
gaps and the export path.

### `kicad-pcb`

Use this skill for schematic-to-PCB transfer, board constraints, component
placement, routing, zones, stackups, silkscreen, and engineering changes.

The ordered PCB path is:

1. verify the board outline and mechanical constraints;
2. inventory footprint references, pads, graphics, layers, and 3D models;
3. dry-run and revision-bind schematic transfer;
4. verify the same inventory after transfer;
5. refresh changed libraries through a reviewed dry-run/apply cycle;
6. place by functional block with mechanical anchors locked;
7. close the visible placement gate;
8. route a complete board with Freerouting by default;
9. accept or reject the imported route using direct evidence;
10. complete layout-physics acceptance against actual routed layers and planes;
11. add/refill zones only after routing is accepted;
12. run final DRC and inventory checks;
13. save and re-query; and
14. close the process-lifecycle gate.

Placement planners are deterministic starting points, not automatic approval.
The workflow records a pre-score, reviews dry-run movement and the `held` set,
applies only the reviewed plan, independently re-scores, and visually inspects
pads, holes, courtyards, edge clearances, connectors, test points, silkscreen,
and model coverage. Blocking overlaps prevent routing regardless of score.

Freerouting is the default for a complete board because it provides global
obstacle avoidance, rip-up/retry, and congestion management. Segment tools are
for deliberate isolated work or a small understood repair, not a substitute
whole-board autorouter.

The route workflow first tests Konnect's revision-bound native export on the
actual board. When the v0.12.1 profile rejects common legal KiCad geometry, the
companion offline bridge saves the checkpoint, requires PCB Editor to be
closed, exports with KiCad's own DSN API, runs Freerouting headlessly, imports
SES, and writes a separate routed board. A standalone JAR without working DSN
export and SES import is not a complete workflow.

Route acceptance requires unchanged placement and footprint inventory,
plausible trace counts by net/layer, no shorts, direct DRC, no required
unrouted connection, current zones, visual inspection, and a completed
layout-physics applicability/evidence matrix. An autorouter success message or
clean DRC alone is not acceptance.

Engineering changes use a before/after checkpoint and preserve unaffected
placement and routing. Unexpected collateral change produces `INCOMPLETE`.

Agent: `konnect_pcb_builder` owns substantial transfer, placement, routing,
zone, and PCB ECO work and returns the stable saved board to an independent
reviewer.

### `kicad-review`

Use this skill for existing-project intake, quick checks, comprehensive design
review, pre-fabrication readiness, project recovery, or simulation-readiness.

Direct evidence outranks aggregate verdicts. Review correlates ERC, DRC,
connectivity, shorts, unrouted count, trace inventory, footprint/pad inventory,
zones, artifacts, component data, and requirements. Contradictions are visible
findings; they are not averaged into a pass.

Findings are classified as `CRITICAL`, `WARNING`, or `SUGGESTION` with evidence
basis, confidence, exact location, and smallest safe fix. Review reports checks
not performed and compares fixed, still-open, new, waived, and unverifiable
findings when prior evidence exists.

Existing-project intake records paths, editor ownership, inventory, artifacts,
and a safe baseline before mutation. Project recovery inventories processes,
locks, autosaves, backups, and timestamps before stopping a proven stale
session or reopening a recovered design.

Simulation readiness is intentionally bounded. It identifies models, sources,
parameters, pin mappings, observables, and unsupported devices. Konnect 0.12.1
does not prove that a simulation ran; model presence cannot be reported as a
simulation result.

Agent: `konnect_design_reviewer` normally operates independently and read-only
after design mutations finish. Verdicts are `READY FOR FAB`, `NOT READY`, or
`INCOMPLETE`.

### `kicad-manufacture`

Use this skill only after the design and BOM are stable enough to freeze a
revision. It records source revision and hashes, runs the strongest applicable
checks, inspects schematic and PCB renders, exports requested files, and
inspects the resulting package.

Typical artifacts include Gerbers, drills, position files, BOM, drawings, and
a release manifest containing filenames, sizes, layers, hashes, tool versions,
and validation results.

Time-sensitive fabricator rules must be verified against the selected
manufacturer's current official capabilities. Static prose is only a starting
reference. Missing, empty, stale, contradictory, or uninspected artifacts make
the release `NOT READY` or `INCOMPLETE`.

### `kicad-bringup`

This is a read-only post-review handoff. It does not mutate KiCad or energize
hardware.

The workflow extracts controller pins, peripheral buses, rails, regulator
limits, programming interfaces, boot/reset controls, indicators, buttons, and
test points. It produces:

- a GPIO table with MCU pin, schematic net, direction, active level, pull/reset
  state, peripheral, voltage domain, load, and safe startup state;
- a test-point table with reference/pad, net, expected resistance, powered
  voltage or waveform, probe ground, and acceptance range; and
- a staged plan from visual/unpowered inspection through current-limited rails,
  reset/programming, proof-of-life, interfaces, and finally loads.

The agent never claims a physical measurement occurred. It defines planned
measurements, current/voltage stop conditions, recovery behavior, and unresolved
assumptions. Verdicts are `READY FOR CONTROLLED BRING-UP`, `NOT READY`, or
`INCOMPLETE`.

## Workflow catalog

| Workflow | Trigger and required inputs | Ordered result and evidence |
|---|---|---|
| Project bootstrap | New project plus functional, interface, rail, mechanical, part, and manufacturing requirements | Non-destructive creation, sheet/block plan, dependency list, initial validation |
| Requirements to schematic | Circuit requirements or reference design | Verified libraries, block-based construction, rendered readability, ERC/connectivity evidence |
| Schematic cleanup | Electrically useful but visually poor sheet | Connectivity baseline, block-first rearrangement, render comparison, unchanged electrical evidence |
| Custom part | Missing or suspect exact symbol/footprint | Datasheet mapping table, verified symbol/footprint, resolvable library IDs |
| Library-update propagation | Corrected shared item must reach projects | Change classification, per-project preview/apply, ERC/DRC and conflict report |
| BOM qualification | Production identity, sourcing, lifecycle, alternates, DNP, or assembly data | Qualified schematic properties and inspected BOM export |
| Schematic to PCB | Validated schematic must become a board | Revision-bound transfer with footprint, pad, graphic, layer, and model invariants |
| PCB constraint setup | Board dimensions, stackup, rules, power, or mechanical requirements | Saved and queried outline, rules, net classes, keepouts, fabrication constraints |
| Component placement | Unplaced or substantially rearranged board | Locked anchors, score/dry-run/apply evidence, visible non-overlapping checkpoint |
| PCB routing | Complete board or interacting nets | Freerouting provenance, route inventory, no shorts/unrouted work, direct DRC |
| Engineering change | Accepted placement/routing must be modified | Before/after delta with unaffected layout preserved and collateral change rejected |
| Existing-project intake | Unknown or inherited KiCad project | Paths, ownership, inventory, checks, renders, artifacts, and safe baseline |
| Simulation readiness | User wants to simulate or assess feasibility | Model/pin/source/observable gap report; no false claim that simulation ran |
| Design review | Audit or readiness decision | Evidence package, finding ledger, contradictions, limits, formal verdict |
| Project recovery | Crash, stale process, lock, autosave, or uncertain state | Candidate preservation, ownership evidence, recovered inventory, validation, cleanup |
| Manufacturing release | Fabrication/assembly files requested | Frozen revision, inspected outputs, hashes, manifest, `READY`/`NOT READY` |
| Hardware bring-up | Reviewed controller board needs firmware/first-power handoff | GPIO/test-point tables and staged read-only bring-up plan |

## Common end-to-end paths

### New board from requirements

```text
requirements
  -> project bootstrap
  -> custom parts where verified standard assets do not exist
  -> requirements-to-schematic
  -> BOM qualification
  -> schematic-to-PCB transfer
  -> constraints
  -> placement gate
  -> Freerouting route and acceptance
  -> independent design review
  -> manufacturing release
  -> optional firmware/bring-up handoff
```

### Correct a reusable part

```text
exact datasheet and package view
  -> custom-part physical mapping
  -> symbol/footprint correction
  -> readback and visual verification
  -> library registration proof
  -> controlled project-by-project propagation
  -> schematic/PCB validation
```

### Resume a troubled project

```text
existing-project intake
  -> process/lock/autosave recovery when needed
  -> saved-state and inventory baseline
  -> owning domain workflow or PCB ECO
  -> independent review
```

## Specialist agents

| Agent | Owns | Required handoff evidence | Explicit boundary |
|---|---|---|---|
| `konnect_library_builder` | Custom symbol/footprint creation or correction | Exact datasheet identity, lead-to-pin-to-pad table, readback, visible numbering, registration | Does not build the rest of the circuit unless separately delegated |
| `konnect_schematic_builder` | Complete schematic build or substantial cleanup | Functional blocks, grouping/regions, renders, ERC, shorts, connectivity, waivers | Does not overlap another mutation owner |
| `konnect_pcb_builder` | Transfer, constraints, placement, routing, zones, PCB ECO | Transfer invariants, placement checkpoint, router provenance, trace/unrouted/DRC evidence, cleanup | Does not hide schematic/library defects with board-only workarounds |
| `konnect_design_reviewer` | Comprehensive independent review | Raw direct checks, findings, evidence basis, confidence, limitations, verdict | Read-only unless fixes are separately authorized |
| `konnect_bringup_planner` | Firmware and first-power planning | GPIO/test-point tables, staged limits and pass/fail evidence | Does not mutate the design or touch physical hardware |

## Hooks

### Relevant-prompt hook

`UserPromptSubmit` detects relevant KiCad/Konnect language and injects concise
router, skill, evidence, agent-order, placement, and Freerouting guidance. It
does not mutate the design or start every agent.

### Pre-tool ownership hooks

| Class | Examples | Required state |
|---|---|---|
| Live or safe closed-board fallback | placement, movement, outline, zone creation, selected graphics deletion | Use live IPC when present; documented fallback only when ownership is safe |
| Closed-board | footprint flip and predefined editor sizes | PCB Editor must not own the board |
| Dry-run/apply | schematic transfer, library refresh, legacy repair | Review a plan and apply its exact revision under ownership rules |
| Live-only | trace mutation, vias, routing support, zone refill | Exactly one responsive live PCB Editor owns the target |

Hooks provide context before the call. Konnect's server-side identity, liveness,
revision, and conflict validation remains authoritative. Process lifecycle
guidance complements hooks by tracking applications from start through cleanup;
it does not assume a hook supervises every spawned child.

## Evidence and completion vocabulary

| State | Meaning |
|---|---|
| `PASS`, `ACCEPTED`, `QUALIFIED`, `READY` | Every required artifact and direct gate for the phase is present |
| `NOT QUALIFIED`, `NOT READY`, `BLOCKED` | A confirmed defect, unmet requirement, or prerequisite prevents acceptance |
| `INCOMPLETE` | Required evidence or capability is missing, failed, unavailable, ambiguous, or contradictory |
| Waived | The user accepted a specific non-blocking finding with evidence and impact recorded |

An aggregate score or review cannot overrule a confirmed short, ERC/DRC error,
unrouted required net, corrupted transfer inventory, wrong physical pin map,
missing requested artifact, or unresolved process ownership.

## Reference map

- `kicad-workflows/references/workflow-catalog.md` — workflow selection.
- `kicad-workflows/references/process-lifecycle.md` — ownership and cleanup.
- `kicad-library/references/custom-part-acceptance.md` — physical pin and pad acceptance.
- `kicad-library/references/workflows.md` — custom-part and propagation paths.
- `kicad-schematic/references/workflows.md` — bootstrap, construction, cleanup.
- `kicad-schematic/references/schematic-layout-acceptance.md` — visual/grouping gate.
- `kicad-schematic/references/wiring-patterns.md` — wiring and label patterns.
- `kicad-schematic/references/common-lib-ids.md` — non-exhaustive shortcut cache.
- `kicad-pcb/references/workflows.md` — transfer, constraints, placement, routing, ECO.
- `kicad-pcb/references/placement-acceptance.md` — visible placement gate.
- `kicad-pcb/references/freerouting-workflow.md` — DSN/SES and route acceptance.
- `kicad-pcb/references/eco-workflow.md` — preserve accepted layout during changes.
- `kicad-pcb/references/power-layout.md` — power, thermal, motors, converters, noisy loads.
- `kicad-pcb/references/pcb-layout-physics-acceptance.md` — return paths,
  critical loops, current/thermal capacity, RF constraints, via process, and DFT evidence.
- `kicad-pcb/references/design-rules.md`, `trace-width-table.md`, and
  `layer-reference.md` — non-authoritative starting references.
- `kicad-review/references/workflows.md` — intake, recovery, simulation, review.
- `kicad-review/references/review-methodology.md`, `design-checklist.md`,
  `error-taxonomy.md`, and `evidence-package.md` — repeatable review evidence.
- `kicad-manufacture/references/manufacturing-release-workflow.md` — frozen release.
- `kicad-manufacture/references/gerber-layers.md`, `jlcpcb-rules.md`, and
  `legacy-through-hole.md` — artifact, fabricator, and manual-assembly branches.

## Current known boundaries

- The companion is reviewed for Konnect 0.12.1. A new Konnect release requires
  a compatibility and guidance-delta review.
- Konnect 0.12.1 has no dedicated native-DNP mutation.
- The companion's offline DSN/SES bridge remains necessary until an equivalent
  released Konnect path passes the route benchmark.
- A standalone Freerouting JAR is insufficient without DSN export and SES import.
- Simulation-readiness guidance does not prove a simulator ran.
- Standard library shortcut lists are discovery caches, not complete or
  project-specific favorites.
- 3D-model presence is a transfer and review invariant when expected; the
  workflow does not invent a model for a custom part.
- Desktop control is optional workflow glue. When unavailable, the skill gives
  the smallest manual step and resumes with direct validation.

## Release durability

`policy/enhancements.json` records every companion-owned behavior, its evidence,
assertions, and retirement condition. `docs/GUIDANCE_DELTA_REGISTER.md` is the
human decision surface. For each Konnect release, every active behavior must be
retained, revised, retired with benchmark evidence, or replaced by a newly
documented requirement.

Packaging tests verify that references are reachable, agents parse, policy IDs
and the living register agree, hooks match the reviewed runtime contract, and
installed files are healthy. After source changes:

```text
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --locked --all-targets -- -D warnings
cargo run -- sync --dry-run
cargo run -- sync
cargo run -- doctor
```

The final `sync` publishes repository guidance into the installed personal
plugin. Existing tasks may retain already-loaded guidance; a new or restarted
Codex task guarantees a fresh load.
