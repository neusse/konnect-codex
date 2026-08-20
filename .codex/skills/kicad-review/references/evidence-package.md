# Repeatable review evidence package

Store review evidence outside KiCad source files in a timestamped project
subdirectory such as `review-evidence/<revision>/`. Never overwrite an earlier
accepted package.

Include when available:

- design-context profile, requirements, and design revision/commit identity;
- raw ERC and item-level DRC output;
- schematic connectivity, shorts, single-pin, and orphan results;
- footprint/pad/component inventory and transfer comparison;
- placement checkpoint and 2D/3D renders;
- trace/via inventory by net and layer, unrouted count, routing provenance, and
  DSN/SES filenames or hashes;
- zone state and outline/layer checks;
- manufacturing-validation raw output and generated artifact inventory with
  sizes/hashes;
- custom-part pin-map evidence;
- a finding ledger with stable ID, status, severity, confidence, evidence
  basis, exact location, source identity, verification path, and disposition;
- datasheet citations and explicit unverified critical-component checks;
- applicable checks not performed, failed, stale, or incomplete, with their
  affected coverage and confidence impact;
- prior-review delta classified as fixed, still-open, new, waived, or
  unverifiable;
- waivers with owner, reason, evidence, scope, and date;
- final READY, NOT READY, or INCOMPLETE verdict.

Machine-readable JSON/CSV is preferred alongside a concise Markdown index.
Record tool failures and unevaluated coverage rather than omitting them. The
package is evidence, not a substitute for resolving contradictions.

Use [review-methodology.md](review-methodology.md) for the evidence vocabulary,
datasheet trust gate, review-limit disclosure, and re-review protocol.
