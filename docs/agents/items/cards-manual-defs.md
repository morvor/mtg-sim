# cards-manual-defs: Hand-written card definitions: src/cards/ registry, compiler hook, coverage reporting and guards; first 32 of 83 one-off cards

SCOPE: the mechanism for hand-written ("manual") card definitions, plus the first 32 manual cards. The candidate list ships as manual-candidates.tsv (92 ability texts on 83 cards: card, normalized text, why it's a one-off). The card items may add a few more one-offs later, each with a test.

GOAL: card support must reach 100% of paper cards (Contraptions and host/augment excepted). The Oracle compiler covers everything that shares a construct with another card; genuinely unique one-offs get definitions written directly in the ability AST, with engine hooks where the AST can't express the rule. card-coverage counts them as supported and reports them separately; every manual card has tests.

READ FIRST: crates/mtg-engine/build.rs (auto-included module dirs), src/card.rs (CardDef::from_scryfall, FaceDef.unsupported, is_fully_supported), src/oracle/mod.rs (compile, split_abilities, parse_ability), src/oracle_ext.rs, src/custom.rs and src/kw/mod.rs (KeywordRules hooks and the registry-wide custom_* dispatch), crates/mtg-tools/src/main.rs (card_coverage, unsupported), src/testing.rs.

MECHANISM (make it concrete; adjust names to the code you find):

1. Registry `crates/mtg-engine/src/cards/`: one file per card (`<snake_case_card_name>.rs`), auto-included by build.rs exactly like `src/kw/` (add `gen(&root.join("src/cards"), &["mod"], &out.join("card_mods.rs"), "pub ")` and `include!(concat!(env!("OUT_DIR"), "/card_mods.rs"));` in `cards/mod.rs`; `pub mod cards;` in lib.rs). `cards/mod.rs` defines

       pub struct ManualAbility {
           pub card: &'static str,     // Oracle card name (full name for multi-face cards)
           pub face: usize,            // face/half index (0 = front/left)
           pub text: &'static str,     // the exact normalized ability block it replaces, as it
                                       // appears in FaceDef::unsupported today (reminder text
                                       // stripped, self-references "~", original numbers)
           pub build: fn(&crate::oracle::CompileContext) -> Vec<crate::ability::Ability>,
           pub reason: &'static str,   // why this is hand-written (shown in reports)
       }
       inventory::collect!(ManualAbility);
       pub fn lookup(full_name: &str, face: usize, block: &str) -> Option<&'static ManualAbility>

   Per-ability (block) granularity, not per card: most manual cards have one unique ability and their other abilities still compile (and items may fix them). A card file may register several ManualAbility entries.

2. Compiler hook (oracle/mod.rs `compile`): before `parse_ability(&block, ctx)`, `if let Some(m) = crate::cards::lookup(ctx.full_name, ctx.face_index, &block)` → append `(m.build)(ctx)` and record the block in a new `Compiled::manual: Vec<String>` (plumbed to `FaceDef::manual` and `CardDef::manual_text()`); skip parsing. Exact-text matching means an Oracle update makes the definition stale and the block reverts to unsupported (safe default). `is_fully_supported()` is unchanged (manual blocks are not unsupported).

3. Behaviour the AST can't express: the card file may also implement the existing `KeywordRules` trait (kw/mod.rs) with `kinds() -> &[]` and register it with `inventory::submit! { KeywordRegistration(&XRules) }`; it then receives the registry-wide hooks — `custom_effect/custom_condition/custom_trigger/custom_value/custom_filter` (already dispatched over the whole registry, kw/mod.rs ~891-969; name them `card:<Card Name>:<what>` and reference them from the AST via `Effect::Custom`, `Condition::Custom`, `TriggerCond::Custom`, `Value::Custom`, `Filter::Custom`, `StaticEffect::Custom`/`Restriction::Custom`), plus global hooks such as `state_based_actions`, `static_state_checks`, `combat_damage_assigner`, `activation_cost`, `block_allowed`, `attack_declaration_ok`, `damage_prevention`, `on_event`, `after_draw`. For hooks the engine dispatches only to keyword kinds present on an object, add one generic dispatch (iterate all registrations) — a small focused core edit, never a card-name check in the engine. Hooks must find their source permanents themselves (by the custom ability on the object, not by name, so copies and text changes behave: a copy of Trinisphere copies its ability and therefore its rule).

4. Reporting and guards (mtg-tools): `card-coverage` prints "Fully supported: N (compiled: A, with hand-written abilities: B)" and lists hand-written cards with their reasons in docs/CARD_COVERAGE.md; a new `mtg-tools manual-check` (run by a test or by `cargo test` via a test in crates/mtg-engine/tests/cards/) fails when (a) a ManualAbility matches no block of its card in the current data (stale), (b) the compiler can now parse the block on its own (delete the manual definition: compile-first policy), (c) a manual card has no test (a test in tests/cards/ that loads the card by name, e.g. a `manual_cards_tested` list or grep for `"<Card Name>"`), (d) two entries claim the same block.

5. Tests: one file per card, `crates/mtg-engine/tests/cards/m_<card>.rs`, using TestGame with the real card and asserting the in-game behaviour of the hand-written ability (cite cr!/ruling! where the test exercises a rule or ruling; rulings coverage benefits).

6. Round-trip interplay (item oracle-roundtrip): hand-written abilities are verified by their tests, not by rendering; the renderer should report them as "manual" (counted separately, not as mismatches). Mention this in docs.

7. Policy (document in the cards/mod.rs header and CLAUDE.md "Extending"): hand-written definitions are only for genuine one-offs (no other card shares the construct). Any construct shared by two or more cards must be compiled. Every entry states its reason. Card items may add entries (with tests) for one-offs they find, listing them in their report.


FIRST BATCH (implement and test these; they exercise different hook kinds — statics via custom names, SBA/cleanup/combat/cost hooks, triggers, one-shot effects):
- Trinisphere: "As long as ~ is untapped, each spell that would cost less than three mana to cast costs three mana to cast." — cost floor of three mana (CR 601.2f after reductions): unique cost rule
- Ancient Adamantoise: "Damage isn't removed from ~ during cleanup steps." — damage isn't removed during cleanup (CR 514.2 exception): unique
- Zilortha, Strength Incarnate: "Lethal damage dealt to creatures you control is determined by their power rather than their toughness." — lethal damage determined by power rather than toughness (CR 704.5g): unique
- Remove Enchantments: "Return to your hand all enchantments you both own and control, all Auras you own attached to permanents you control, and all Auras you own attached to attacking creatures your opponents control. Then " — bounce enchantments by owner/controller/attachment combinations: unique
- Ogre Enforcer: "~ can't be destroyed by lethal damage unless lethal damage dealt by a single source is marked on it." — can't be destroyed by lethal damage unless a single source dealt it (CR 704.5g exception): unique
- Ghostly Flame: "Black and/or red permanents and spells are colorless sources of damage." — black/red permanents and spells are colorless sources of damage: unique damage-source color rule
- Loot, the Anomaly: "If ~'s power is negative, he assigns combat damage as though his power were positive." — assigns combat damage as though negative power were positive: unique
- Archon of Coronation: "As long as you're the monarch, damage doesn't cause you to lose life." — while you're the monarch, damage doesn't cause you to lose life: unique
- Phyrexian Unlife: "As long as you have N or less life, all damage is dealt to you as though its source had infect." — damage dealt to you as though its source had infect while at 0 or less life: unique
- Magnetic Web: "If a creature with a magnet counter on it attacks, all creatures with magnet counters on them attack if able." — attack/block requirements for all creatures with magnet counters: unique pair of abilities
- Magnetic Web: "Whenever a creature with a magnet counter on it attacks, all creatures with magnet counters on them block that creature this turn if able." — attack/block requirements for all creatures with magnet counters: unique pair of abilities
- Celestial Convergence: "At the beginning of your upkeep, remove an omen counter from ~. If there are no omen counters on ~, the player with the highest life total wins the game. If two or more players are tied for highest li" — the player with the highest life total wins (draw on a tie): unique
- Approach of the Second Sun: "If ~ was cast from your hand and you've cast another spell named ~ this game, you win the game. Otherwise, put ~ into its owner's library seventh from the top and you gain N life." — win if cast from hand and another Approach was cast this game; put seventh from the top: unique
- Common Cause: "Nonartifact creatures get +N/+N as long as they all share a color." — anthem conditional on all nonartifact creatures sharing a color: unique
- Mana Maze: "Players can't cast spells that share a color with the spell most recently cast this turn." — can't cast spells sharing a color with the spell most recently cast this turn: unique
- Damping Engine: "A player who controls more permanents than each other player can't play lands or cast artifact, creature, or enchantment spells. That player may sacrifice a permanent of their choice for that player t" — player with the most permanents can't play lands or cast permanent spells unless they sacrifice: unique
- Arboria: "Creatures can't attack a player unless that player cast a spell or put a nontoken permanent onto the battlefield during their last turn." — can't attack a player unless that player cast a spell or put a nontoken permanent onto the battlefield during their last turn: unique
- Peace Talks: "This turn and next turn, creatures can't attack, and players and permanents can't be the targets of spells or activated abilities." — two-turn duration ('this turn and next turn') for attack and targeting bans: unique
- Wall of Shadows: "~ can't be the target of spells that can target only Walls or of abilities that can target only Walls." — can't be targeted by spells that can target only Walls: unique targeting test on the spell's legal-target set
- Defensive Formation: "Rather than the attacking player, you assign the combat damage of each creature attacking you. You can divide that creature's combat damage as you choose among any of the creatures blocking it." — defending player assigns attackers' combat damage: unique
- Butcher Orgg: "You may assign ~'s combat damage divided as you choose among defending player and/or any number of creatures they control." — divides its combat damage among defending player and their creatures (CR 510.1 exception): unique
- Opposition Agent: "While an opponent is searching their library, they exile each card they find. You may play those cards for as long as they remain exiled, and you may spend mana as though it were mana of any color to " — opponents exile cards they find while searching and you may play them: unique search replacement
- Silhouette: "Choose target creature. If a spell or ability that targets that creature would cause a source to deal damage to that creature this turn, prevent that damage." — prevents damage from spells/abilities that target the creature: unique
- Duplicant: "As long as a card exiled with ~ is a creature card, ~ has the power, toughness, and creature types of the last creature card exiled with it. It's still a Shapeshifter." — P/T and creature types of the last creature card exiled with it: unique last-exiled tracking
- Jandor's Ring: "{N}, {T}, Discard the last card you drew this turn: Draw a card." — cost: discard the last card you drew this turn: unique cost tracking
- Fatespinner: "At the beginning of each opponent's upkeep, that player chooses draw step, main phase, or combat phase. The player skips each instance of the chosen step or phase this turn." — opponent chooses a step/phase to skip: unique
- Time and Tide: "Simultaneously, all phased-out creatures phase in and all creatures with phasing phase out." — simultaneous phase in/out of different groups: unique
- Sands of Time: "At the beginning of each player's upkeep, that player simultaneously untaps each tapped artifact, creature, and land they control and taps each untapped artifact, creature, and land they control." — simultaneously untaps tapped and taps untapped permanents; skip untap step: unique
- Noxious Vapors: "Each player reveals their hand, chooses one card of each color from it, then discards all other nonland cards." — each player keeps one card of each color and discards the rest: unique
- Dead Ringers: "Destroy two target nonblack creatures unless either one is a color the other isn't. They can't be regenerated." — destroy unless either target is a color the other isn't: unique comparison
- Akroma, Vision of Ixidor: "At the beginning of each combat, until end of turn, each other creature you control gets +N/+N if it has flying, +N/+N if it has first strike, and so on for double strike, deathtouch, haste, hexproof," — +1/+1 per keyword from a list ('and so on'): unique
- Spy Network: "Look at target player's hand, the top card of that player's library, and any face-down creatures they control. Look at the top four cards of your library, then put them back in any order." — look at hand, top card of library and face-down creatures, then rearrange: unique
- Mana Web: "Whenever a land an opponent controls is tapped for mana, tap all lands that player controls that could produce any type of mana that land could produce." — tap all lands that could produce a type of mana the tapped land could produce: unique

Texts in manual-candidates.tsv are in the planning normal form (numbers as N, like `mtg-tools unsupported`); a ManualAbility must match the real block with its numbers — print it from `card("Name").unsupported_text()` (or have the tool print raw blocks).

For each card: read its Oracle text and rulings (`zcat data/rulings.jsonl.gz | grep -i '<name>'`), write the definition from the AST (prefer existing AST nodes; use custom hooks only for the unique rule), write tests in tests/cards/m_<card>.rs that would fail if the behaviour were wrong, and cite the CR rules exercised (e.g. 704.5g for Zilortha/Ogre Enforcer, 514.2 for Ancient Adamantoise, 601.2f for Trinisphere, 510.1 for Butcher Orgg/Defensive Formation).

DONE: mechanism landed with the guards (stale/compilable/untested checks) and reporting; CLAUDE.md "Extending" section documents src/cards/ and the one-off policy; the first batch is supported with tests; card-coverage shows the compiled/manual split. Report: coverage before/after, the manual cards added, hooks added to the engine, and any candidate you found to be compilable after all (move it back to the card items).
