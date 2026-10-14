# roundtrip-gaps-1: renderer gaps in filters, conditions, replacements and triggers

Part of the `roundtrip-followups.md` stream. Baseline (Oct 14): 25,278 of 28,702 fully
supported cards pass the round trip; 3,394 mismatch.

CLUSTERS TAKEN (from `roundtrip --clusters 40`):
- `gap: Filter` (64) — Alrund, God of the Cosmos; Keral Keep Disciples; Rowan's Talent
- `gap: Condition` (22) — Spara's Adjudicators; Zariel, Archduke of Avernus; Seraphic Greatsword
- `gap: replacement Discriminant` (16) — Fasting; Forethought Amulet; Loxodon Smiter
- `gap: TriggerCond` (9) — Reidane, God of the Worthy; Leyline of Combustion; Rayne, Academy Chancellor
- `gap: target slot 0 never mentioned` (8), `gap: target slot 0 out of range` (6) — Champion of the Weird; Disarm; Plasm Capture
- `gap: the triggering object outside a triggered ability` (5) — Garruk, Curse Breaker; Reflect Damage

The parallel item roundtrip-gaps-2 takes the Effect/Value/Modification/Sel/StaticEffect
and remembered-group gaps; don't take those.

For each card: teach the renderer (`crates/mtg-engine/src/oracle/render/`) to word the AST
node from its meaning, independently of the parser. Once it renders, triage any remaining
mismatch: parser bug (fix + in-game test in `tests/cards/roundtrip_gaps_1.rs` + row in the
docs/ROUNDTRIP.md bug table), renderer wording, or a CR-justified equivalence. Gaps that
expose an AST that can't express the text's meaning are usually parser bugs. Regenerate
docs/ROUNDTRIP.md and docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Remaining

Still mismatching in the taken clusters (128 of the 130 cards pass):
- **Nahiri, the Lithomancer** — the −10 creates a token "named Stoneforged Blade. It has
  indestructible, 'Equipped creature gets +5/+5 and has double strike,' and equip {0}."
  The renderer words a token's abilities the general way ("with indestructible, ...");
  the card's "named X. It has ..." sentence form isn't a wording it has yet. Renderer
  wording only; the token's abilities are compiled correctly.
- **Heliod's Punishment** — "Enchanted creature can't attack or block. It loses all
  abilities and has '...'" is compiled as two static abilities in the other order. No
  play effect: "can't attack or block" is a restriction of the Aura's static ability
  (not an ability of the creature), so losing all abilities doesn't remove it, and both
  abilities of the Aura have the same timestamp (CR 613.7a); the comparison is
  line-based, so the order shows up as a mismatch.
