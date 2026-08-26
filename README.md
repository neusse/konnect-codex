<p align="center">
  <img src="docs/assets/konnect-codex-banner.svg" alt="Konnect Codex Plugin — KiCad workflows adapted for Codex" width="100%">
</p>

<h1 align="center">Konnect Codex Plugin</h1>

<p align="center"><strong>A reviewed, versioned, first-class Codex plugin for Konnect.</strong></p>

<p align="center">
  <a href="https://github.com/neusse/konnect-codex/actions/workflows/ci.yml"><img alt="CI status" src="https://github.com/neusse/konnect-codex/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/neusse/konnect-codex/blob/main/LICENSE"><img alt="License: AGPL-3.0" src="https://img.shields.io/badge/license-AGPL--3.0-7c3aed.svg"></a>
  <img alt="Rust 1.85 or newer" src="https://img.shields.io/badge/Rust-1.85%2B-f97316.svg">
  <img alt="Codex plugin" src="https://img.shields.io/badge/Codex-plugin-10a37f.svg">
  <a href="https://github.com/mixelpixx/Konnect"><img alt="Reviewed for Konnect 0.9.0" src="https://img.shields.io/badge/Konnect-0.9.0-22d3ee.svg"></a>
  <a href="https://github.com/neusse/konnect-codex/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/neusse/konnect-codex"></a>
</p>

`konnect-codex` is a standalone **Codex plugin** for the Konnect server. It
packages reviewed Codex-native skills, agents, hooks, and MCP configuration so
Konnect behaves like a first-class Codex integration. It is deliberately
separate from Konnect so each release can preserve a known-good Codex workflow
without changing Konnect itself.

Release numbers match the Konnect release reviewed by the plugin:
`konnect-codex v0.9.0` supports `Konnect v0.9.0`. The exact reviewed upstream
commit and guidance fingerprints are recorded in
[`compatibility.json`](compatibility.json).

The plugin supplies:

- eight independently reviewed Codex skills plus the execution router,
  including dedicated BOM qualification and lifecycle guidance;
- a Codex execution-router skill that makes the eager-tool profile and safe
  KiCad workflow explicit;
- five reviewed Codex agents for custom libraries, schematic construction, PCB
  layout, independent design review, and firmware/bring-up handoff, without a
  hard-coded model;
- a Freerouting-first PCB workflow with placement, IPC ownership, route-import,
  and direct DRC acceptance gates;
- Codex-native hooks, relevant-prompt guidance, and a live-KiCad IPC fallback;
- a private Konnect configuration with `eager_toolsets = true` so clients that
  cache the first MCP tool list can see the complete tool catalogue;
- ownership, health checks, disable/enable, and exact uninstall support;
- a machine-enforced [guidance change policy](docs/CHANGE_POLICY.md) with
  per-file upstream provenance, named Codex enhancements, and retirement rules.

The reviewed plugin skills are authoritative by default. Do not run
`konnect init --client codex`; that installs another set of skills with the same
names. If they are already installed, remove only that native guidance with
`konnect uninstall --client codex` before syncing the plugin. An explicit
`--prefer-native-skills` option remains available for future compatibility
testing.

## Upstream and credit

<p align="center">
  <a href="https://github.com/mixelpixx/Konnect">
    <img src="docs/assets/konnect-upstream.svg" alt="Konnect — the upstream KiCad MCP server" height="96">
  </a>
  <br>
  <sub>Original Konnect artwork; displayed here with credit to the upstream project.</sub>
</p>

