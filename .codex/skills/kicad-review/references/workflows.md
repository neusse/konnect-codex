# Review and Recovery Workflows

## Existing-project intake

1. Identify the exact project, schematic, and board paths and current editor
   ownership.
2. Inventory files, sheets, symbols, footprints, nets, board outline, routes,
   zones, and manufacturing artifacts without mutation.
3. Run applicable ERC/DRC/connectivity checks and render the current design.
4. Record blockers, stale artifacts, and the safe baseline for later changes.

## Simulation readiness

1. Identify the requested simulation and required observables.
2. Inventory simulation-capable models, sources, parameters, and unsupported
   devices or abstractions.
3. Check that the intended circuit connectivity and model pin mappings are
   suitable for the simulator.
4. Report the missing models, substitutions, or setup needed.

Konnect v0.13.0 does not itself prove that a circuit simulation ran. Report only
readiness evidence; never convert model presence into a simulated result.

## Design review

Follow [review-methodology.md](review-methodology.md), use
[design-checklist.md](design-checklist.md), classify findings with
[error-taxonomy.md](error-taxonomy.md), and assemble direct outputs with
[evidence-package.md](evidence-package.md).

## Project recovery

1. Inventory editor processes, locks, autosaves, backups, and timestamps before
   changing state.
2. Identify the newest internally consistent saved source; preserve every
   candidate until the user selects or evidence establishes the recovery path.
3. Stop stale sessions only when ownership is proven and doing so is within the
   user's request.
4. Reopen the recovered project, query its inventory, and run applicable
   validation before resuming design work.
5. Apply the shared
   [process-lifecycle gate](../../kicad-workflows/references/process-lifecycle.md):
   preserve the recorded pre-existing sessions, close task-owned recovery
   windows normally, and verify no task-owned orphan remains.

Never overwrite uncertain source files or treat a successful open as proof that
all design content survived.
