# ruling-conflicts-5: a returned card's "its" value, read in the graveyard or on the battlefield

Run at effort max, with an independent investigator subagent and a separate adversarial skeptic subagent, as in ruling-conflicts-1. The user's standing rule: "the CR and rulings should not contradict"; apparent conflicts are investigated before the user decides.

**Raised by:** rulings-P155 (landed 721818e). Provisional register entry `returned-card-values-current-on-the-battlefield` (docs/interpretations/zones-objects.md). Sister Hospitaller's ruling is left OPEN and uncited.

**The question.** An effect returns a card from a graveyard to the battlefield, then uses a value of it. Is the value the card's last known information in the graveyard, or the permanent's current value on the battlefield? The two differ when the card enters as a copy, with counters, or with modified characteristics. Today the engine reads the battlefield for every wording.

**Sources** (verbatim; check each in data/):
- Sister Hospitaller (Oracle: "When this creature enters, return target creature card from your graveyard to the battlefield. You gain life equal to its mana value."). Ruling (2022-10-07): "Use the mana value of the card as it last existed in the graveyard to determine the amount of life you gain."
- Reanimate (Oracle: check data/; "... You lose life equal to its mana value."). Ruling (2025-09-19): "The amount of life you lose is determined by the mana value of the card in your graveyard, not the creature once it's on the battlefield." This is recent and the same template as Sister Hospitaller. Check what the engine does for Reanimate today and whether a test pins it.
- Cleric Class (level 3: "When this Class becomes level 3, return target creature card from your graveyard to the battlefield. You gain life equal to that creature's toughness."). Ruling (2021-07-23): "When Cleric Class's level 3 ability resolves, you gain life equal to the creature's toughness as it exists on battlefield, which may be different than its toughness in the graveyard."
- Vanish into Memory. Rulings (2017-03-14): "Vanish into Memory cares about the creature's power just before it left the battlefield and its toughness just after it returns to the battlefield. ..." and "... Since the card returned to the battlefield has no toughness, you discard no cards."
- CR 608.2h, 400.7j, 113.7a; find any others that bear on it.

**Hypothesis to test (not to assume):** the wording decides. "its [value]" refers to the targeted card, the object the effect found in the graveyard, so it uses that object's last known information (608.2h: it is no longer in the zone it was expected to be in). "that creature's [value]" names a creature, which exists only on the battlefield (400.7j lets the effect find it there).

**Survey.**
1. Find every card with "return/put ... from [a] graveyard to/onto the battlefield" followed by "its"/"that creature's"/"that card's"/"that permanent's" + a value, and every ruling on any of them that says which reading applies (`zcat data/rulings.jsonl.gz`).
2. Group them by wording and by date. Does any ruling cut across the hypothesis?

**Outcomes.**
- **The wording split holds across the rulings:**
  - implement it generally in the compiler and eval: "its" after such a return reads the card's last known information from the zone it left; "that creature's" reads the battlefield;
  - cite Sister Hospitaller, Reanimate, Cleric Class and Vanish into Memory;
  - rewrite the register entry as an interpretation, provisional until the user decides;
  - report to the coordinator with every source quoted verbatim with dates.
- **The rulings contradict with no consistent split:** change nothing in the engine. Write up both readings with the evidence and report for the user to decide (the coordinator presents it).

Either way, follow worker.md's one-item-per-session rule and notify the coordinator when it lands.
