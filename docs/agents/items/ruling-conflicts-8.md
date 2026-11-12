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
