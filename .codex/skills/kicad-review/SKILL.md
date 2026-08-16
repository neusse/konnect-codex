---
name: kicad-review
description: "Review and validate KiCad designs through Konnect MCP tools. Use for schematic or PCB audits, ERC, DRC, connectivity checks, issue finding, readiness decisions, and pre-fabrication review."
---

# KiCAD Design Review & Validation Workflow

This skill guides Codex through systematic design review of a KiCAD project using MCP tools.
ALL checks are performed through MCP tools — never parse .kicad_sch or .kicad_pcb files directly.

---

## Tool availability

The companion exposes the complete Konnect catalogue eagerly. Call visible
review tools directly. When running against a lazy Konnect server and the tools
are absent, load these toolsets:

```
load_toolset('sch_analysis')     # find_orphan_items, find_shorted_nets, find_single_pin_nets
load_toolset('verification')     # run_drc, check_clearance, get_design_rules
load_toolset('sch_export')       # run_erc
load_toolset('pcb_export')       # get_drc_violations
load_toolset('manufacturing')    # validate_for_manufacturing
load_toolset('design_review')    # audit_decoupling, audit_connections, audit_power_rails, etc.
```

On a lazy server, these toolsets support deeper analysis:

```
load_toolset('sch_analysis')     # get net info, trace connections, inspect components
load_toolset('pcb_routing')      # query_traces, get_nets_list
```

Use `get_active_toolsets()` only to diagnose a missing tool on a lazy server.

---

## Quick Checks (Escalating Severity)

Run these first — they are fast and catch the most critical issues.

### Level 1: Structural Integrity

```
find_orphan_items()
```

Finds floating wires, labels, and symbols not connected to anything.
These are almost always bugs (leftover from edits or incomplete wiring).

### Level 2: Critical Net Issues

```
find_shorted_nets()
```

Detects nets that are connected together but should not be. A shorted net means:
- Two different net labels on the same wire
- Power rails bridged unintentionally
- Signal nets merged by accident

**This is always a critical error. Fix before proceeding.**

### Level 3: Suspicious Connections

```
find_single_pin_nets()
```

A net with only one pin connected is almost always a mistake:
- Incomplete wiring (forgot to connect the other end)
- Orphan net labels (typo in name, so it does not match)
- Leftover stubs from deleted components

---

## Formal Checks

### ERC — Electrical Rules Check

```
run_erc()
```

Checks schematic-level rules:
- Pin type conflicts (output driving output, unconnected inputs)
- Power pin connections
- Missing no-connect flags
- Duplicate reference designators
- Missing net connections

Review each violation. Some can be waived (e.g., intentional unconnected pins marked with no-connect flag).

### DRC — Design Rules Check

```
get_drc_violations()
```

Checks PCB-level rules:
- Clearance violations (copper-to-copper, copper-to-edge)
- Minimum trace width violations
- Minimum drill size violations
- Unrouted connections (incomplete routing)
- Zone fill issues
- Courtyard overlaps

**Every DRC error must be resolved or explicitly justified before manufacturing.**

---

## Design Audits

These go beyond rule checking — they evaluate design quality and best practices.

### Decoupling Audit

```
audit_decoupling()
```

Checks:
- Every IC power pin has a bypass capacitor
- Capacitor is placed close to the pin (PCB proximity)
- Appropriate capacitor values (100nF ceramic minimum)
- Bulk capacitance present for high-current ICs

### Connection Audit

```
audit_connections()
```

Checks:
- All expected connections are made
- No nets with unexpected fan-out
- Signal integrity basics (termination on long traces)
- Pull-up/pull-down resistors where required (I2C, reset pins, enable pins)

### Power Rail Audit

```
audit_power_rails()
```

Checks:
- All power rails have proper source (regulator, connector, etc.)
- Current capacity matches expected load
- Voltage levels are consistent (no 3.3V device on 5V rail)
- Power sequencing considered for multi-rail designs
- Power flags present (avoids ERC false positives)

### Manufacturing Audit

```
audit_manufacturing()
```

Checks:
- All footprints are fab-house compatible
- Pad sizes meet minimum requirements
- Silkscreen readability
- Test point accessibility
- Fiducial marks present (for SMT assembly)
- Mechanical clearances around mounting holes

---

## Full Review Shortcut

```
run_design_review()
```

Runs ALL audits and checks in sequence, producing a consolidated report.
Use this for a comprehensive pre-manufacturing review.

Read `status`, `coverage`, and `diagnostics` before interpreting the findings.
If `status` is `partial` or `failed`, the verdict is `INCOMPLETE — review could
not evaluate the full design`. Report that verdict verbatim, explain the
diagnostics and unevaluated coverage, and do not describe the design as ready,
passing, clean, or looking good. Findings gathered before the coverage gap are
still valid and should still be reported.

