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
- Rules: 402.3 "A player may arrange their hand in any convenient fashion and look at it as much as they want. A player can’t look at the cards in another player’s hand but may count those cards at any time." / 400.7 "An object that moves from one zone to another becomes a new object with no memory of or relation to its previous existence. ..."
- Question: Urza's Bauble: "Look at a card at random in target player's hand." The CR doesn't say for how long the looking player may keep seeing the card (a player remembers it; the card is shuffled among the others physically).
- Decision: The ability's controller looks at one card chosen at random (only they see it, it isn't revealed) and may see that card for as long as it remains in that hand; once it changes zones it's a new object and the permission ends. In mtg-api observations the card shows in the looker's view.
- Alternatives: (A) Only a log line, as `Effect::LookAtHand` does: the looker learns the card but no view shows it. (B) Visible until the end of the resolution: the player "forgets" it, which a real player wouldn't.
- Why: The information was legitimately obtained and the card is the same object until it moves (400.7); keeping it visible is how a player who remembers the card would play.
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
