# ruling-conflicts-8: Shahrazad's subgame, who chooses to play or draw first (decided by the user)

**Raised by:** rulings-P200 (landed da027caf).

**The user decided on Nov 12, before any investigation:** "as for who goes first, the randomly selected player should always get the option to choose". This follows the Shahrazad ruling, and CR 729.2's "following all other rules in rule 103" brings in 103.1's choice. No investigation is needed. This is now an implementation item; run the usual implementer and reviewer flow (worker.md).

**Sources:**
- Shahrazad ruling (2004-10-04): "You randomly chose which player chooses to go first or draw first."
- CR 729.2: "... Randomly determine which player goes first. The subgame proceeds like a normal game, following all other rules in rule 103, "Starting the Game.""
- CR 103.1: "At the start of a game, the players determine which one of them will choose who takes the first turn."

**Task.**
1. In subgame.rs, a randomly selected player chooses who takes the first turn of the subgame, through a decision with the usual validation and default.
2. Check how the main game's start (start.rs, 103.1) asks the same question, and reuse it.
3. Cite the Shahrazad ruling in a test. Add a register entry `subgame-random-player-chooses-first` (decided by the user, Nov 12) that quotes the user, the ruling, 729.2 and 103.1.

Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.

**Done (implementer, Nov 13).** `subgame::begin` selects a random player (APNAP order of the main game) and asks them `Choose who takes the first turn` through `start::ask_who_takes_the_first_turn`, which the main game's `choose_starting_player` now shares: the subgame's players in turn order from the chooser, the chooser first and the default for an invalid or default answer. The answer becomes the subgame's starting player, and the subgame records the chooser (`StartState::chooser`). The selected player always chooses: an archenemy or a variant's seat doesn't go first instead. Register entry `subgame-random-player-chooses-first` (docs/interpretations/sources.md). Shahrazad's ruling is cited by `tests/cr/r729_subgames.rs` (4 of 4 Shahrazad texts cited; it wasn't exempt or listed as uncovered anywhere). The old check that "no player chooses" in `a_subgame_has_its_own_zones_made_from_the_main_game_libraries` was replaced by the new tests.

**Review (Nov 13).** That the selected player also chooses in a subgame of an Archenemy, Team vs. Team or Emperor game (rather than the archenemy going first, CR 103.1b and 904.6 "Rather than a randomly determined player, the archenemy takes the first turn of the game.", or the variant's pick, 808.4 and 809.4) goes beyond the question the user answered, so it has its own entry, `subgame-variants-dont-decide-who-goes-first` (players-multiplayer), **provisional** for the user to confirm; the engine keeps the reading it had before Nov 12. New tests: the chooser is offered the players in turn order from them (four players), one player per team with shared team turns (a Two-Headed Giant subgame: the starting team), and an Archenemy and a Team vs. Team subgame start with the player the chooser chose. The Shahrazad ruling is cited only by the test that casts Shahrazad, which now also checks that the chooser is selected at random.
