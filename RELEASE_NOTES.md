# konnect-codex plugin v0.6.1 — companion revision 5

This release is reviewed for
[Konnect v0.6.1](https://github.com/mixelpixx/Konnect/releases/tag/v0.6.1) at
commit `506abe094204c6d4acd77415892e9e0e8fdb35fb`.

Konnect 0.6.1 fixes the KiCad crash caused by footprint graphics on
`Dwgs.User` and makes `konnect init --help` non-destructive. Its bundled
skills, references, agents, and hook are byte-for-byte unchanged from 0.6.0,
so the companion's existing Codex enhancements remain applicable.

Companion revision 5 closes the workflow gaps exposed by the DR2000 benchmark.
It adds custom-part physical pin-map acceptance, a dedicated library builder,
a firmware/first-power bring-up planner, visual placement approval, ECO and
power/thermal layout branches, legacy through-hole sourcing guidance, and a
repeatable raw review-evidence package.

Revision 5 also adds an executable Freerouting bridge. It detects KiCad's
bundled `pcbnew` Python API, Java, and the ActionPlugin or standalone engine
JAR; exports DSN, runs Freerouting, imports SES, and saves a separate
`.freerouted.kicad_pcb` without overwriting the source. Live PCB preflight
requires exactly one PCB Editor; offline routing requires none.

Companion revision 4 makes Freerouting the default whole-board routing strategy
for the PCB skill and `konnect_pcb_builder`. It adds a documented KiCad
ActionPlugin/Konnect bridge decision, a clean-placement gate before routing,
route-import inventory and DRC acceptance, stale trace-query detection, and a
hard stop when a live IPC phase falls back to closed-file mutation.

Revision 3 added `konnect_pcb_builder`, a dedicated PCB construction agent for
transfer integrity, board setup, placement, routing, zones, and direct layout
verification. The router and prompt hook use a deterministic schematic -> PCB
-> independent review sequence, with one agent owning the live KiCad project at
a time.

The revision retains revision 2's durable reviewed-mode installation across MCP
restarts. Konnect 0.6.1 silently reinstalls its native Codex skills whenever
the `.installed-codex` marker is absent, reversing an explicit uninstall. The
plugin now owns a reversible suppression guard while enabled, repairs it before
every MCP launch, reports it through `doctor`, and restores the marker's prior
state on disable or uninstall. The upstream behavior is tracked in
[Konnect #242](https://github.com/mixelpixx/Konnect/issues/242).

## Included

- Seven reviewed Codex-native KiCad workflow skills and one execution router.
- Five Codex agents for custom libraries, complete schematic construction, PCB
  layout, independent design review, and read-only firmware/bring-up handoff.
- Codex-native hooks and eager discovery of the complete Konnect MCP catalogue.
- Reversible sync, disable, enable, doctor, and uninstall operations.
- A compatibility audit that detects upstream guidance or hook drift.
- A machine-enforced guidance change policy with per-file upstream provenance,
  named Codex enhancements, evidence, and retirement criteria.
- Deterministic sequential delegation for full schematic builds, PCB transfer
  and layout, and final design reviews.
- Freerouting-first whole-board routing with explicit DSN/SES bridge selection,
  placement readiness, route-import acceptance, and local-repair boundaries.
- A non-overwriting offline Freerouting route bridge plus live/offline PCB
  ownership preflight.
- Datasheet-view-aware custom-part acceptance, ECO preservation, power/thermal
  layout, legacy/manual assembly, controlled bring-up, and raw evidence flows.
- Schematic collision/evidence gates, PCB transfer invariants, contradictory
  verifier handling, and direct manufacturing artifact verification derived
  from the safe-parts benchmark.
- Health output that reports plugin-managed and upstream-native agents
  separately.
- A pre-install gate that requires Konnect to be present and exactly v0.6.1
  before any plugin file is created or replaced.
- Downloadable Windows, Linux, and macOS archives with SHA-256 checksums.

For the easiest setup, open the README's
[Install with Codex](https://github.com/neusse/konnect-codex#install-with-codex)
section and paste its installation request into a Codex task. Codex will select
the platform archive, verify its checksum, install the plugin, and run the
health check. Manual installation remains documented immediately below it.

Konnect and its original hardware workflows are created and maintained by
[mixelpixx](https://github.com/mixelpixx). This plugin is an independent Codex
integration.
