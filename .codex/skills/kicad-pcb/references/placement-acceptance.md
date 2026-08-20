# Placement acceptance

Routing must not begin until the mechanical placement is visibly reviewable and
passes direct checks.

1. Save the board and record reference, X/Y, side, rotation, pad count, hole
   count, trace count, and unrouted count.
2. Generate or capture a 2D board render with courtyards, board outline, pads,
   holes, and reference designators visible. A 3D view is useful but does not
   replace copper/courtyard inspection.
3. Run direct overlap, clearance, and DRC checks. Inspect pad-to-pad, hole-to-pad,
   courtyard, board-edge, connector-access, mounting, test-point, and tall-part
   conflicts.
4. Check repeated components and dense rows for shared midpoints or coincident
   pads rather than assuming a grid placement is safe.
5. Verify power, noisy, sensitive, RF, and high-current blocks against the
   relevant layout constraints.
6. Present the render and exception list as the placement checkpoint. Proceed
   to whole-board routing only after the checkpoint is explicitly accepted or
   all blocking conflicts are resolved.

The checkpoint is invalid if IPC ownership changes, component positions change,
or the board is reopened from a different path. Recreate it after such events.
