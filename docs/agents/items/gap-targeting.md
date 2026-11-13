# gap-targeting: Target and choice constraints across several objects; exact target counts

GOAL: Targets (and untargeted multi-object choices) can only be constrained one object at a time. Rules text frequently constrains them as a group (same graveyard, shared type, different controllers, one per opponent, total mana value, each mode a different player) and sometimes requires an exact number that depends on choices made while casting.

GAPS:

[G06] Target constraints across several targets (group requirements)
- Rules: CR 115.1, 115.3, 601.2c (targets chosen with all their requirements), 700.2 (modes), 115.10
  ("each mode must target a different player").
- Engine now: `TargetSpec` (ability.rs:584-601) has only `what`, `min`, `max`, `distinct_from`, `divide`,
  `chosen_by_opponent`, `condition`. Nothing can require targets to be in the same zone/graveyard, to have
  different or the same controllers, to share a creature/card type, to have equal toughness, to have a total
  mana value/power at most N, or to be one per opponent ("for each opponent, up to one target creature that
  player controls"). No code in the engine checks such relations (no hits for "single graveyard",
  "different controller", "same controller" outside oracle/).
- Cards (UNS): Decompose, Arashin Sunshield, Famished Ghoul, Unlicensed Hearse, Pestilent Cauldron, Martyr
  of Bones ("from a single graveyard"); Return from Extinction, Unbury, Raise the Draugr, Secret Tunnel
  ("that share a creature type"); The Trickster-God's Heist ("share a card type"); Simic Guildmage ("another
  target creature with the same controller"); Cloud, Midgar Mercenary/Limit Break ("with different
  controllers"); V.A.T.S. ("with equal toughness"); The Super Hero Civil War, Tocasia, Dig Site Mentor and
  Joshua's chapter III ("total mana value N or less"); Mass Mutiny, Molten Primordial, Hideous Taskmaster,
  In the Darkness Bind Them, Bronzebeak Foragers, Diluvian Primordial, Sheoldred // The True Scriptures,
  Age of Ultron, Battle at the Helvault, Vault 13, The Parting of the Ways ("for each opponent/player, ...
  up to one target ... that player controls"); Vindictive Lich, Balor, Shadrix Silverquill, Splinter & Leo
  ("Each mode must target a different player").
- The same relations are missing for untargeted choices of several cards (searches, "choose", costs):
  "with different names" (Saheeli Rai -X, Transmutation Font, Ormos, Archive Keeper, Battle for Bretagard),
  "for each card type, ... a card of that type" (Atraxa, Grand Unifier), "total mana value X or less" (Rod of
  Absorption); `library::search` and `ask_objects` validate only count and filter per card
  (library.rs:106-135).
- Rulings (UNCOVERED): Return from Extinction / Unbury / Raise the Draugr "the cards must share at least one
  creature type"; Vindictive Lich "You can't choose more modes ... than you have opponents".
- Approach: add a `group: Vec<TargetGroupRule>` to the ability's targeting (SameZone, SameController,
  DifferentControllers, ShareCreatureType, ShareCardType, EqualToughness, TotalAtMost(Value,
  stat), OnePerPlayer(PlayerFilter)) checked in target_rules.rs when choosing (601.2c) and, where the rules
  say so, on resolution (608.2b only checks each target individually, so most are choice-time only); a
  per-player slot expansion for "for each opponent ... target"; a cross-mode "different player" rule.
- Size: L

[G07] Exact target counts that depend on choices (multikicker, "X target creatures")
- Rules: CR 601.2c, 702.33d (multikicker), 115.1 ("X target creatures" means exactly X).
- Engine now: `TargetSpec.min` is a fixed `u32` (ability.rs:586) while `max` is a `Value`; "X target
  creatures"/"one more than the number of times it was kicked" compile as min 1 (or 0), max X, so fewer
  targets may be chosen and more kicks than targets are allowed.
- Cards: Comet Storm, Strength of the Tajuru (supported, but approximate), Thrive, Rot-Curse Rakshasa
  ("each of X target creatures").
- Rulings (UNCOVERED): Comet Storm / Strength of the Tajuru "The number of targets you choose for ... is one
  more than the number of times it's kicked".
- Approach: make `min` a `Value` (about 36 constructions; `TargetSpec::one/up_to` helpers keep most call
  sites unchanged) and evaluate both bounds after X/kicks are announced (601.2b before 601.2c).
- Size: S-M

ESTIMATED SIZE: G06 L, G07 S-M.

OVERLAPS: Touches `TargetSpec` and target_rules.rs; gap-effect-language and gap-variants-misc don't. The 'single graveyard' cost of G20 (gap-spell-costs) needs this item's group rule.

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
