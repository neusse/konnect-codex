# Konnect guidance change policy

`konnect-codex` is a versioned compatibility layer, not an unrelated rewrite of
Konnect guidance. Every release maintains three distinguishable layers:

1. **Upstream baseline** — `policy/upstream-baseline.json` records the exact
   normalized hash of every skill, reference, and agent reviewed from the pinned
   Konnect release.
2. **Codex enhancements** — `policy/enhancements.json` records every intentional
   behavioral difference, its evidence, its affected assets, and the condition
   under which it should be removed.
3. **Reviewed runtime assets** — `.codex/skills` and `.codex/agents` are the
   Codex-native files embedded and installed by the companion.
4. **Living decision record** — `docs/GUIDANCE_DELTA_REGISTER.md` is the
   human review surface for every active/retired enhancement, while
   `docs/GUIDANCE_STANDARDS.md` defines the minimum quality contract for new
   and refreshed guidance.

The reviewed runtime assets are maintained deliberately; new upstream prose is
never merged into them blindly. The machine-readable baseline makes upstream
drift specific, while enhancement assertions prevent a release refresh from
silently dropping a Codex safety rule.

## Release refresh

For every Konnect release:

1. Pin the new Konnect version and commit.
2. Generate a new per-file upstream baseline and compare it with the previous
   baseline. Account for every added, removed, or modified upstream asset.
3. Review every active enhancement. Port it, revise it to match new tool
   behavior, or retire it only when the recorded retirement condition is met.
4. Update every row in `docs/GUIDANCE_DELTA_REGISTER.md`: retain, revise,
   retire with benchmark evidence, or add. Update `policy/enhancements.json`,
   release notes, assertions, and tests in the same change.
5. Audit skills, agents, and hooks against `docs/GUIDANCE_STANDARDS.md`. Update
   the reviewed Codex skills and agents. Keep procedures concise and put
   release-specific evidence or tables in references rather than duplicating it.
6. Run `konnect-codex audit --source <Konnect checkout>`. A per-file mismatch,
   aggregate guidance mismatch, hook mismatch, or version mismatch blocks the
   release.
7. Run formatting, tests, Clippy, lifecycle tests, and the end-to-end KiCad
   benchmark. The benchmark must exercise deterministic agent delegation,
   schematic collision checks, PCB transfer invariants, contradictory-verdict
   handling, and manufacturing artifact verification.
8. Publish with the matching Konnect version and an incremented
   `companion_revision` when the supported Konnect release is unchanged.

## Evidence and retirement

An enhancement is an explicit compatibility decision, not permanent sediment.
Each entry must identify why it exists and when it can be removed. When upstream
implements equivalent behavior, verify it in the benchmark, retire the
enhancement, and record the retirement in release notes. If upstream changes the
tool contract without solving the observed behavior, adapt the enhancement and
keep it active.

## Conflict policy

Konnect owns MCP tool semantics and KiCad safety constraints. `konnect-codex`
owns Codex invocation, delegation, evidence reconciliation, and fail-closed
completion gates. Where generic upstream advice conflicts with component data,
project requirements, or stronger independent verification, the reviewed Codex
assets preserve the user requirements and report the conflict explicitly.
