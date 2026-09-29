# Konnect-to-Codex guidance delta register

This living document answers four questions for every companion release:

1. What does `konnect-codex` intentionally add or correct?
2. What evidence caused the difference?
3. How is the difference protected from release drift?
4. What verified upstream behavior would let us revise or retire it?

`policy/enhancements.json` is the executable source of truth for IDs, status,
assertions, evidence, and retirement conditions. This document is the human
review surface. Every active or retired policy ID must appear here, and tests
fail if either side loses an entry.

## Release review state

- Supported Konnect: `0.12.1`, commit
  `fa62e1ccb9eba359519bf8e3eab53a6cffeee33c`
- Companion release: `v0.12.1-codex.2` (`companion_revision = 2`)
- Last full guidance review: 2026-09-29
- Upstream guidance issues reviewed: #356, #357, #358 and the v0.12.1
  guidance delta
- Upstream issue #358 correction: Konnect v0.12.1 still registers
  `refill_zones` under `pcb_export`; the companion therefore keeps that real
  tool in the live-only hook class. The issue's broader structured-output and
  runtime-classification findings still apply.

## Active decisions

| Policy ID | Companion behavior retained or added | Verification / retirement review |
|---|---|---|
| `agent-delegation` | Deterministic Codex specialist handoffs | Router assertion; retire for equivalent native Codex routing |
| `schematic-evidence-and-collision-gate` | Reconcile orphan false positives and block stub-created shorts | Skill/agent assertions and schematic benchmark |
| `schematic-layout-readability-gate` | Functional blocks, hand-wired local passive topology, native-or-managed closure status, manifest-driven block relocation, structured acceptance record, label-inclusive visual gate, and independent schematic review | Skill/reference/builder/reviewer assertions and rendered benchmark |
| `pcb-transfer-integrity` | Preserve pad/graphic/layer/model invariants across transfer | PCB skill assertion and transfer benchmark |
| `contradictory-verifier-gate` | Direct evidence outranks aggregate passes | Review/manufacture/agent assertions |
| `requirements-based-review-defaults` | Datasheets and requirements control conditional design advice | Reviewer assertion |
| `doctor-agent-reporting` | Report companion and native agents separately | Doctor tests |
| `pcb-builder-delegation` | Give transfer/layout/routing to one PCB owner | Router/agent assertions |
| `freerouting-first-routing` | Use Freerouting for complete boards; local segments only for repairs | PCB skill/reference/agent assertions and route benchmark |
| `offline-freerouting-bridge` | Use KiCad-native DSN/SES as a non-overwriting fallback when the v0.12.1 native profile rejects common legal board geometry | CLI tests, actual-board export/route benchmark, and post-route acceptance |
| `pcb-live-state-and-placement-gates` | Stop on IPC ownership loss and require visible placement acceptance | PCB/reviewer assertions and preflight tests |
| `custom-part-physical-pin-acceptance` | Require view-aware datasheet lead-to-pad proof | Library reference/agent assertions |
| `visual-placement-checkpoint` | Require a reviewed 2D placement artifact before routing | PCB reference/agent assertions |
| `pcb-ownership-preflight` | Check process ownership before live/offline work | CLI and PCB skill assertion |
| `eco-and-power-layout-branches` | Preserve accepted ECO state and calculate power/thermal constraints | PCB references and benchmark |
| `firmware-bringup-handoff` | Provide read-only firmware and staged first-power handoff | Bring-up skill/agent assertions |
| `legacy-sourcing-and-review-evidence` | Track lifecycle/socket/manual-assembly risk and raw evidence packages | Manufacture/review references |
| `evidence-grounded-review-methodology` | Record context, evidence basis, confidence, limits, and review delta | Review skill/reference/agent assertions |
| `bom-lifecycle-workflow` | Qualify MPN/datasheet/alternate/lifecycle data and verify BOM export | BOM/router assertions |
| `v0.10-feedback-acceptance-integration` | Convert placement scores, v0.11 held sets, and visual baselines into independent acceptance gates | Skill/agent assertions and placement benchmark |
| `v0.9-known-safety-gates` | Preserve the remaining #315 and #328 workarounds | Release-specific assertions; #326 and #331 retired in v0.11 |
| `reference-reachability-and-evidence-contracts` | Link every reference, align agents with skills, correct manufacturing claims, and forbid invented evidence | Reachability and evidence-phrase tests; upstream #357 |
| `codex-hook-contract` | Emit structured Codex context and classify each matched PCB tool by runtime ownership contract | Hook-policy/matcher/output tests; upstream #358 findings adapted for Codex |
| `guidance-governance-register` | Require this living register and stable guidance standards on every release | Bidirectional policy/register test |
| `explicit-workflow-routing` | Route multi-stage outcomes through a discoverable catalog with ordered phases, stop conditions, direct evidence, and explicit outcomes | Skill/reference reachability, policy assertions, and workflow benchmarks |
| `owned-process-lifecycle-cleanup` | Baseline pre-existing versus task-owned applications, prevent duplicate restarts, and close or terminate only verified task-owned editors and helpers before completion | Shared lifecycle reference, PCB/Freerouting/recovery agent assertions, and orphan-process regression scenarios |
| `pcb-layout-physics-acceptance` | Require context-calibrated evidence for return planes, critical loops, thermal/current paths, RF/edge/stitching constraints, via process, and DFT access beyond DRC | PCB reference, builder, reviewer, release, and routed-board benchmarks |
| `benchmark-ledger-completeness` | Keep a gate-by-gate project-local record of failures, recovery, artifacts, cleanup, and terminal outcome | Workflow assertion and completed benchmark artifact inventory |