Equivalent to running:
1. find_orphan_items
2. find_shorted_nets
3. find_single_pin_nets
4. run_erc
5. get_drc_violations
6. audit_decoupling
7. audit_connections
8. audit_power_rails
9. audit_manufacturing

---

## Severity Classification

### CRITICAL — Must fix before manufacturing

| Finding                            | Why Critical                                    |
|------------------------------------|-------------------------------------------------|
| Shorted nets                       | Short circuit on the board, may damage components |
| Missing ground connection          | Circuit will not function                       |
| Reversed polarity on power IC      | Immediate destruction on power-up               |
| Unrouted nets                      | Missing connections on fabricated board          |
| DRC clearance violation            | May cause electrical short on fab board          |
| Power pin unconnected              | IC will not operate                             |
| Wrong voltage on IC power pin      | Exceeds absolute maximum, destroys part         |

### WARNING — Should fix, design risk

| Finding                            | Why a Warning                                   |
|------------------------------------|-------------------------------------------------|
| Missing decoupling capacitor       | Noise susceptibility, possible oscillation      |
| No test points on key signals      | Cannot debug in production                      |
| No ESD protection on connectors    | Vulnerable to ESD damage in the field           |
| Single-point-of-failure nets       | No redundancy for critical signals              |
| Pull-up/pull-down missing          | Floating input, unpredictable behavior          |
| Tight clearances (near DRC limit)  | Higher fab defect rate                          |

### SUGGESTION — Improvement opportunities

| Finding                            | Why a Suggestion                                |
|------------------------------------|-------------------------------------------------|
| Consolidate passive values         | Fewer unique BOM lines, lower assembly cost     |
| Add net labels to unnamed nets     | Improves schematic readability                  |
| Missing silkscreen designators     | Harder to assemble and debug manually           |
| Components could be closer         | Shorter traces, better signal integrity         |
| Consider bulk capacitor addition   | Better transient response on power rails        |
| Add board revision marking         | Traceability for manufacturing runs             |

---

## Reporting Format

Present findings grouped by severity with actionable fix suggestions:

```
## Design Review Results

### CRITICAL (X issues) — Must fix

1. **[Finding title]**
   - Location: [component reference or net name]
   - Issue: [what is wrong]
   - Fix: [specific action to take using MCP tools]

### WARNING (X issues) — Should fix

1. **[Finding title]**
   - Location: [component reference or net name]
   - Issue: [what is wrong]
   - Fix: [specific action to take]

### SUGGESTION (X items) — Optional improvements

1. **[Finding title]**
   - Detail: [what could be better]
   - Action: [suggested improvement]

### Summary
- Critical: X (must resolve)
- Warnings: X (recommended)
- Suggestions: X (optional)
- Verdict: [LOOKS GOOD / NEEDS ATTENTION / NOT READY / INCOMPLETE]
- Coverage status: [complete / partial / failed]
- Coverage diagnostics: [none, or each unevaluated sheet/object/audit]
```

---

## Review Workflow

### Quick Review (5-minute check)

1. `find_shorted_nets()` — catch fatal issues
2. `run_erc()` — schematic rule check
3. `get_drc_violations()` — PCB rule check
4. Report findings

### Full Review (comprehensive)

1. Load all review toolsets
2. `run_design_review()` — full audit suite
3. Check `status`, `coverage`, and `diagnostics`; never approve an incomplete review
4. Classify all gathered findings by severity
5. Present report with fix suggestions
6. Offer to fix CRITICAL issues immediately

### Pre-Manufacturing Review

1. Full review (above)
2. `validate_for_manufacturing()` — fab-specific checks
3. Verify BOM completeness
4. Check part availability (if targeting specific fab house)
5. Final verdict: ready to manufacture or not

---

## Rules

1. **Never skip quick checks** — find_shorted_nets catches the worst bugs fast
2. **Classify every finding** — severity helps the user prioritize
3. **Provide specific fixes** — name the MCP tool and parameters to resolve each issue
4. **Run DRC after fixes** — verify that corrections did not introduce new violations
5. **Do not approve a design with CRITICAL issues** — even if the user says "it's fine"
6. **Load toolsets first** — check `get_active_toolsets()` and load what you need
7. **Save before reviewing** — ensures checks run against current state
8. **Offer to fix** — after reporting, offer to use MCP tools to resolve issues
9. **Re-run after fixes** — always verify fixes resolved the issue and created no new ones
10. **Document waivers** — if user explicitly waives a warning, note it in the report
11. **Never soften `INCOMPLETE`** — partial or failed coverage is not a passing review
