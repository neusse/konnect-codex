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

- Supported Konnect: `0.11.0`, commit
  `a22ad2153dcf45dbcf1cc63b5b0f1e40c93d7956`
- Companion release: `v0.11.0` (`companion_revision = 1`)
- Last full guidance review: 2026-08-29
- Upstream guidance issues reviewed: #356, #357, #358
- Upstream issue #358 correction: Konnect v0.11.0 still registers
  `refill_zones` under `pcb_export`; the companion therefore keeps that real
  tool in the live-only hook class. The issue's broader structured-output and
  runtime-classification findings still apply.

## Active decisions

| Policy ID | Companion behavior retained or added | Verification / retirement review |
|---|---|---|
| `agent-delegation` | Deterministic Codex specialist handoffs | Router assertion; retire for equivalent native Codex routing |
| `schematic-evidence-and-collision-gate` | Reconcile orphan false positives and block stub-created shorts | Skill/agent assertions and schematic benchmark |
| `schematic-layout-readability-gate` | Functional blocks, group closure, label-inclusive visual gate | Skill/reference/agent assertions and rendered benchmark |
| `pcb-transfer-integrity` | Preserve pad/graphic/layer/model invariants across transfer | PCB skill assertion and transfer benchmark |
| `contradictory-verifier-gate` | Direct evidence outranks aggregate passes | Review/manufacture/agent assertions |
| `requirements-based-review-defaults` | Datasheets and requirements control conditional design advice | Reviewer assertion |
| `doctor-agent-reporting` | Report companion and native agents separately | Doctor tests |
| `pcb-builder-delegation` | Give transfer/layout/routing to one PCB owner | Router/agent assertions |
| `freerouting-first-routing` | Use Freerouting for complete boards; local segments only for repairs | PCB skill/reference/agent assertions and route benchmark |
| `pcb-live-state-and-placement-gates` | Stop on IPC ownership loss and require visible placement acceptance | PCB/reviewer assertions and preflight tests |
| `custom-part-physical-pin-acceptance` | Require view-aware datasheet lead-to-pad proof | Library reference/agent assertions |
| `visual-placement-checkpoint` | Require a reviewed 2D placement artifact before routing | PCB reference/agent assertions |
| `offline-freerouting-bridge` | Retain non-overwriting offline DSN/SES bridge until upstream equivalent passes | CLI/route tests and PCB assertions |
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
