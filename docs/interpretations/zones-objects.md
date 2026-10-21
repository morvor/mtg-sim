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
