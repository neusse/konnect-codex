# Konnect Codex Companion

`konnect-codex` converts the guidance installed for Claude by Konnect into a
Codex-native, reversible integration. It is deliberately separate from the
Konnect server so it can be removed when upstream Codex support reaches feature
parity.

It supplies the pieces that Konnect v0.5.0 does not yet install for Codex:

- a personal Codex plugin containing client-neutral copies of every installed
  Konnect skill;
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
or upstream-installed Codex skills.

## Build and install

```powershell
cargo install --path tools/konnect-codex
konnect-codex sync --konnect .\target\release\konnect.exe --config .\konnect.toml
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
