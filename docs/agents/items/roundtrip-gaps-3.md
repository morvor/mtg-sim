# roundtrip-gaps-3: the new gap clusters

Part of the `roundtrip-followups.md` stream. Baseline (Oct 14, origin ae5c399, after
roundtrip-tail-5 landed): 26,145 of 29,374 fully supported cards pass; 3,199 mismatch.
New cards became fully supported since the earlier gap items, and roundtrip-tail-5 made
one-use replacement effects a gap when nothing words their limit; so new gap clusters appeared.
This item runs alone (one item at a time).

CLUSTERS TAKEN (every `gap:` cluster from `roundtrip --clusters 400` on ae5c399):
   51  gap: EventsThisTurn  [Vizier of Deferment; Macabre Reconstruction; Mad Dog]
   29  gap: Value  [Nalathni Dragon; Void Mirror; Initiates of the Ebon Hand]
   15  gap: Filter  [Kotis, Sibsig Champion; Rapid Augmenter; Grist, Voracious Larva // Grist, the Plague Swarm]
   14  gap: Effect  [Tempt with Immortality; The Death of Gwen Stacy; Osseous Sticktwister]
   10  gap: one-use replacement effect  [Circle of Solace; Impulsive Maneuvers; Dazzling Reflection]
    9  gap: EventsDuring  [Tangle Kelp; Command the Stage; Shape of the Wiitigo]
    8  gap: Condition  [Karlach, Fury of Avernus; Finest Hour; Tifa, Martial Artist]
    7  gap: Sel  [Avengers Assemble!; Neyali, Suns' Vanguard; Robber of the Rich]
    2  gap: TriggerCond  [Critical Hit; Resolute Veggiesaur]
    2  gap: a remembered group no instruction names  [Spelunking; Breaking Wave]
    1  gap: Modification  [Blood Sun]
    1  gap: Restriction  [Invasion Plans]
    1  gap: keyword action undo  [Serene Sleuth]
    1  gap: several remembered numbers  [Rottenmouth Viper]
    1  gap: target slot 0 out of range  [Behemoth of Vault 0]

For each card: teach the renderer (`crates/mtg-engine/src/oracle/render/`) to word the AST
node from its meaning, independently of the parser. Once it renders, triage any remaining
mismatch: parser bug first (does the AST mean what the text says, in multiplayer, with
control changes, tokens, copies, replacement effects, zero-quantity choices?) with a fix +
in-game test in `tests/cards/roundtrip_gaps_3.rs` that fails on the old code + a row in the
bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else renderer wording, or a
CR-justified equivalence. A gap where the AST can't express the text's meaning is usually a
parser bug. For the one-use replacement gap: a card whose text has no "the next time" (or
equivalent) limit but whose AST has one is a parser bug. Add "Parser bugs found" and
"Remaining" sections (each still-mismatching card with its reason). Regenerate
docs/ROUNDTRIP.md and docs/roundtrip-passing.txt; `roundtrip --check` must pass.
