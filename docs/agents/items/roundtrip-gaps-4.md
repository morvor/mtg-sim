# roundtrip-gaps-4: new gap clusters and the 4+-card wording clusters

Part of the `roundtrip-followups.md` stream. Baseline (Oct 21, origin a29dfa98, after
verified-cards landed): 26,438 of 30,041 fully supported cards pass; 3,572 mismatch (new
cards keep becoming fully supported, bringing new gaps). Runs alone (one item at a time).
Round-trip failures keep cards out of the `verified` tier (docs/verified-cards.tsv), so each
fix here also raises the verified count.

CLUSTERS TAKEN (from `roundtrip --clusters 400` on a29dfa98): every `gap:` cluster, and
every wording cluster of 4 or more cards:
   24  gap: Effect  [Search for Survivors; Psychotic Episode; Underdark Rift]
   23  gap: Filter  [Pompous Gadabout; Weftwalking; Illusionary Terrain]
   18  gap: StaticEffect  [Enthralling Hold; Enduring Renewal; Diligent Farmhand]
    9  gap: Value  [Hot Pursuit; Wheel of Misfortune; Goblin Game]
    8  gap: a comparison of values of the object  [Yenna, Redtooth Regent; Sanar, Innovative First-Year; Abundant Harvest]
    3  gap: Condition  [Rocket Launcher; Shirei, Shizo's Caretaker; Expert-Level Safe]
    3  gap: PlayerModification  [Shaman's Trance; Lumbering Laundry; Detection Tower]
    3  gap: an alternative cost for spells cast with a permission  [Blue Mage's Cane; Jadzi, Oracle of Arcavios // Journey to the Oracle; The Infamous Cruelclaw]
    3  gap: an alternative cost for spells cast with a static permission  [Glimpse the Cosmos; Worldheart Phoenix; Valgavoth, Terror Eater]
    3  gap: remembering TriggerPlayer  [Sabertooth Cobra; Glass Asp; Nafs Asp]
    2  gap: a vote for objects  [Círdan the Shipwright; Mob Verdict]
    2  gap: one-use replacement effect  [Urabrask, Heretic Praetor; Moment of Silence]
    2  gap: qualities of an alternative list  [Leonardo, Sewer Samurai; Relic Axe]
    2  gap: replacement Discriminant  [Enduring Angel // Angelic Enforcer; Bitter Feud]
    1  gap: Modification  [Greymond, Avacyn's Stalwart]
    1  gap: TriggerCond  [Dream Devourer]
    1  gap: a remembered group no instruction names  [Tishana's Tidebinder]
    1  gap: remembering Union  [Eladamri, Korvecdal]
    1  gap: several remembered numbers  [The Seventh Doctor]
    1  gap: target slot 0 out of range  [Plague Reaver]
    9  oracle `the top x card of` vs rendered `a number of card from`  [Memorial Vault; Kotis, the Fangkeeper; Storybook Ride]
    8  oracle `you attacked with creature with` vs rendered `the total power of it`  [Minion of the Mighty; Werewolf Pack Leader; Hobgoblin Captain]
    5  oracle `it` vs rendered `that player`  [Soul Tithe; Pumpkin Bombs; The Fall of Kroog]
    4  oracle `each` vs rendered `your`  [Aku Djinn; Teferi, Mage of Zhalfir; Seeker of Slaanesh]
    4  oracle `from among it without paying` vs rendered `in a library from among`  [Velomachus Lorehold; Aetherworks Marvel; Kiora, Sovereign of the Deep]
    4  oracle `it is the second time` vs rendered `this is the second time`  [Belladonna Took; Saint Traft and Rem Karolus; Vito, Fanatic of Aclazotz]
    4  oracle `you discarded a card` vs rendered `there is a card you`  [Mysterio's Mirage; Gilt-Blade Prowler; Ragged Recluse // Odious Witch]
    4  oracle `{1} less to cast for` vs rendered `{x} less to cast where`  [Licia, Sanguine Tribune; Lumbering Megasloth; Valiant Changeling]

For each card: teach the renderer (`crates/mtg-engine/src/oracle/render/`) to word the AST
node from its meaning, independently of the parser. Triage every mismatch: parser bug first
(does the AST mean what the text says, in multiplayer, with control changes, tokens, copies,
replacement effects, zero-quantity choices, another player casting the card?) with a fix +
in-game test in `tests/cards/roundtrip_gaps_4.rs` that fails on the old code + a row in the
bug table (`FINDINGS` in crates/mtg-tools/src/roundtrip.rs); else renderer wording, or a
CR-justified equivalence. A gap where the AST can't express the text's meaning is usually a
parser bug. Add "Parser bugs found" and "Remaining" sections (each still-mismatching card
with its reason). Regenerate docs/ROUNDTRIP.md and docs/roundtrip-passing.txt;
`roundtrip --check` must pass.
