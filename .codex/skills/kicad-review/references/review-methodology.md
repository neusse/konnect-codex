# Evidence-grounded review methodology

Use this reference for comprehensive design reviews and readiness decisions.
It defines how to calibrate the review, describe evidence, expose uncertainty,
and compare repeated reviews. It does not add analysis capabilities beyond the
available Konnect tools and exported artifacts.

## Design context

Establish the context that can materially change a finding before assigning
severity:

- prototype, validation build, or production release;
- operating voltages, currents, loads, environment, and expected lifetime;
- target fabricator, assembler, process limits, and inspection expectations;
- safety, regulatory, isolation, reliability, or traceability requirements;
- enclosure, connector, thermal, test, service, and bring-up constraints; and
- explicit project waivers and accepted risks.

Use requirements and user-confirmed context as authoritative. Design content
such as complete MPN coverage, safety markings, or wide-temperature parts can
suggest a context but cannot establish it by itself. Ask for confirmation when
an inferred context would materially raise or suppress a finding. Do not apply
fixed hobby, medical, automotive, aerospace, IPC-class, or test-point
thresholds without a stated project or manufacturing requirement.

Record the resulting profile in the report. Mark unknown fields as unknown
rather than silently selecting a demanding or permissive default.

## Evidence basis

Give every substantive finding one primary evidence basis:

| Basis | Meaning |
|---|---|
| `datasheet-verified` | Checked against the exact manufacturer document and package or part suffix; cite page, table, figure, or section. |
| `konnect-direct` | Established by a direct Konnect check or query such as item-level ERC/DRC, connectivity, inventory, geometry, or artifact inspection. |
| `export-verified` | Established from a generated netlist, BOM, report, render, Gerber, drill, or assembly artifact whose revision is identified. |
| `aggregate-derived` | Reported only by a summary audit or readiness tool and not independently confirmed. |
| `engineering-inference` | Reasoned from available facts, requirements, and engineering practice without direct confirmation. |
| `unverified` | Relevant evidence was unavailable, incomplete, stale, or contradictory. |

Do not describe `aggregate-derived`, `engineering-inference`, or `unverified`
claims as verified. A direct result can still be incomplete or stale; retain the
plugin's contradiction and live-state gates.

## Confidence

Assign confidence independently from severity:

- `high`: direct evidence is current, specific, and mutually consistent;
- `medium`: evidence is relevant but incomplete, indirect, or dependent on a
  documented assumption; or
- `low`: evidence is missing, ambiguous, stale, contradictory, or based mainly
  on a generic convention.

Severity answers how harmful the issue could be. Confidence answers how well
the claim is established. A potentially destructive but poorly evidenced issue
can be `CRITICAL / low confidence`; report it as a blocking verification need,
not as a proven defect.

## Datasheet trust gate

For production-critical components, verify the exact MPN and package suffix
against manufacturer data when that data is available. Prioritize:

- regulators, power switches, drivers, and protection devices;
- processors, programmable devices, memories, clocks, and transceivers;
- transistors or diodes with package-dependent pin assignments;
- connectors, isolation devices, and safety-related components; and
- custom symbols, footprints, and mechanically constrained parts.

Check the facts that affect the actual use: physical pin map, supply and
absolute-maximum limits, required externals, package, polarity, thermal pad,
ratings, and layout constraints. Internal agreement between schematic, PCB,
and library data proves consistency, not physical correctness. When the exact
datasheet is unavailable, label the affected checks `unverified`, state the
assumption, and explain the consequence. Do not block unrelated checks.

## Finding record

Each critical or warning finding must contain:

- stable finding ID and current status;
- severity and confidence;
- primary evidence basis;
- exact component, pin, pad, net, layer, coordinate, rule, or artifact;
- requirement or expected behavior;
- observed result and source identity;
- calculation or verification path when applicable;
- smallest safe correction or next verification step; and
- waiver or disposition, when one exists.

Suggestions can use a shorter form but must still distinguish a requirement
from a preference.

## False-positive and contradiction triage

Do not repeat tool output unfiltered. Correlate a candidate finding with the
design context and the strongest independent evidence available. Record a
dismissed candidate when it is material to trust in the review, including why
it was dismissed and what evidence resolved it.

When checks disagree, preserve both raw results, identify which result is more
direct and current, and keep the verdict `NOT READY` or `INCOMPLETE` until the
disagreement is explained. A passing aggregate result never outranks a direct
failure or missing requested artifact.

## Review limits

Every comprehensive report must include `Not performed / review limits`. Name
each applicable check that was not run, failed, or had incomplete coverage and
state:

- why it was unavailable;
- which components, sheets, nets, layers, or artifacts were affected; and
- which conclusions therefore remain unsupported.

Typical entries include unavailable datasheets, incomplete ERC/DRC coverage,
missing PCB or fabrication files, absent thermal or EMC evidence, stale live
state, unavailable lifecycle data, and uninspected mechanical constraints.
Silence is not evidence that a check passed.

## Re-review delta

When a prior evidence package exists, preserve it and compare findings using
stable IDs plus design locations. Classify each prior and current finding as:

- `fixed`;
- `still-open`;
- `new`;
- `waived`; or
- `unverifiable`.

Also record design revision identity, changed requirements, changed tool or
server versions, and any changed review coverage. Do not call a finding fixed
only because a later tool omitted it; require evidence that the underlying
condition changed or that the original finding was invalid.

## Methodology provenance

The evidence-basis, review-limit, design-intent, and re-review concepts were
adapted in original wording after review of the MIT-licensed
[`kicad-happy` review guidance](https://github.com/neusse/kicad-happy/tree/main/skills/kicad/references).
Konnect remains the authority for tool behavior and KiCad source safety in this
plugin.
