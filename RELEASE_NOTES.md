# konnect-codex plugin v0.6.0

This release is reviewed for
[Konnect v0.6.0](https://github.com/mixelpixx/Konnect/releases/tag/v0.6.0) at
commit `2e5bbc2f0a2b16baa9aab89b94cceec7e472a1d6`.

## Included

- Six reviewed Codex-native KiCad workflow skills and one execution router.
- Two Codex agents for complete schematic construction and design review.
- Codex-native hooks and eager discovery of the complete Konnect MCP catalogue.
- Reversible sync, disable, enable, doctor, and uninstall operations.
- A compatibility audit that detects upstream guidance or hook drift.
- A pre-install gate that requires Konnect to be present and exactly v0.6.0
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
