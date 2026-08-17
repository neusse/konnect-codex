# konnect-codex plugin v0.6.1 — companion revision 2

This release is reviewed for
[Konnect v0.6.1](https://github.com/mixelpixx/Konnect/releases/tag/v0.6.1) at
commit `506abe094204c6d4acd77415892e9e0e8fdb35fb`.

Konnect 0.6.1 fixes the KiCad crash caused by footprint graphics on
`Dwgs.User` and makes `konnect init --help` non-destructive. Its bundled
skills, references, agents, and hook are byte-for-byte unchanged from 0.6.0,
so the companion's six preexisting Codex enhancements remain applicable; this
revision adds a seventh lifecycle enhancement.

Companion revision 2 makes reviewed-mode installation durable across MCP
restarts. Konnect 0.6.1 silently reinstalls its native Codex skills whenever
the `.installed-codex` marker is absent, reversing an explicit uninstall. The
plugin now owns a reversible suppression guard while enabled, repairs it before
every MCP launch, reports it through `doctor`, and restores the marker's prior
state on disable or uninstall. The upstream behavior is tracked in
[Konnect #242](https://github.com/mixelpixx/Konnect/issues/242).

## Included

- Six reviewed Codex-native KiCad workflow skills and one execution router.
- Two Codex agents for complete schematic construction and design review.
- Codex-native hooks and eager discovery of the complete Konnect MCP catalogue.
- Reversible sync, disable, enable, doctor, and uninstall operations.
- A compatibility audit that detects upstream guidance or hook drift.
- A machine-enforced guidance change policy with per-file upstream provenance,
  named Codex enhancements, evidence, and retirement criteria.
- Deterministic sequential delegation for full schematic builds and final
  design reviews.
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
