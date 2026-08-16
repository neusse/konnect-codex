# konnect-codex plugin v0.5.1

This release is reviewed for
[Konnect v0.5.1](https://github.com/mixelpixx/Konnect) at commit
`df6f2b0cb8ee5f266a17ae00cd7dcf95fb057150`.

## Included

- Six reviewed Codex-native KiCad workflow skills and one execution router.
- Two Codex agents for complete schematic construction and design review.
- Codex-native hooks and eager discovery of the complete Konnect MCP catalogue.
- Reversible sync, disable, enable, doctor, and uninstall operations.
- A compatibility audit that detects upstream guidance or hook drift.
- Downloadable Windows, Linux, and macOS archives with SHA-256 checksums.

Install the matching Konnect v0.5.1 server normally, without running
`konnect init --client codex`. Download the archive for your operating system,
place `konnect-codex` on `PATH`, run `konnect-codex sync`, and start a new Codex
task.

Konnect and its original hardware workflows are created and maintained by
[mixelpixx](https://github.com/mixelpixx). This plugin is an independent Codex
integration.