## v0.12.1 review decisions

- **Revise** schematic readability guidance for companion revision 2: local
  passive networks must be visibly hand-wired inside their parent blocks, the
  builder must return a structured acceptance record, and the independent
  reviewer must render and apply the same layout gate rather than relying on ERC.
- **Revise** block-movement guidance after upstream #315 was dispositioned
  `wontfix`: retain honest native-group limitations, but make agent-operated
  relocation executable through a complete closure manifest, bounded
  reconstruction, cleared-source check, and electrical/visual equivalence gate.
- **Retain** the active companion delta set. The Specctra exporter and
  bridge-selection code did not expand to the legal roundrect and unnumbered
  NPTH geometry covered by the compatibility bridge.
- **Revise** PCB guidance and hooks for native live-IPC `flip_component` on
  KiCad 10.0.6+, while retaining the guarded closed-board fallback when no live
  editor owns the board.
- **Revise** placement guidance: the force-directed planner is deprecated and
  diagnostic-only; decoupling plans require exact references and must remain
  blocked when out of bounds or non-improving.
- **Revise** schematic guidance to verify atomic junction reconciliation for
  batch symbol placement.
- **Resolve the upstream packaging gap** recorded for v0.12.0: Konnect v0.12.1
  now ships the linked reliability-contract reference. The companion retains
  its broader Codex evidence contract as an intentional independent review
  path, not as a workaround for a missing upstream file.

## v0.12.0 review decisions

- **Retain** the complete active companion delta set. The Specctra exporter and
  bridge-selection code did not change from v0.11.1, so the proven
  non-overwriting routing fallback remains required.
- **Revise** the library, manufacturing, PCB, review, and schematic guidance to
  consume v0.12.0's footprint-pad geometry, verified artifact manifest,
  JLCPCB correction/preview, DRC ownership, repeated-sheet net scope, and
  partial annotation contracts.
- **Add no duplicate companion implementation** for those native contracts;
  the companion explains how Codex must interpret and verify them.
- **Record an upstream packaging gap:** the released Konnect skill and two
  agents link `references/reliability-contract.md`, but that file is absent
  from the v0.12.0 asset tree. The companion retains its independently shipped
  evidence contract and must not claim the upstream reference was loaded.

## Retired decisions

| Policy ID | Retirement evidence | Preserved behavior |
|---|---|---|
| `native-auto-install-suppression` | Konnect v0.11.0 startup is non-mutating and guidance installation requires explicit `konnect init` (#242) | A v0.11 sync removes the companion's legacy guard and only a marker it originally created. |
| `verified-symbol-and-pin-guidance` | Konnect v0.11.0 corrected unsafe universal pin rules, known invalid library IDs, and LED polarity, with asset tests (#356) | The corrected text remains in the Codex translation; it is no longer counted as a companion-only delta. |

## Update procedure

On a new Konnect release, compare upstream assets and tool contracts first.
For every row above, mark one of these decisions in the release PR:

The required decision set is: retain, revise, retire with benchmark evidence,
or add.

- **retain** — upstream did not supply equivalent verified behavior;
- **revise** — upstream changed the contract, but the observed risk remains;
- **retire** — upstream now supplies equivalent behavior and the companion
  benchmark proves it; or
- **add** — a new benchmark, issue, or review exposed a companion requirement.

Never delete a behavior because upstream prose changed. Retirement requires
tool/asset inspection plus the relevant executable or KiCad benchmark evidence.
When a new companion-only fix is made, add its policy entry, assertions, this
table row, release-note entry, and a regression test in the same PR.
