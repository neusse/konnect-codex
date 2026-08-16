<p align="center">
  <img src="docs/assets/konnect-codex-banner.svg" alt="Konnect Codex Companion — KiCad workflows adapted for Codex" width="100%">
</p>

<h1 align="center">Konnect Codex Companion</h1>

<p align="center"><strong>Turn Konnect's Claude integration into a reversible, capability-complete Codex plugin.</strong></p>

<p align="center">
  <a href="https://github.com/neusse/konnect-codex/actions/workflows/ci.yml"><img alt="CI status" src="https://github.com/neusse/konnect-codex/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/neusse/konnect-codex/blob/main/LICENSE"><img alt="License: AGPL-3.0" src="https://img.shields.io/badge/license-AGPL--3.0-7c3aed.svg"></a>
  <img alt="Rust 1.85 or newer" src="https://img.shields.io/badge/Rust-1.85%2B-f97316.svg">
  <img alt="Codex plugin" src="https://img.shields.io/badge/Codex-plugin-10a37f.svg">
  <a href="https://github.com/mixelpixx/Konnect"><img alt="Konnect 0.5 or newer" src="https://img.shields.io/badge/Konnect-0.5%2B-22d3ee.svg"></a>
</p>

`konnect-codex` complements Konnect's native Codex skill installation with a
reversible plugin, converted agents, and hooks. It is deliberately separate
from the Konnect server so it can be removed when upstream Codex support reaches
feature parity.

Konnect v0.5.1 installs its six shared skills natively with
`konnect init --client codex`. The companion detects that client-scoped install
and reuses those skills instead of registering duplicate copies. It supplies the
pieces that v0.5.1 does not yet install for Codex:

- a personal Codex plugin containing the MCP and hook integration;
- a Codex execution-router skill that makes the eager-tool profile and safe
  KiCad workflow explicit;
- Codex TOML versions of every Konnect-supplied Claude agent, without a
  hard-coded model;
- Codex JSON versions of every Konnect hook found in Claude's installed
  settings, plus relevant-prompt guidance and a live-KiCad IPC fallback;
- a private Konnect configuration with `eager_toolsets = true` so clients that
  cache the first MCP tool list can see the complete tool catalogue;
- ownership, health checks, disable/enable, and exact uninstall support.

The companion does not modify the normal Konnect configuration, Claude files,
or upstream-installed Codex skills. If native skills are not installed, it
retains its compatibility fallback and creates client-neutral skill copies in
the plugin.

## Build and install

Install directly from GitHub:

```powershell
konnect init --client codex
cargo install --git https://github.com/neusse/konnect-codex
konnect-codex sync
konnect-codex doctor
```

Or install from a local checkout:

```powershell
cargo install --path .
konnect-codex sync --konnect C:\path\to\konnect.exe --config C:\path\to\konnect.toml
konnect-codex doctor
```

By default `sync` reads Konnect's installed assets from `~/.claude`. Use
`--source <path>` for a checkout's `crates/konnect/assets` directory or another
directory containing `skills/` and `agents/`.

Start a new Codex task after `sync` so the app discovers the plugin, skills,
agents, and MCP tools. Codex requires the user to review and trust newly
installed plugin hooks before it runs them.

## Lifecycle

```powershell
konnect-codex disable       # remove active plugin and agents, retain generated state
konnect-codex enable        # reactivate the retained integration
konnect-codex native-status # compare native Konnect coverage with the companion
konnect-codex uninstall     # remove only companion-owned files and marketplace entry
```

`uninstall` verifies hashes before removing anything. If a managed file was
edited after installation, it stops and preserves the file. `--force` is
available only for intentionally discarding those companion-owned edits.

## Generated locations

- Plugin source: `~/plugins/konnect-codex`
- Codex agents: `~/.codex/agents/konnect_*.toml`
- State/config: `~/.konnect/codex-companion`
- Marketplace entry: `~/.agents/plugins/marketplace.json`

The plugin is installed as `konnect-codex@personal` using the Codex CLI. Native
Konnect skills under `~/.agents/skills` are neither replaced nor removed.
