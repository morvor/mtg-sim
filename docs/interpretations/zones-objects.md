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

## looked-at-hand-card-stays-seen: A card looked at in another player's hand stays visible to the looker while it remains there
- Kind: interpretation
- Status: provisional
- Rules: 402.3 "A player may arrange their hand in any convenient fashion and look at it at any time. A player can’t look at the cards in another player’s hand but may count those cards at any time." / 701.20e "Some effects instruct a player to look at one or more cards. Looking at a card follows the same rules as revealing a card, except that the card is shown only to the specified player." / 400.7 "An object that moves from one zone to another becomes a new object with no memory of or relation to its previous existence. ..."
- Question: Urza's Bauble: "Look at a card at random in target player's hand." By 701.20e/701.20a the card is shown only for as long as the effect needs it, but the player remembers what they saw. Should the looker's view keep showing it?
- Decision: The ability's controller looks at one card chosen at random (only they see it, it isn't revealed) and may see that card for as long as it remains in that hand; once it changes zones it's a new object and the permission ends. In mtg-api observations the card shows in the looker's view.
- Alternatives: (A) Only a log line, as `Effect::LookAtHand` does: the looker learns the card but no view shows it. (B) Visible until the end of the resolution: the player "forgets" it, which a real player wouldn't.
- Why: The information was legitimately obtained and the card is the same object until it moves (400.7); keeping it visible models a player who remembers the card (a presentation choice: no game rule reads it). Only the looking player gains the permission; other players' views are unchanged.
- Affects: Urza's Bauble.
- Code: `crates/mtg-engine/src/kw/look_at_random.rs`; `crates/mtg-engine/src/zones.rs` (`allow_look`, `may_look`); `crates/mtg-engine/src/facedown.rs` (`can_look_at`).
- Tests: `tests/cards/paper_random_hand.rs` (`urzas_bauble_looks_privately_at_a_random_card`).

## the-token-another-ability-created: "The token" in another ability is the one token the card's token-making ability created
- Kind: interpretation
- Status: decided
- Rules: 607.1d "Abilities printed on two objects can be linked if one object is a token, emblem, or nontoken permanent and the second object was the source of the ability that either created the token or emblem or put that nontoken permanent onto the battlefield. ..." / Dance of Many's rulings: "Each Dance of Many is associated only with its token creature. If one Dance leaves the battlefield, only the corresponding token is affected, not all tokens from all instances of Dance of Many." and "If Dance of Many leaves the battlefield before its first ability has resolved, ... That token won't have any connection to a Dance of Many permanent".
- Question: Dance of Many's "exile the token" and "When the token leaves the battlefield" are in abilities other than the one that creates it. Which token do they mean?
- Decision: Only on a card with exactly one instruction that creates exactly one token ("create a token ..."), "the token" in its other abilities is the token that permanent's ability created (`Game::link_to_creator`): "exile the token" acts on it while it's on the battlefield, and "when the token leaves the battlefield" triggers for it, judged by its creator record as it left (look-back, CR 603.10a). A token created by the ability after the source left has no creator and isn't "the token".
- Alternatives: Any token with the same name, or every token the card's abilities ever created: both contradict the rulings.
- Why: The rulings; 607.1d's link between a token and the source of the ability that created it.
- Affects: Dance of Many.
- Code: `crates/mtg-engine/src/oracle/patterns/paper_referents.rs` (`act_on_the_token`, `the_token_leaves`); `crates/mtg-engine/src/kw/created_by_source.rs`.
- Tests: `tests/cards/paper_cost_token_referents.rs` (`dance_of_many_exiles_its_token_when_it_leaves`, `dance_of_many_is_sacrificed_when_its_token_leaves`).

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

