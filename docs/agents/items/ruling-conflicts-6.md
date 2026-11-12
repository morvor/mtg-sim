# ruling-conflicts-6: Headliner Scarlett, looking at a face-down exiled card after the turn ends

Run at effort max, with an independent investigator subagent and a separate adversarial skeptic subagent, as in ruling-conflicts-1. The user's standing rule: the CR and rulings should not contradict, and apparent conflicts are investigated before the user decides.

**Raised by:** rulings-P144 (landed 0873c2e4). The ruling is left OPEN and uncited.

**Sources** (verbatim; check each in data/):
- Headliner Scarlett: its Oracle text (quote it from data/).
- Ruling (2024-02-02): "Once the turn ends, you can't look at or play the card exiled with Headliner Scarlett that turn. It remains face down in exile, and no one can look at it."
- CR 406.3: "... However, if a player is instructed to look at a card and then exile it face down, or once a player is allowed to look at a card exiled face down, that player may continue to look at that card until it leaves the exile zone or is part of a pile of cards that are shuffled, even if the instruction allowing the player to do so no longer applies." The P144 worker says this has been present in substance since the CR of 2016-01-16, was reworded 2023-06-16, and was in force on 2024-02-02. Verify this from the archived CR texts if you can reach them.

**What the engine does:** it follows 406.3: the player keeps seeing the card in their observation. The "can't play" half agrees with the ruling and is pinned by tests/cards/permission_scopes.rs.

**Questions.**
1. Is it a real contradiction? Classify it with the discrepancy-log kinds: outdated, template error (compare other "exile face down ... look at and play this turn" cards' rulings), copying error (the release notes), inconsistent.
2. Do other cards of the same shape have rulings that agree with 406.3 or with this ruling? List them with dates.

**Outcome.**
- Report to the coordinator with every source quoted verbatim with dates and a recommendation.
- If the ruling is outdated, a template error or a copying error under an existing user policy: apply that policy (exemption plus a log entry) and say so.
- Otherwise leave it OPEN for the user's decision.
- Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.
