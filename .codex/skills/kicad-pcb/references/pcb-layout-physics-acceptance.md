# PCB layout physics acceptance

DRC proves geometric rule compliance. It does not prove that return current,
thermal flow, current capacity, RF clearance, or production access is adequate.
Apply this gate before accepting a new route, a substantial PCB change, or a
manufacturing release.

## Establish applicability

Record each item below as `applicable`, `not applicable` with a reason, or
`unverified`. For applicable items, identify the affected references, nets,
layers, interfaces, operating limits, and the evidence used. A generic rule of
thumb is not evidence when a stackup, component layout example, interface
specification, current calculation, or assembly process is available.

## Acceptance checks

1. **Reference-plane continuity** - Inspect every fast-edge, clock, RF, and
   controlled-impedance route against its adjacent reference plane. The route
   must not cross a plane split, void, cutout, antipad field, or missing zone
   that forces a large return-current detour. Put a nearby return via at a
   signal layer transition when the reference changes, or document the
   engineered return structure.
2. **Decoupling current loops** - Map each required bypass capacitor to the
   supply pin or pin group it serves. Verify a short, low-inductance supply and
   return loop, appropriate layer placement, and a ground transition at the
   capacitor where needed. Distance alone is not acceptance evidence.
3. **Switching-regulator hot loops** - Identify the high-di/dt input loop,
   switch node, rectifier or synchronous devices, inductor, and output loop.
   Compare placement and copper with the exact regulator datasheet or
   evaluation-board layout. Keep the hot loop compact and bound the switch-node
   copper; record any intentional departure.
4. **Thermal paths** - For exposed-pad and high-dissipation parts, follow the
   exact land-pattern, copper-area, and thermal-via recommendations unless a
   thermal calculation supports another construction. Check dissipation,
   ambient and enclosure assumptions, copper spreading, airflow, and heat into
   sensitive neighbors.
5. **Current capacity** - Size traces, pours, pads, connectors, and every
   layer-transition via set from continuous and peak current, copper weight,
   permitted temperature rise, and allowed voltage drop. Use parallel vias or
   a larger structure where one via is the bottleneck. Record the calculation
   and its margin.
6. **Differential interfaces** - Derive width, gap, and reference geometry from
   the selected stackup and interface impedance. Inspect pair symmetry,
   continuous reference, coupling changes, layer transitions, avoidable stubs,
   and the interface-specific skew budget. Equal length by itself is not a
   passing result.
7. **Antenna keepouts** - Apply the exact module or antenna keepout on every
   required copper and component layer. Include mounting hardware, shields,
   cables, batteries, and enclosure metal in the clearance review. Preserve
   the conditions required by any module certification.
8. **Board-edge emissions** - Keep fast or sensitive routes away from board
   edges using a clearance justified by stackup and compliance risk. Treat
   edge-distance multiples as heuristics, not universal limits, and document
   the tradeoff when a two-layer board cannot use an internal route.
9. **Ground stitching and return transitions** - Place stitching vias where
   they shorten a real return path: near signal vias, reference transitions,
   connectors, shielding boundaries, noisy regions, and board edges when the
   EMC strategy calls for them. Choose spacing from the relevant frequency and
   geometry. Do not place indiscriminate via fields through antenna keepouts,
   isolation barriers, assembly clearances, or split-domain boundaries.
10. **Via-in-pad process** - Identify every via that intersects a solderable
    component pad. Verify that the selected fabricator and assembler support
    the required filled, capped, and plated process, and include it in the fab
    notes and quote. Thermal-pad vias follow the component land pattern and
    stencil/voiding requirements; they are not an automatic exception.
11. **Design-for-test access** - Build a test-access matrix from programming,
    bring-up, debug, and production-test requirements. Cover required rails,
    reset/programming signals, buses, and fault-isolation nodes with accessible
    pads or connectors. Check probe size, spacing, side, fixture access, and
    signal-integrity loading; test points are not automatically required on
    every communication conductor.

## Evidence package

Preserve:

- the completed applicability matrix and all waivers;
- the stackup and impedance/current/thermal assumptions used;
- datasheet or evaluation-board layout comparisons for critical parts;
- annotated copper or layer renders showing planes, voids, keepouts, vias,
  current paths, and test access as applicable; and
- direct DRC, unrouted, zone-fill, net/layer inventory, and manufacturing
  process results.

An applicable item without enough evidence is `INCOMPLETE`. A conflict with an
exact datasheet, stackup, interface requirement, fabricator process, or product
requirement is blocking until corrected or explicitly waived by the responsible
engineer. A clean DRC or successful autorouter result cannot override this
gate.
