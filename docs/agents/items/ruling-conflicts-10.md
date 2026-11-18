# ruling-conflicts-10: who orders one player's per-object processes (Chaotic Transformation vs CR 608.2f)

Run at effort max, with an independent investigator subagent and a separate adversarial skeptic subagent, as in ruling-conflicts-1. The user's standing rule: the CR and rulings should not contradict; apparent conflicts are investigated before the user decides.

**Raised by:** rulings-P135 (landed f1d4559). The Chaotic Transformation ruling is left OPEN; logged in docs/RULING_DISCREPANCIES.md under "Not yet investigated".

**The conflict.**
- Chaotic Transformation (Oracle): "Exile up to one target artifact, up to one target creature, up to one target enchantment, up to one target planeswalker, and/or up to one target land. For each permanent exiled this way, its controller reveals cards from the top of their library until they reveal a card that shares a card type with it, puts that card onto the battlefield, then shuffles."
- Ruling (2022-09-09): "If a player controls multiple permanents that were exiled this way, that player first chooses which target they're considering, then they reveal cards until they reveal a card that shares a type with that permanent as it last existed on the battlefield, then they put that card onto the battlefield and shuffle. They then repeat this process for the remaining targets they controlled."
- CR 608.2f (CR effective 2026-09-25): "... APNAP order is used to make the primary determination of the order of those actions. Secondarily, if the action is to be taken on both a player and an object they control or on multiple objects controlled by the same player, the player who controls the resolving spell or ability chooses the relative order of those actions."
- They agree when the caster's own permanents are exiled; for another player's permanents the ruling has that player choose the order, the CR has the caster choose. The engine asks no one (a fixed order via simultaneous::for_each_object).

**Survey.** Find when 608.2f's "Secondarily" sentence was added (CR change history in data/ or release notes; the P135 reviewer's unverified recollection: around Soulfire Eruption, 2020) and whether the ruling predates or postdates it. Collect other rulings on the order of per-object processes for one player (Soulfire Eruption, Mythos/"for each" reveal-until cards, Proteus/Polymorph-style mass effects, Transmogrify-likes, Synthetic Destiny, Wild Swing etc.), with dates, and note who chooses in each. Check whether the ruling can be read as consistent (e.g. the ruling's "that player first chooses which target they're considering" read as the order the caster lets them, or the per-object action being that player's own action under another rule).

**Outcome.**
- **Reconciled** (a reading under which both hold, or a later CR change superseding the ruling with a dated source): implement 608.2f's order choice generally in simultaneous.rs (the order chooser asked per player's set of objects), cite the ruling (or exempt the superseded part per `ruling-contradicting-oracle-gives-way`-style policy only if justified), add or cite a register entry, log in docs/RULING_DISCREPANCIES.md.
- **They contradict:** change nothing in behavior beyond what both agree on; report both readings with every source quoted verbatim with dates, with a recommendation, for the user to decide.
- Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.
