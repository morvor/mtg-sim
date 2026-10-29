# Zones and objects

Judgment calls about zones and what's in them (CR 400–408). Format and kinds:
`docs/agents/items/interpretation-register.md`; index: `docs/INTERPRETATIONS.md`. What the
engine doesn't support at all (Contraptions, host/augment, sticker sheets, ...) is a scope
decision listed in `docs/DEFERRED.md`, not here.

## outside-the-game-is-the-sideboard: Cards a player brings in "from outside the game" come from their sideboard
- Kind: policy
- Status: decided
- Rules: 400.11 "An object is outside the game if it isn’t in any of the game’s zones. Outside the game is not a zone." / 400.11a "Cards in a player’s sideboard are outside the game. See rule 100.4." / 400.11b "Some effects bring cards into a game from outside the game. ..." / Eyetwitch rulings (shared by the Lesson cards): "If you bring a card into the game from outside the game in a Constructed tournament (or a Constructed game played under tournament conditions), you may select only a card from your sideboard. In casual Constructed games, the card comes from your collection." and "If you bring a card into the game from outside the game in a Sealed Deck or Booster Draft event, you may select only a card that is in the card pool that you built your deck from."
- Question: Which cards are outside the game for "a card you own from outside the game" (learn, wishes, Coax from the Blind Eternities)? Under the rulings it depends on the setting: the sideboard in tournaments, the card pool in Limited, the whole collection in casual games.
- Decision: A player's cards outside the game are the cards set up as their sideboard (`Game::add_to_sideboard`, in `Zone::Outside`); the engine has no collection.
- Alternatives: A configurable collection or card pool per player for casual or Limited games: closer to the casual ruling, but the "whole collection" would be an arbitrary list of cards.
- Why: The sideboard is the tournament reading and the only well-defined set of cards; deck setups that want a Limited pool put it in the sideboard.
- Affects: learn (Lessons), wishes (Burning Wish, Glittering Wish, ...), Coax from the Blind Eternities, Research // Development, The Raven's Warning, Eyetwitch.
- Code: `crates/mtg-engine/src/start.rs` (`Game::add_to_sideboard`); `crates/mtg-engine/src/object.rs` (`Zone::Outside`).
- Tests: `tests/cards/game_rules_outside.rs` (`coax_takes_an_eldrazi_from_outside_the_game_or_face_up_exile`).

## alteration-host-of-two-types: "Another permanent of that type" for a host with both named types
- Kind: interpretation
- Status: provisional
- Rules: 701.3a "To attach an Aura, Equipment, or Fortification to an object or player means to take it from where it currently is and put it onto that object or player. ..."
- Question: Enchantment Alteration moves "target Aura attached to a creature or land to another permanent of that type". For an Aura on a land creature (Dryad Arbor), "that type" could be either.
- Decision: A host that's a creature is treated as a creature (the first type named): the Aura moves to another creature.
- Alternatives: Let the caster choose either type, or allow any permanent sharing either type.
- Why: The text reads "that type" as one type; a host of both types is rare and the first named type is the simplest reading.
- Affects: Enchantment Alteration.
- Code: `crates/mtg-engine/src/oracle/patterns/attach_moves.rs` (`to_another_permanent_of_that_type`).
- Tests: `tests/cards/c4_attach_moves.rs` (`enchantment_alteration_moves_an_aura_to_a_permanent_of_its_hosts_type`, `enchantment_alteration_treats_a_land_creature_host_as_a_creature`).

## library-arranged-by-owner-not-instructing-player: Cards an effect puts into another player's library "in any order" are arranged by their owner
- Kind: deviation
- Status: provisional
- Rules: 401.4 "If an effect puts two or more cards in a specific position in a library at the same time, the owner of those cards may arrange them in any order. That library’s owner doesn’t reveal the order in which the cards go into the library." / 101.1 "Whenever a card’s text directly contradicts these rules, the card takes precedence. ..." / Misinformation ruling: "You don’t need to show the opponent what order you put the cards on top of their library." / Sealed Fate ruling: "The controller of this spell decides the order of the cards on the library."
- Question: When a spell's controller puts cards owned by another player into that player's library "in any order" (Misinformation, Drafna's Restoration, Lodestone Bauble, Agonizing Memories), who arranges them: the cards' owner (401.4) or the player the text instructs to put them (the card's "in any order", read as overriding 401.4, as the Misinformation and Sealed Fate rulings imply)?
- Decision: For cards moved from another zone (graveyard, hand) into a library, the owner arranges them (`zones::order_simultaneous`), whatever the text says. Cards rearranged within a library by a dig ("look at the top N cards of target opponent's library ... put the rest back in any order") are arranged by the player the text instructs, as Sealed Fate's ruling says (`dig_steps::rearrange`).
- Alternatives: Let the effect's controller arrange whenever the text says "in any order": matches the rulings, but the AST doesn't record "in any order" (the parser accepts and drops it, and the move pipeline knows only the moving player), so it needs a field on `Destination` and on the move events; and texts without "in any order" (Hallowed Burial) must keep 401.4's owner.
- Why: 401.4 is explicit and the AST can't yet tell the texts apart; the order of cards from a graveyard affects only which known card is drawn first. To fix: record "in any order" on `Destination` and carry the arranger through `MoveEv` to `order_simultaneous`.
- Affects: Misinformation, Drafna's Restoration, Lodestone Bauble, Agonizing Memories (about 4 cards).
- Code: `crates/mtg-engine/src/zones.rs` (`order_simultaneous`); `crates/mtg-engine/src/oracle/patterns/hand_graveyard_grammar.rs` (" in any order").
- Tests: `tests/rulings/r_p147_restock.rs` (`misinformation_the_cards_owner_arranges_them`).
