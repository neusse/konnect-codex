# Codex guidance standards

This is the acceptance standard for every reviewed skill, agent, hook, and
release in `konnect-codex`. It complements the machine-readable enhancement
policy; it does not replace the exact upstream baseline or release tests.

## Skills

- Use Codex-native `SKILL.md` frontmatter and progressive disclosure. Every
  bundled reference must be linked from its parent skill with a concrete read
  trigger.
- Treat shortcut tables as non-exhaustive caches. Verify identifiers against
  the active KiCad libraries and never use personal favorites as a shared
  allowlist.
- Exact requirements and manufacturer/package data outrank generic rules.
  Package-sensitive and custom parts require a physical lead -> symbol pin ->
  footprint pad map with explicit drawing view and direction.
- Name actual Konnect tools and their released contracts. Do not promise a
  validation, export, router, fallback, or live-editor behavior the pinned
  Konnect release does not implement.
- Static fabricator limits, stock, fees, prices, field names, and capabilities
  are time-sensitive. Require a current official source, retrieval date, and
  selected process instead of treating old prose as authoritative.
- Completion requires direct evidence, contradiction reconciliation, and
  explicit `INCOMPLETE` handling. Aggregate or heuristic output cannot outrank
  requirements, datasheets, ERC/DRC, connectivity, inventory, or artifacts.
- Schematic completion includes functional blocks, group/region closure,
  label-inclusive overlap checks, page-boundary checks, and rendered visual
  inspection. PCB completion includes transfer invariants, visible placement,
  route provenance, direct DRC, unrouted, and artifact checks.

## Agents

- Give one agent ownership of a KiCad mutation phase. Library -> schematic ->
  BOM -> PCB -> independent review -> bring-up handoffs are sequential.
- Each agent must read or be governed by the matching domain skill and named
  references. Its completion gate must not be weaker than the skill it executes.
- Never claim that a query, render, inspection, ERC, DRC, export, route, or
  datasheet check ran unless its result is present in the handoff.
- Every reported result maps to an actual prescribed tool/evidence step. A
  missing, failed, skipped, structurally impossible, or contradictory required
  result is `BLOCKED` or `INCOMPLETE`, never an inferred pass.
- A read-only agent must say which actions and measurements remain plans rather
  than performed work.

## Hooks

- Emit the structured Codex hook response expected by the event. Successful
  `PreToolUse` guidance uses `hookSpecificOutput.hookEventName` and
  `additionalContext`; stdout contains only that JSON object.
- Keep hook commands safely quoted after generated installation, including
  Windows paths with spaces.
- Classify tools by released runtime contract: live-only, live-or-closed
  fallback, closed-board-only, or dry-run/apply. Do not send one inaccurate IPC
  warning to operations with different ownership rules.
- The checked-in hook contract and hook matchers must agree exactly. Server-side
  board identity, liveness, revision, and conflict checks remain authoritative.
- Uninstall removes only companion-owned files and entries and preserves mixed
  user configuration.

## Release acceptance

For every Konnect release, apply `docs/CHANGE_POLICY.md`, review every row in
`docs/GUIDANCE_DELTA_REGISTER.md`, and update the machine policy. A release is
blocked by an unaccounted upstream asset, an unreachable reference, an invalid
known library shortcut, an unmatched hook target, a policy assertion failure,
format/test/Clippy failure, failed local `sync`/`doctor`, or missing release
artifact evidence.
