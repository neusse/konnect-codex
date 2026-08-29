---
name: kicad-bom
description: "Qualify and maintain KiCad bills of materials through Konnect. Use for MPNs, manufacturers, alternates, DNP state, datasheets, lifecycle or stock risk, component sourcing, BOM health, cost preparation, and assembly BOM export."
---

# KiCad BOM qualification

Use this skill to make the schematic component properties the maintained BOM
source of truth, qualify each important line item, and hand an inspected export
to manufacturing. Use `kicad-manufacture` after BOM qualification for Gerbers,
position files, and fabrication packages.

## Capability boundary

Use only visible Konnect tools for KiCad-source changes. Read-only
qualification can also use available manufacturer documents and current
distributor sources, with citations and observation dates. Useful Konnect tools
include:

- `load_user_config` for manufacturer, distributor, and fabrication defaults;
- `list_schematic_components` and `check_bom_health` for inventory and gaps;
- `edit_schematic_component` and `batch_edit_schematic_components` for fields;
- `enrich_datasheets` and `get_datasheet_url` for available datasheet links;
- `search_jlcpcb_parts`, `get_jlcpcb_part`, and
  `suggest_jlcpcb_alternatives` for supported catalogue evidence; and
- `export_bom` for the controlled output.

If a required distributor, lifecycle, pricing, or compliance source is not
available through the current tool set, report that limitation. Do not invent
current stock, price, lifecycle status, qualification, or alternates.

## Workflow

1. Identify the target schematic, design revision, build quantity, prototype or
   production intent, target assembler, acceptable substitutions, DNP policy,
   and any lifecycle or qualification requirements.
2. Load user configuration when available. Inventory the schematic and run
   `check_bom_health`. Preserve its direct output as baseline evidence.
3. Establish the required property set. Use exact project field names and keep
   at least `Manufacturer`, `MPN`, `Datasheet`, and assembly-specific part
   number fields when applicable. Record alternates explicitly rather than
   encoding them in free-form values. Preserve and inspect KiCad's native DNP
   state through the generated BOM.
4. Qualify each non-generic line item against its exact function and package.
   Confirm manufacturer, MPN and suffix, footprint or package, ratings,
   availability evidence, and datasheet identity. Prioritize critical and
   long-lead parts before passives.
5. Apply the datasheet trust gate below. Treat catalogue listings as discovery
   evidence, not as proof of electrical suitability or active manufacturer
   production.
6. Select alternates only after checking electrical function, pin map, package,
   footprint, temperature and voltage/current ratings, assembly process, and
   any firmware-visible difference. Record an explicit no-alternate risk when
   a critical single-source part has no qualified substitute.
7. Review proposed field updates as a component-by-component change set. Apply
   cohesive updates with Konnect, then re-list the affected components and
   re-run `check_bom_health`.
8. Export the BOM with explicit fields and labels. Verify the file exists, is
   non-empty, contains the expected references and quantities, excludes DNP
   parts according to the stated policy, and preserves MPN and assembler part
   numbers.
9. Report unresolved gaps and hand the qualified BOM to `kicad-manufacture`.

Konnect 0.11.0 does not expose a dedicated mutation for KiCad's native DNP
attribute. Do not create a custom field named `DNP` and assume it controls
KiCad export behavior. When native DNP state must change and no visible tool
supports it, preserve the source file and report the smallest manual KiCad
step; then re-export and verify the result.

## Schematic source of truth

Maintain durable part identity in schematic symbol properties. Generated CSV,
price, and stock files are views of that data, not competing authorities. Do
not overwrite an existing MPN, datasheet, DNP decision, or alternate merely
because a catalogue search returns a different candidate. Report the conflict
and resolve it against requirements and component evidence first.

For shared multi-board assemblies, keep per-board schematic properties intact
and create a separate system-level aggregation artifact. Do not force
mechanical, cable, enclosure, consumable, or programming items into a KiCad
schematic solely to make the assembly list complete.

## Datasheet trust gate

For every critical IC, semiconductor, connector, protection part, custom part,
and package-sensitive component:

- match the exact MPN and suffix to the document;
- verify package and physical pin assignment;
- verify the ratings and required externals used by the design;
- cite the manufacturer document in the qualification evidence; and
- label the line `unverified` when exact evidence is unavailable.

Datasheet presence alone is not qualification. A URL for a family datasheet or
a different package does not close the gate.

## Time-sensitive evidence

Stock, price, lead time, catalogue category, and lifecycle claims require a
source and observation date. Present them as a snapshot, not a durable fact.
Never infer active lifecycle from distributor stock. For production-critical
parts, distinguish manufacturer lifecycle evidence from reseller availability.

## Completion gate

A qualified BOM requires:

- every fitted line has a stable value, footprint, manufacturer, and exact MPN
  unless the project explicitly permits a generic part;
- every critical line has a verified datasheet or a visible unresolved risk;
- alternates are footprint and function compatible, not keyword matches;
- DNP and non-placeable items follow the stated assembly policy, or an exact
  native-DNP capability limitation is reported;
- current stock or price claims carry source dates;
- `check_bom_health` was reconciled after edits; and
- the exported BOM was inspected directly.

Report one outcome: `QUALIFIED`, `NOT QUALIFIED`, or `INCOMPLETE`. Include
component-specific gaps, evidence sources and dates, assumptions, conflicts,
and the exact export path.

## Methodology provenance

The source-of-truth and lifecycle workflow concepts were adapted in original
wording after review of the MIT-licensed
[`kicad-happy` BOM guidance](https://github.com/neusse/kicad-happy/blob/main/skills/bom/SKILL.md).
All KiCad mutations and supported catalogue operations remain bounded by
Konnect's tool contracts.
