# ruling-conflicts-9: "search for exactly two cards ... that have different names" when only one name can be found

Run at effort max, with an independent investigator subagent and a separate adversarial skeptic subagent, as in ruling-conflicts-1. The user's standing rule: the CR and rulings should not contradict; judgment calls go in the register before the user decides.

**Raised by:** rulings-P181 (landed 1e73d1b). The engine makes an unregistered judgment call.

**The question.**
- Burning-Rune Demon (Oracle): "When this creature enters, you may search your library for exactly two cards not named Burning-Rune Demon that have different names. If you do, reveal those cards. An opponent chooses one of them. Put the chosen card into your hand and the other into your graveyard, then shuffle."
- Today, if the library holds only one distinct qualifying name, the engine finds just that one card (CR 701.23b) and the opponent's "choice" sends it to the hand. "Exactly two" could instead mean two or none (the search can't find a single card), or that the whole "if you do" fails.
- Relevant rules to quote in full: 701.23a, 701.23b ("... that player isn't required to find some or all of those cards even if they're present in that zone"), 701.23d, 701.23g, 608.2c/608.2d (doing as much as possible), 609.3, and any rule on "exactly".
- Related wordings to survey: Extrapolate the Impossible ("You may reveal exactly two cards you own with different names from outside the game. An opponent chooses one of them. ..."), Grown from the Stump ("Place exactly two cards named Hydra Head ... If you can't, ..."), and any other "exactly N" search/reveal and "different names" searches (Gifts Ungiven, Intuition, Fact or Fiction-style piles, Mythos of Brokkos, the Kaldheim demons' cycle-mates). Collect every ruling on finding fewer cards than asked, with dates; check the Kaldheim release notes text if present in data.

**Outcome.**
- **A source decides it** (a ruling or the CR plainly): make the engine follow it generally (one rule for "exactly N" searches, not a Burning-Rune Demon special case), with tests citing cr!/ruling!, and a decided-by-source register entry.
- **Nothing decides it:** keep or change the engine to the best-supported reading, add a PROVISIONAL register entry with the alternatives and every source quoted verbatim with dates, and pin it with a test using interp!.
- Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.
