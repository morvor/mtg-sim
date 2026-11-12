# ruling-conflicts-8: Shahrazad's subgame, who chooses to play or draw first

Run at effort max, with an independent investigator subagent and a separate adversarial skeptic subagent, as in ruling-conflicts-1. The user's standing rule: the CR and rulings should not contradict, and apparent conflicts are investigated before the user decides.

**Raised by:** rulings-P200 (landed da027caf). The ruling is left OPEN.

**Sources** (verify each in data/):
- Shahrazad ruling (2004-10-04): "You randomly chose which player chooses to go first or draw first."
- CR 729.2: "... Randomly determine which player goes first. The subgame proceeds like a normal game, following all other rules in rule 103, "Starting the Game."" (quote it in full).
- CR 103.1: "At the start of a game, the players determine which one of them will choose who takes the first turn." Also check 103.2 and its subrules.
- The P200 implementer reports that the CRs effective 2003-10-01 and 2004-10-01 already said, in 506.4: "The subgame proceeds like a normal game. Randomly determine which player goes first." Verify this from the archived CR texts (pit142857/mtg-cr).

**Questions.**
1. Are the two readings different in play? The ruling has a random player choose who goes first; the CR has a random player go first. Check whether 729.2's "following all other rules in rule 103" brings 103.1's choice back in.
2. If they do differ, classify the ruling: outdated, copying error, inconsistent.

**What the engine does:** it follows 729.2 (subgame.rs, a random player goes first, no choice).

**Outcome.**
- Reconcile, or apply an existing user policy (exemption plus a log entry), or leave the ruling OPEN for the user.
- Report with every source quoted verbatim with dates.
- Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.
