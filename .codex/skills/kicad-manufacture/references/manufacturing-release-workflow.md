# Manufacturing Release Workflow

1. Freeze and identify the exact design revision and source-file hashes.
2. Save/reopen the project and run the strongest applicable ERC, connectivity,
   footprint, unrouted, short, and direct DRC checks.
3. Complete the PCB layout-physics applicability matrix against the frozen
   board, including direct evidence or an explicit engineering waiver for every
   applicable row. Clean DRC alone does not close this gate.
4. Inspect schematic and PCB renders, board outline, layers, silkscreen,
   courtyards, orientation, and expected 3D-model coverage.
5. Qualify the BOM and assembly data, including DNP and alternates where
   required.
6. Export Gerbers, drills, position files, BOM, drawings, and other requested
   artifacts from the frozen revision.
7. Inspect the produced archive and record filenames, sizes, layers, hashes,
   tool/version information, and validation outputs in a release manifest.

Outcome is `READY` only when every required artifact and direct acceptance check
is present. A failed, skipped, unavailable, stale, or contradictory item makes
the release `NOT READY` or `INCOMPLETE` with the next safe action stated.
