# Plan: durable reliability guidance for Codex

Part of #13. Planning only; no skills, agents, hooks, installed files or runtime behavior have changed.

## Scope

Adapt the final Konnect reliability contract into this repository's independent Codex guidance. The upstream policy work is https://github.com/mixelpixx/Konnect/pull/550; this repository owns every companion deliverable and no upstream issue depends on it.

Review existing skills, agents, routing, hook support and living change policy before editing. Preserve previous PCB/schematic/library, lifecycle and workflow enhancements. Record the upstream version/commit and keep a durable local reference with source, rationale, tests and retirement criteria for each adaptation.

Cover complete/partial/failed/uncertain outcomes, target and source identity, missing evidence, unsupported capabilities, and recovery without repeating applied mutations. Do not claim guidance implements missing server safeguards. Keep Codex instructions independent of Claude model and hook declarations.

## Acceptance and evidence

Issue #13 owns the full checklist. Test packaged guidance and routing/agent handoffs on partial batches, stale saved state, invalid arguments and unavailable checks. Verify installed artifacts at implementation time and document how release refresh reapplies local enhancements. No tests have been run for behavior that has not been implemented.

## Sequence

Design may proceed independently; final contract wording depends on upstream #550. Reconcile the existing local unpublished main commit before implementation without publishing it accidentally. This draft starts from published origin/main. Remain draft until implementation and evidence are complete; then add the terminal closing reference for #13. No release or installation is part of this planning step.

