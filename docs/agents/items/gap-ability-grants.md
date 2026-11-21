# gap-ability-grants: Layer 6 and linked abilities: gaining others' abilities, partial keyword loss, ability-quality hexproof, linked targets

GOAL: Ability-changing effects are missing three capabilities: having all activated (and triggered) abilities of other objects, losing one instance of a parametrised keyword (protection from red, islandwalk, 'bands with other'), and hexproof qualities that describe the targeting ability; plus linked abilities that refer to a player or object a partner ability targeted.

GAPS:

[G11] Gaining the activated (and triggered) abilities of other objects
- Rules: CR 613.1f (layer 6 ability-adding effects), 613.8 (dependency: what the effect copies depends on
  other layer-6 effects), 607.5 (linked pairs gained together), 201.5b (a gained ability that names its
  original object uses the new object's name; Necrotic Ooze ruling).
- Engine now: `Modification` (ability.rs ~1520-1560) has `AddAbility(Ability)`, `AddKeyword`,
  `AddKeywordsOf { kinds, from }` (keywords only) and `AddThisAbility`, but nothing that copies the
  activated/triggered abilities of the objects a filter/selection names, re-evaluated continuously.
- Cards (UNS): Necrotic Ooze, Experiment Kraj, Quicksilver Elemental ("gains all activated abilities of
  target creature until end of turn"), Agatha's Soul Cauldron, Koh, the Face Stealer (activated and
  triggered), Steward of the Harvest, Manascape Refractor, Skill Borrower, Myr Welder, Idris, Soul of the
  TARDIS.
- Rulings: Necrotic Ooze "gains only activated abilities. It doesn't gain keyword abilities (unless those
  keyword abilities are activated)", "references the card it's printed on by name ... as though it
  referenced Necrotic Ooze"; Experiment Kraj "costs of the activated abilities ... must be paid with the
  correct colors"; S27 #63/#66 colon rulings (Steward of the Harvest/Skill Borrower, Koh/Idris).
- Approach: `Modification::AddAbilitiesOf { from: Sel|Filter, kinds: {Activated, Triggered}, zone }` in
  layer 6, computed after the source objects' own layer-6 state (dependency per 613.8 since the copied set
  depends on other layer-6 effects), with `~` in the copied text rebound to the gaining object and
  link ids preserved (607.5).
- Size: M

[G12] Losing or targeting by part of a keyword: one protection/landwalk instance; "hexproof from" abilities
- Rules: CR 702.14 (landwalk variants are separate abilities), 702.16 (each "protection from X" is a
  separate ability; losing "protection from red" leaves others), 613.1f; 702.11d ("hexproof from
  [quality]" where the quality can describe the spell or ability itself).
- Engine now: `Modification::RemoveKeyword(KeywordKind)` (ability.rs:1551) removes every instance of a
  kind; there's no way to remove "protection from red" or "islandwalk" only. Hexproof-from qualities are
  matched only against the ability's *source* (`object_untargetable`, stack.rs:457-475), so "hexproof from
  activated and triggered abilities" can't be expressed.
- Cards (UNS): Mystic Decree ("All creatures lose flying and islandwalk"), Shelkin Brownie ("loses all
  'bands with other' abilities", which must not remove plain banding), Volatile Stormdrake ("hexproof
  from activated and triggered abilities"; ruling "can't be the target of any activated or triggered abilities your opponents control").
- Approach: `RemoveKeyword` with an optional parameter/filter match (`RemoveKeywordInstance(Keyword)`);
  let hexproof/protection quality filters also be matched against the targeting stack object (kind:
  spell/activated/triggered).
- Size: S

[G15] Linked abilities that refer to the player or object a previous ability affected (CR 607.1)
- Rules: CR 607.1 (one ability affects objects or *players*, the other refers to them), 607.1d (linked
  across a created token), 607.3.
- Engine now: linked data exists for exiled cards (`GameObject::linked`, `Sel::Linked`, eval.rs:907) and
  for choices (`linked_choices` with a `player` field, eval.rs:197-210, 707-710), but no effect records
  *the target* of one ability for its linked partner, and `Effect::Choose` only records choices made with
  a `ChoiceKind`.
- Cards (UNS): Laquatus's Champion, Soul Scourge ("When ~ leaves the battlefield, that player gains 6
  life"), Ugin, the Ineffable ("When that token leaves the battlefield, put the exiled card into your
  hand", linked across the token, 607.1d).
- Rulings: Laquatus's Champion "the player who gains 6 life is the player who was the target of the first
  ability (or, if that ability is still on the stack, the player who is its target)".
- Approach: an `Effect::NoteLinked { what: Sel }` (or a flag on targets) that stores entities in the
  source's linked data under the ability's link id, plus `Sel::LinkedPlayer`; for a still-pending
  first ability, read the target from the stack object (the ruling's parenthetical).
- Size: S-M

ESTIMATED SIZE: G11 M, G12 S, G15 S-M.

OVERLAPS: Layer 6 work also touched by gap-zone-moves (enchant swap).

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