[Konnect](https://github.com/mixelpixx/Konnect) is created and maintained by
[mixelpixx](https://github.com/mixelpixx). Konnect provides the KiCad MCP server,
tool catalogue, file-safety model, and original hardware workflows on which
this plugin depends. `konnect-codex` is an independent Codex plugin and
does not replace or claim authorship of Konnect. Please report server and KiCad
tool issues to the upstream project and support its development there.

## Install with Codex

Open a Codex task and paste the following request. Codex can select the correct
release for the current operating system, verify it, install the plugin, and
run its health check for you:

```text
Install the konnect-codex v0.9.0 companion revision 1 plugin from
https://github.com/neusse/konnect-codex/releases/tag/v0.9.0 for this operating
system. First locate the Konnect executable and run `konnect --version`. Stop
without changing anything if Konnect is missing or is not exactly v0.9.0.
Download SHA256SUMS.txt and verify the plugin archive before extracting it.
Install the konnect-codex executable in a user-writable location on PATH. Do
not run `konnect init --client codex`. If Konnect's native Codex guidance is
already installed, remove only that guidance with `konnect uninstall --client
codex`. Preserve the Konnect server and its configuration. Run
`konnect-codex sync`, then `konnect-codex doctor`. Report the installed paths
and health result, and tell me when to start a new Codex task.
```

The installer registers `konnect-codex@personal` through Codex's plugin system.
Start a new Codex task after installation so Codex discovers the plugin's
skills, agents, hooks, and MCP server.

`sync` runs `konnect --version` before creating or replacing any plugin file.
Missing Konnect and every version other than the exact supported release stop
installation with a corrective error; existing plugin state is left unchanged.

## Manual installation

Install the matching Konnect release normally, without its Codex guidance flag.
Then download the archive for your operating system from
[GitHub Releases](https://github.com/neusse/konnect-codex/releases/latest), put
`konnect-codex` on `PATH`, and run:

```powershell
konnect --version            # must report: konnect 0.9.0
konnect-codex sync
konnect-codex doctor
```

You can also install the version-matched source release with Cargo:

```powershell
cargo install --git https://github.com/neusse/konnect-codex --tag v0.9.0
konnect-codex sync
konnect-codex doctor
```

Or install from a local checkout:

```powershell
cargo install --path .
konnect-codex sync --konnect C:\path\to\konnect.exe --config C:\path\to\konnect.toml
konnect-codex doctor
```

`sync` installs the reviewed assets embedded in this release; it does not
translate files from `~/.claude`. Passing `--source <path>` asks it to verify
that a Konnect checkout or installed asset directory still matches the reviewed
fingerprint before installation.

Codex requires the user to review and trust newly installed plugin hooks before
it runs them.

## Lifecycle

```powershell
konnect-codex disable       # remove active plugin and agents, retain generated state
konnect-codex enable        # reactivate the retained integration
konnect-codex audit         # verify installed Claude-side assets still match this release
konnect-codex native-status # compare native Konnect coverage with the plugin
konnect-codex sessions      # list active companion -> Konnect MCP process pairs
konnect-codex stop-sessions # retire those pairs before an upgrade or after a stale task
konnect-codex freerouting status # inspect KiCad Python, Java, and router bridge
konnect-codex pcb-preflight --board C:\path\board.kicad_pcb --mode live
konnect-codex uninstall     # remove only plugin-owned files and marketplace entry
```

For a complete-board route, close PCB Editor and run
`konnect-codex freerouting route --board C:\path\board.kicad_pcb`. The bridge
uses KiCad's native DSN/SES Python API and launches the installed Freerouting
engine headlessly, then
writes `board.freerouted.kicad_pcb`; it never overwrites the source board. Open
the generated board and pass the plugin's inventory, unrouted, short, and direct
DRC acceptance gate before adopting it.

`uninstall` verifies hashes before removing anything. If a managed file was
edited after installation, it stops and preserves the file. `--force` is
available only for intentionally discarding those plugin-owned edits.

While reviewed mode is enabled, the plugin maintains a reversible
`~/.konnect/.installed-codex` suppression marker. Konnect 0.6.1 through 0.9.0
otherwise silently reinstalls its six native Codex skills on every MCP start after an
uninstall. The companion repairs the marker before launching Konnect and
restores its prior state when disabled or uninstalled; it does not claim or
delete native skill files.

Each MCP launch owns its Konnect child process with an operating-system cleanup
boundary on Windows. If Codex retains obsolete MCP connections, `sessions`
shows the exact adapter/server PIDs and their owning process. `stop-sessions`
only stops a `konnect.exe` process whose direct parent is
`konnect-codex.exe`, then lets the waiting adapter exit normally; it does not
target Cargo, Rust, KiCad, standalone Konnect runs, or unrelated processes.
Run it before replacing either executable when an upgrade reports a locked
file. Codex starts a fresh session the next time the MCP server is needed.

## Generated locations

- Plugin source: `~/plugins/konnect-codex`
- Codex agents: `~/.codex/agents/konnect_*.toml`
- State/config: `~/.konnect/codex-companion`
- Marketplace entry: `~/.agents/plugins/marketplace.json`

The plugin is installed as `konnect-codex@personal` using the Codex CLI. Native
Konnect skills under `~/.agents/skills` are neither replaced nor removed; use
Konnect's client-scoped uninstall command if you previously installed them.

## Release review process

Every Konnect release is handled as a compatibility review under the
[guidance change policy](docs/CHANGE_POLICY.md):

1. Compare the new per-file upstream baseline with the previous release and
   account for every changed skill, reference, agent, hook, and tool contract.
2. Review every entry in [`policy/enhancements.json`](policy/enhancements.json):
   port it, adapt it, or retire it only after its recorded condition is met.
3. Run `konnect-codex audit --source <Konnect checkout>` against the pinned
   version, commit, per-file baseline, aggregate guidance, and hook fingerprint.
4. Validate enhancement assertions, skill frontmatter, agent TOML, lifecycle
   safety, tests, Clippy, and packaging on Windows, Linux, and macOS.
5. Run the end-to-end KiCad benchmark before publishing matching artifacts.
