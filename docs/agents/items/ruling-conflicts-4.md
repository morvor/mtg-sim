# ruling-conflicts-4: follow-up conflict cases (Nov 5, third set)

Same method, conclusion types and actions as `ruling-conflicts-1.md` to `-3`: an independent investigator subagent and an adversarial skeptic subagent for each case, at effort max, run as separate subagents. Also:
- quote the sources with dates;
- apply the Nov 5 policies (`ruling-contradicting-oracle-gives-way`, misattached rulings, copying errors);
- log every finding in docs/RULING_DISCREPANCIES.md;
- add a STATUS section at the end when done.

**Cases:**
1. **Fiery Annihilation / Searing Blaze** (logged by cards-1 from rulings-P043 or P047; see docs/agents/engine-gaps.md "Apparent conflicts (Nov 5)").
2. Any other entry in that list not yet investigated in ruling-conflicts-1 to -3.

DONE as in ruling-conflicts-3.

## STATUS

Done (Nov 6). Conclusion types: A reconciled, B outdated, C settled by the ruling, D unresolved.
For each case an independent investigator and a separate adversarial skeptic subagent reported
(read-only); the skeptic's corrections were applied. Sources outside `data/` were the archived CR
texts (https://github.com/pit142857/mtg-cr), the ormos mirror of the Wizards article archive (up to
2022) for release notes and update bulletins, and, for Stone Calendar's Oracle history, the public
repositories Card-Forge/forge, magefree/mage and taw/magic-search-engine (MTGJSON-built index);
magic.wizards.com, web.archive.org and Scryfall's API were outside the allowed hosts. The two cases
were the only open entries of the "Apparent conflicts (Nov 5)" list (`docs/agents/engine-gaps.md`)
and of "Not yet investigated" in `docs/RULING_DISCREPANCIES.md`; none is left.

| # | Case | Conclusion | Commit(s) |
|---|---|---|---|
| 1 | Fiery Annihilation (2024-11-08): "If the target creature is an illegal target when Fiery Annihilation begins to resolve, the target Equipment (if any) won't be exiled." against Searing Blaze (2010-03-01): "If either target is illegal by the time Searing Blaze resolves, it still deals damage to the other target." | D: on the exact structure (a second "target" described by the first, the first illegal but still there) the post-2015 rulings are one each way: Yosei, the Morning Star (2017-11-17) "If the player becomes an illegal target, the target permanents are tapped, but the player doesn't skip their untap step." vs Fiery Annihilation. 608.2b's information sentence dates from the CR effective 2015-07-17 (before: "The effect may still determine information about illegal targets, though, ..."), 608.2g lost "or a target that's become illegal" in the CR effective 2016-07-22, but Yosei's and Run Away Together's rulings (the latter republished 2025-11-17: "can still determine its controller only to check whether the other creature is a legal target") show the change wasn't read as reaching the target check, so not B; no reading grounded in the CR reconciles them, so not A. Recommended reading R1 (the engine's): the literal CR (the sentence is about "part of the effect", 609.1), Run Away Together, and 115.9b by analogy. Memory's Journey's ruling (2011-09-22) agrees with Fiery Annihilation's outcome under every reading (the illegal player is the one who would shuffle); the earlier note that it went the other way is corrected. Engine unchanged; new provisional register entry `dependent-target-checked-against-an-illegal-target`; new tests; Memory's Journey's, Witness the Future's and Yosei's player rulings cited; Fiery Annihilation's ruling uncited and unexempted, logged as inconsistent (open for review) | 35785c12; docs and comments in the STATUS commit |
| 2 | Stone Calendar (2004-10-04): "You may choose not to apply Stone Calendar’s cost reduction effect." against its Oracle text "Spells you cast cost {1} less to cast." | B: the ruling glosses an "up to" wording (oldest capture found: Forge's card script, 2011-12-30, "Spells you cast cost up to {1} less to cast."; Wizards' Oracle text for Fluctuator kept "up to" in the 2009-04-22 and 2016-09-30 update bulletins); the Oracle update that went with Bloomburrow removed "up to" from it and from Catalyst Stone, Flash, Fluctuator, Mana Matrix, Planar Gate and Urza's Filter (taw/magic-search-engine's MTGJSON data: commit e2b33f13, 2024-07-24, still "up to"; the next, 1e5a15c7, 2024-07-29, not; the same update carries the "enters" templating the CR effective 2024-07-30 adopted). The cost rules didn't change (601.2f "minus all cost reductions"; 409.1f of the CR of 2004-10-01 the same); Semblance Anvil (2011-01-01): "Semblance Anvil's cost reduction effect is mandatory." Exempt in `docs/rulings-exemptions/rulings-P033.tsv` (new), logged as outdated; engine unchanged (the reduction always applies; no decision is offered) | 35785c12; docs and comments in the STATUS commit |

**Tests.** `tests/rulings/illegal_anchor_targets.rs` (new):
`fiery_annihilation_exiles_equipment_still_attached_to_an_illegal_creature` (Gods Willing gives the
Bears protection from red in response; the Bonesplitter, still attached, is exiled and the Bears take no
damage; control with protection from blue) and `yosei_taps_the_permanents_of_a_player_who_became_an_illegal_target`
(the player gains hexproof with the trigger on the stack: the permanents are tapped and untap in the
player's next untap step; control without hexproof: the untap step is skipped). Both fail, as do the
Searing Blaze and Suffer the Past tests, when the engine is changed to drop illegal targets from the
context in which later targets are checked (tried and reverted). `interp!` added to
`two_target_burn_still_affects_the_legal_target`, `suffer_the_past_player_illegal_cards_legal` and
`run_away_together_one_target_illegal_uses_lki`; Memory's Journey's and Witness the Future's player
rulings cited in `target_player_shuffles_target_cards_does_nothing_if_the_player_is_illegal`
(`r_p163_graveyard.rs`), which already gave the player hexproof.

**For the user.**
1. **Fiery Annihilation (case 1).** May its ruling "If the target creature is an illegal target when
   Fiery Annihilation begins to resolve, the target Equipment (if any) won't be exiled." (2024-11-08) be
   set aside as inconsistent with Yosei's (2017-11-17) and Run Away Together's (2022-06-10 to
   2025-11-17) rulings, as Gnoll War Band's was? It turns on the end of CR 608.2b: "If part of the
   effect requires information about an illegal target, it fails to determine any such information.
   Any part of the effect that requires that information won’t happen." (whether it reaches 608.2b's
   first step, "it checks whether the targets are still legal", for a target whose requirement names
   the illegal one). Recommendation: yes (an exemption line in `rulings-P047.tsv` citing the decision;
   `dependent-target-checked-against-an-illegal-target` becomes decided). Unlike Gnoll War Band's case,
   the contrary post-2015 ruling on this structure is Yosei's alone; the other way (R2) would make
   Searing Blaze's and Suffer the Past's rulings outdated and Yosei's, Run Away Together's and the
   share-a-type rulings inconsistent, and change the engine (no exile, no damage, no tapping, and
   Glyph of Delusion's anchored target illegal once the Wall has left).
2. **Stone Calendar (case 2)** is exempt as outdated on our own (worker policy); it is in the log for
   review. Wizards' note on the Bloomburrow Oracle changes wasn't read, so the dating rests on
   third-party data (two consecutive commits of an MTGJSON-built index, verified byte for byte).
