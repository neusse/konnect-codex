# konnect-codex plugin v0.7.0 — companion revision 1

This release is reviewed for
[Konnect v0.7.0](https://github.com/mixelpixx/Konnect/releases/tag/v0.7.0) at
commit `8e458d43602e7979bdc0e4456ce9c0cbe3eb2fe4`.

Konnect 0.7.0 fixes the schematic-to-board corruption that converted footprint
graphics into phantom pads, reads the applied IPC state back, requires DRC
evidence for review/manufacturing verdicts, preserves all DRC result categories,
and adds guarded closed-board move, rotate, and flip behavior. The upstream
guidance review found one changed asset: `kicad-pcb/SKILL.md`, updated for those
closed-board operations. The companion PCB skill now carries that contract while
retaining its stricter single-owner, transfer-inventory, placement, Freerouting,
and route-acceptance gates.

The existing transfer-integrity and contradictory-verdict enhancements remain
active until the v0.7 behavior is exercised in the companion's end-to-end KiCad
benchmark; the source fixes alone do not retire a proven workflow gate.

The companion now reports retained MCP process pairs through `sessions`,
retires only verified companion-to-Konnect pairs through `stop-sessions`, and
warns about duplicate sessions in `doctor`. On Windows, every newly launched
Konnect server is assigned to a kill-on-close Job Object so an unexpectedly
terminated adapter cannot leave its child behind. These safeguards make locked
binary recovery local and explicit while still allowing separate active Codex
tasks to hold independent MCP sessions.

Companion revision 6 adds evidence-grounded review methodology and a dedicated
KiCad BOM qualification skill. Comprehensive reviews now record confirmed
design context, evidence basis, confidence, exact-part datasheet status,
explicit review limits, false-positive dispositions, and revision-to-revision
finding status. BOM work now maintains schematic properties as the source of
truth, qualifies exact MPNs and alternates, distinguishes lifecycle evidence
from time-sensitive stock, and directly verifies the exported BOM.

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
restarts. Konnect 0.6.1 through 0.7.0 silently reinstalls its native Codex skills whenever
the `.installed-codex` marker is absent, reversing an explicit uninstall. The
plugin now owns a reversible suppression guard while enabled, repairs it before
every MCP launch, reports it through `doctor`, and restores the marker's prior
state on disable or uninstall. The upstream behavior is tracked in
[Konnect #242](https://github.com/mixelpixx/Konnect/issues/242).

## Included

- Eight reviewed Codex-native KiCad workflow skills and one execution router.
- Five Codex agents for custom libraries, complete schematic construction, PCB
  layout, independent design review, and read-only firmware/bring-up handoff.
- Codex-native hooks and eager discovery of the complete Konnect MCP catalogue.
- Reversible sync, disable, enable, doctor, and uninstall operations.
- Scoped MCP session inspection and cleanup, with Windows child-process
  ownership for abnormal adapter exits.
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
- A pre-install gate that requires Konnect to be present and exactly v0.7.0
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
