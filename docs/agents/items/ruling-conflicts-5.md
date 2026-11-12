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

## STATUS (Nov 12): conclusion C, implemented provisionally; four questions for the user

Investigated at effort max: an investigator, then an adversarial skeptic whose corrections were applied, then the implementation. Every source is quoted verbatim, with dates, in the register entry `returned-card-values-by-wording` (docs/interpretations/zones-objects.md; it replaces `returned-card-values-current-on-the-battlefield`, renamed), Kind interpretation, Status provisional.

**Conclusion: C.** The CR doesn't settle it and the rulings do, by wording, once each is dated. 608.2h doesn't say which zone a card the effect itself moved is "expected to be in" (it frames the question, as in the user-decided entry `exiled-card-power-as-it-last-existed-in-the-graveyard` of Nov 9; it isn't the basis); 400.7j lets the effect find the permanent without requiring it to; 109.2 and 109.2a are about nouns, not pronouns; 707.14 (Magar) is the only rule that reads a moved card in the zone it left. After an effect puts a card from a graveyard onto the battlefield:
- "that card's" mana value, power or toughness, and "its" mana value of a card a target or a choice found in a graveyard: the card as it last existed in the graveyard (Reanimate, Grave Researcher // Reanimate, Sister Hospitaller);
- "that creature's" values, type checks ("If it's an Elf") and instructions about the returned object: the permanent (Cleric Class, Vanish into Memory, Too Greedily, Too Deep; Return Upon the Tide, Defy Death, Essence Flux, Splash Portal);
- a card noun reads the card ("card vs permanent", not "graveyard vs battlefield"): after a permanent was exiled, the card in exile (Deicide, The Book of Vile Darkness); after a card left a graveyard, the card as it last existed there (Reanimate, Myrkul, Lord of Bones, Dread Defiler).

**Evidence summary.**
- The two rulings that looked contradictory were written for the same printed "its": Reanimate's in the Ultimate Masters release notes (2018-11-29, "You lose life equal to its converted mana cost."; Scryfall dated it 2018-12-07 and re-dated it 2025-09-19, text unchanged, per the MTGJSON snapshots of 2025-08-16 and 2025-11-10 in the taw/magic-search-engine index), Cleric Class's in the Adventures in the Forgotten Realms release notes (2021-07-12, "You gain life equal to its toughness."). From July 2021 both were live, one reading the graveyard and one the battlefield, until Wizards' Oracle Changes of 2022-09-02 (Jess Dunks) reworded Cleric Class to "that creature's toughness": "The intent of this effect was to refer to the creature on the battlefield, not the creature card in the graveyard ... it turns out that "it" has been used a little bit inconsistently on printed cards". Seventeen days later the same compiler gave Sister Hospitaller's "its mana value" the graveyard (Warhammer 40,000 Commander release notes, 2022-09-19). Reanimate's "its mana value" became "that card's mana value" between the index snapshots of 2025-12-13 and 2025-12-15, the only such change among that update's 261 Oracle changes; Sister Hospitaller, Death and Queen's Bay Paladin kept "its".
- Type checks read the permanent whatever the pronoun (Return Upon the Tide has Sister Hospitaller's exact template), so "it" alone doesn't tell the zone; the split between amounts and type checks describes the rulings rather than a rule.
- No ruling found contradicts the split under today's Oracle texts. The skeptic's confidence: about 80% overall; 85–90% for "its mana value" and "that card's" after a graveyard return; about 90% for "that creature's"; about 85% for type checks; 55–60% for extending the graveyard to "its power" and "its toughness".
- Survey of pool cards that read a value or a type of a card after moving it (rulings from data/rulings.jsonl.gz):

| Card | Move | Wording | Ruling (date) | Reads |
|---|---|---|---|---|
| Sister Hospitaller | graveyard → battlefield, target | its mana value | 2022-10-07 | graveyard |
| Reanimate | graveyard → battlefield, target | "its", "that card's" since Dec 2025 | 2025-09-19 (written 2018-11-29) | graveyard |
| Grave Researcher // Reanimate | graveyard → battlefield, target | that card's mana value | 2026-03-20 | graveyard |
| Cleric Class | graveyard → battlefield, target | printed "its toughness", "that creature's" since 2022-09-02 | 2021-07-23 | battlefield |
| Too Greedily, Too Deep | graveyard → battlefield, target | That creature ... its power | 2023-06-16 | battlefield |
| Vanish into Memory | battlefield → exile → battlefield | that creature's power, toughness | 2017-03-14 | last on the battlefield, then the returned permanent |
| Vivien's Invocation | library → battlefield, reflexive | it ... its power | 2018-07-13 | battlefield (last known if it left) |
| The First Tyrannic War | hand → battlefield, enters with | If its mana cost contains {X} | 2022-10-07 | as it enters (the copied permanent) |
| Turntimber Symbiosis | library → battlefield, enters with | If that card has mana value 3 or less | 2020-09-25 | library |
| Swords to Plowshares | battlefield → exile, target | its power | 2022-12-08 | battlefield (last known) |
| Myrkul, Lord of Bones | graveyard → exile | a copy of that card | 2022-06-10 | graveyard |
| Deicide | battlefield → exile, target | If the exiled card is a God card | 2014-04-26 | exile |
| Return Upon the Tide, Defy Death | graveyard → battlefield, target | If it's an [Elf/Angel] | 2021-02-05, 2012-05-01 | battlefield |
| Essence Flux, Splash Portal | battlefield → exile → battlefield | If it's a Spirit; If that creature is a ... | 2016-04-08, 2024-07-26 | battlefield |
| Kaya, Intangible Slayer | battlefield → exile, target | If it wasn't an Aura | 2023-02-04 | battlefield (last known) |
| Death, Queen's Bay Paladin | graveyard → battlefield, target | its mana value | none | (graveyard, by C) |
| Phyrexian Delver, Sheoldred's Restoration | graveyard → battlefield, target | that card's mana value | none on the zone | (graveyard, by C) |

**Changes** (branch wip/ruling-conflicts-5):
- Engine, `crates/mtg-engine/src/oracle/patterns/r608_moved_card_info.rs`: after a move onto the battlefield of a target or chosen card from a graveyard (`b.it == Var(IT)`), "... equal to its mana value" (mana value only; "its power"/"its toughness" untouched) for gaining or losing life and drawing compiles to the card's last known information (`Sel::Before(Var(IT))`), as "that card's" already did. Scope checked with a compile of every playable card before and after (a temporary example printing each face's `{:?}` of abilities, diffed with ability `uid`s masked, as those are process-local and shift): exactly Sister Hospitaller, Life // Death (Death) and Queen's Bay Paladin change; Terminal Velocity and the rest don't. Wonderscape Sage's compile differs between any two runs only in the order of an `Or` of land subtypes (nondeterministic, harmless; unrelated).
- Engine fix found by Deicide's ruling, `crates/mtg-engine/src/oracle/patterns/conditions_referents.rs` (`parse_condition_with`): "If the exiled card is a God card" after "Exile target enchantment" read the permanent as it last existed, so a Theros God that wasn't a creature (and so not a God on the battlefield) was never searched for; it now reads the card in exile (`vars::IT`, CR 400.7j), while "its controller" and "that card" stay the target. Same scan: Deicide only.
- Tests, `crates/mtg-engine/tests/rulings/returned_card_values.rs`: Sister Hospitaller (Clone returned as a copy of Grizzly Bears: 24 life), Death (16), Queen's Bay Paladin with Moritte of the Frost (15); each fails under the old battlefield reading (checked by reverting the pattern: 22, 18, 18); Deicide on a Heliod, God of the Sun that isn't a creature, leaving the hand's Heliod. The Cleric Class test (battlefield) now cites the new id, and the Reanimate and Phyrexian Delver tests in `r_p069_costs.rs` cite it too.
- Rulings newly cited: Sister Hospitaller's (left open by rulings-P155) and Deicide's two. The other halves of the split (Reanimate, Grave Researcher, Vanish into Memory, Too Greedily, Too Deep, Return Upon the Tide, Vraska's Stoneglare, ...) were already cited.
- `docs/RULING_DISCREPANCIES.md`: an inconsistent entry, status confirmed, for Cleric Class's and Reanimate's rulings (written for the same "its", resolved by the 2022-09-02 Oracle change; Reanimate's re-dating is in its evidence).

**For the user.**
1. Accept C for values: after a graveyard return, "its mana value" and "that card's" values are the card's in the graveyard, "that creature's" the permanent's? The alternatives are a uniform battlefield reading (sets aside Sister Hospitaller's and Reanimate's rulings), a uniform last-known-information reading (sets aside Cleric Class's, Vanish into Memory's, Too Greedily, Too Deep's and the type-check rulings), or C′ below.
2. "Its power" and "its toughness" after such a return: the engine reads the permanent (C′, by characteristic). No paper card has the template (Teyo, Aegis Adept's emblem is digital), and its only ruling was Cleric Class's under the printed text Wizards reworded. Keep C′, or extend the graveyard reading (C)?
3. A card that never arrives (Containment Priest exiles it instead; Grafdigger's Cage keeps it in the graveyard): `Var(IT)` is then empty and Reanimate makes its caster lose no life (checked: Hill Giant, 20 life either way); reading "that card" as the card (608.2h, and Vraska's Stoneglare's ruling for a target that wasn't destroyed) would lose 4. No ruling covers it for "that card's" or "its"; Call from the Grave (a playtest card outside the pool, "that creature's casting cost") deals no damage "In the case where no permanent was put onto the battlefield". A fix would read the target slot's card when nothing entered. Change it?
4. The noun rule as restated (the skeptic's correction): "card" reads the card and "creature"/"permanent" the permanent, wherever each now is, not "graveyard vs battlefield". Accept it as the general rule?
