# roundtrip-gaps-2: renderer gaps in effects, values, modifications and remembered groups

Part of the `roundtrip-followups.md` stream. Baseline (Oct 14): 25,278 of 28,702 fully
supported cards pass the round trip; 3,394 mismatch.

CLUSTERS TAKEN (from `roundtrip --clusters 40`):
- `gap: Effect` (38) — Karn, Scion of Urza; Plunge into Darkness; Karn, Living Legacy
- `gap: Value` (34) — Gorex, the Tombshell; Windfall; Extus, Oriq Overlord
- `gap: Modification` (17) — Quickening Licid; Tempting Licid; Transmogrifying Licid
- `gap: a remembered group no instruction names` (8) — Green Slime; Twist Allegiance
- `gap: remembering Union` (7) — Cauldron's Gift; Distended Mindbender
- `gap: terms of a static permission to play cards` (6) — Lobelia, Defender of Bag End; Chainer, Nightmare Adept
- `gap: Sel` (4), `gap: StaticEffect` (4), `gap: qualities of an alternative list` (4),
  `gap: several remembered numbers` (4) — Korvold, Gleeful Glutton; Lilah, Undefeated Slickshot; Combo Attack; Primal Adversary

The parallel item roundtrip-gaps-1 takes the Filter/Condition/replacement/TriggerCond and
target-slot gaps; don't take those.

For each card: teach the renderer (`crates/mtg-engine/src/oracle/render/`) to word the AST
node from its meaning, independently of the parser. Once it renders, triage any remaining
mismatch: parser bug (fix + in-game test in `tests/cards/roundtrip_gaps_2.rs` + row in the
docs/ROUNDTRIP.md bug table), renderer wording, or a CR-justified equivalence. Gaps that
expose an AST that can't express the text's meaning are usually parser bugs. Regenerate
docs/ROUNDTRIP.md and docs/roundtrip-passing.txt; `roundtrip --check` must pass.