## ice-cauldron-last-exiled: Ice Cauldron's "last card exiled with ~" is the last card its first ability exiled with a permission to cast it
- Kind: interpretation
- Status: provisional
- Rules: Ice Cauldron: "{X}, {T}: You may exile a nonland card from your hand. You may cast that card for as long as it remains exiled. ... / {T}, Remove a charge counter from this artifact: Add this artifact's last noted type and amount of mana. Spend this mana only to cast the last card exiled with this artifact." / 607.2a "If an object has an activated or triggered ability printed on it that instructs a player to exile one or more cards and an ability printed on it that refers either to “the exiled cards” or to cards “exiled with [this object],” these abilities are linked. ..."
- Question: Which card is "the last card exiled with" the Cauldron, and is it still that card once it's cast from exile?
- Decision: The card its first ability most recently exiled (and gave the permission to cast); the mana can be spent only to cast a spell that was that card in exile as it was cast (the spell is a new object, CR 400.7, whose previous object was the exiled card). If an activation exiled nothing, the card exiled before stays the last one.
- Alternatives: Track only the card exiled by the activation that put the most recent charge counter (the ruling's wording "the card that was exiled ... when the most recent charge counter was put there"), so that a charge without an exile allows no spell: the card's text says "the last card exiled", which an activation exiling nothing doesn't change.
- Why: The text names the most recent card the Cauldron exiled; the ruling describes the usual case, where each charge exiles a card.
- Affects: Ice Cauldron.
- Code: `crates/mtg-engine/src/cards/ice_cauldron.rs` (`on_event`, `custom_filter`).
- Tests: `tests/cards/m_ice_cauldron.rs` (`only_the_last_exiled_card_can_be_cast_with_the_mana`).

## cyclopean-tomb-land-choice: Cyclopean Tomb's controller chooses which of its lands loses its mire counters, among those with mire counters
- Kind: policy
- Status: provisional
- Rules: Cyclopean Tomb: "... at the beginning of each of your upkeeps for the rest of the game, remove all mire counters from a land that a mire counter was put onto with this artifact but that a mire counter has not been removed from with this artifact." / ruling: "As it resolves, you must remove a mire counter from a land that had a mire counter put on it by that instance of Cyclopean Tomb ..."
- Question: Several lands qualify: which one? And does a qualifying land that no longer has mire counters (they were removed some other way) use up an upkeep?
- Decision: The Tomb's controller chooses among the qualifying lands that still have mire counters (all qualifying lands if none has any). A land stops qualifying once mire counters have been removed from it this way; removing none (it had none) doesn't count as removing one "with ~".
- Alternatives: Choose among every qualifying land, including those with no mire counters, which lets the controller waste each upkeep's removal.
- Why: "Remove all mire counters from a land" leaves the choice of land to the player instructed; the ruling's "you must remove a mire counter" reads as the removal actually happening when it can.
- Affects: Cyclopean Tomb.
- Code: `crates/mtg-engine/src/cards/cyclopean_tomb.rs` (`custom_effect`, `REMOVE`).
- Tests: `tests/cards/m_cyclopean_tomb.rs` (`after_it_dies_each_upkeep_one_of_its_lands_loses_its_mire_counters`).

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

## attach-as-it-transforms-to-a-legal-player: Curse of Leeches is attached as it transforms to a player its controller chooses among the players it can enchant
- Kind: interpretation
- Status: decided
- Rules: Curse of Leeches: "As this permanent transforms into Curse of Leeches, attach it to a player." / 712.20 "If a double-faced card would have an “As [this permanent] transforms . . .” ability after it transforms or converts, that ability is applied while that permanent is transforming or converting, not afterward." / 701.3a "... An Aura, Equipment, or Fortification can’t be attached to an object or player it couldn’t enchant, equip, or fortify, respectively." / 701.3b "If an effect tries to attach an Aura ... to an object or player it can’t be attached to, the Aura ... doesn’t move." / 303.4f "If an Aura is entering the battlefield ... and the effect putting it onto the battlefield doesn’t specify the object or player the Aura will enchant, that player chooses what it will enchant as the Aura enters the battlefield. The player must choose a legal object or player according to the Aura’s enchant ability and any other applicable effects." / ruling: "Attaching Curse of Leeches as it transforms doesn't target a player, so it can be attached to a player with hexproof this way."
- Question: Who chooses "a player", and may the chooser pick a player it can't be attached to (one with protection from black), leaving it unattached so it's put into the graveyard (CR 704.5m)?
- Decision: Its controller chooses, among the players in range of influence it can legally enchant (hexproof doesn't matter: no targeting). With no such player it stays unattached, and the state-based action puts it into its owner's graveyard.
- Alternatives: Any player may be chosen, and choosing one it can't enchant leaves it unattached (701.3b), so the controller could choose to lose it.
- Why: 303.4f, the rule for an Aura attached without an Aura spell's target, requires a legal choice; transforming into an Aura in place is the same situation, and "attach it to a player" reads as an instruction to attach it, not an option to fail.
- Affects: Curse of Leeches // Leeching Lurker (the only card).
- Code: `crates/mtg-engine/src/cards/curse_of_leeches.rs` (`custom_effect`, `ATTACH`).
- Tests: `tests/cards/m_curse_of_leeches.rs` (`the_choice_is_among_players_it_can_enchant`).
