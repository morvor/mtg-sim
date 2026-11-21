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

## STATUS

Done (Nov 12). **Conclusion: kind inconsistent.** In the conflict items' A to D scheme it is D only in that the user decides: the CR's words and every other ruling point one way, the ruling alone the other. An independent investigator and a separate adversarial skeptic (the reviewer) reported. The skeptic re-checked every quote against `data/` (snapshot 2026-09-24, CR effective 2026-09-25), the archived Comprehensive Rules (https://github.com/pit142857/mtg-cr, commit 8351b94f; 406.3 extracted by script from all 130 files, dated by each file's own effective date) and ormos (commit bd11dee2), tried to refute the conclusion, and didn't. No engine or test change. The ruling stays OPEN, uncited and unexempted. It is logged in `docs/RULING_DISCREPANCIES.md` (Inconsistent rulings, "Headliner Scarlett: "no one can look at it" once the turn ends", open for review), as Fiery Annihilation's was before the user decided (ruling-conflicts-4, review 6f55d1e9).

**Finding.** The ruling's look half ("you can't look at"; "no one can look at it") contradicts CR 406.3's continuing look and every other ruling on the question. The continuing look is in every CR since the one effective 2013-02-01 and read word for word as today when the ruling was published. The ruling's play half agrees with the CR (514.2, 611.2a) and the engine.
- **Not reconciled:** see "Readings tried" below.
- **Not outdated:** 406.3 hasn't changed since the CR effective 2023-06-16. The same day's rulings speak of "the effect allowing you to look at and play the exiled card", so the look permission was in the text they were written for.
- **Not a template error or copying error:** in the log both kinds need a ruling whose words contradict the card's own Oracle text, and this one contradicts the CR. Nor is it a copy of another card's ruling: no other ruling has its words.
- **Not narrower than stated:** it is wrong in the very situation it was written for.
- **No worker policy applies:** `ruling-contradicting-oracle-gives-way` covers Oracle-text contradictions only, and it leaves rulings that contradict other rulings to case-by-case decisions. So nothing was exempted.

**Sources** (verbatim; from `data/` unless stated).
- **Headliner Scarlett** (Ravnica: Clue Edition, released 2024-02-23), Oracle text: "Haste" / "When Headliner Scarlett enters, creatures target player controls can't block this turn." / "At the beginning of your upkeep, exile the top card of your library face down. You may look at and play that card this turn."
- **Its rulings, all dated 2024-02-02:**
  - "You pay all costs and follow all normal timing rules for a card played this way. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty." (CITED, P144)
  - "If Headliner Scarlett leaves the battlefield, the effect allowing you to look at and play the exiled card continues to apply for the rest of the turn." (CITED, `headliner_scarlett_permission_lasts_the_turn_even_if_she_leaves`)
  - "Once the turn ends, you can't look at or play the card exiled with Headliner Scarlett that turn. It remains face down in exile, and no one can look at it." (OPEN)
- **The current CR (effective 2026-09-25):**
  - 406.3: "Exiled cards are, by default, kept face up and may be examined by any player at any time. Cards “exiled face down” can’t be examined by any player except when instructions allow it. However, if a player is instructed to look at a card and then exile it face down, or once a player is allowed to look at a card exiled face down, that player may continue to look at that card until it leaves the exile zone or is part of a pile of cards that are shuffled, even if the instruction allowing the player to do so no longer applies."
  - 406.3b: "Some spells and abilities allow a player to cast spells with certain qualities from among face-down cards in exile. A player may cast such a spell only if they are allowed to look at the face-down card in exile and if the resulting spell has the specified qualities."
  - 514.2: "Second, the following actions happen simultaneously: all damage marked on permanents (including phased-out permanents) is removed and all “until end of turn” and “this turn” effects end. This turn-based action doesn’t use the stack."
  - 708.5: "At any time, you may look at a face-down spell you control on the stack or a face-down permanent you control (even if it’s phased out). You can’t look at face-down cards in any other zone or face-down spells or permanents controlled by another player."
  - 101.1: "Whenever a card’s text directly contradicts these rules, the card takes precedence. ..."
- **406.3's history** (archived CRs):
  - **Effective 2012-10-01, the last CR without it:** "406.3. Exiled cards are, by default, kept face up and may be examined by any player at any time. Cards "exiled face down" can't be examined by any player except when instructions allow it. A card exiled face down has no characteristics, but the spell or ability that exiled it may allow it to be cast from exile. Unless that card is being cast face down (see rule 707.4), the card is turned face up just before the player announces that he or she is casting the spell (see rule 601.2)."
  - **Effective 2013-02-01 (Gatecrash), the sentence added:** "However, once a player is allowed to look at a card exiled face down, that player may continue to look at that card as long as it remains exiled, even if the instruction allowing the player to do so no longer applies."
    - It is unchanged in substance through the CR effective 2021-11-19 (curly quotes in 2014, "they" from 2018-04-27; 406.3a split off and 406.3b added from 2021-02-05).
    - The Gatecrash Update Bulletin (Matt Tabak, 2013-01-28; ormos, Wayback capture 2017-12-25) says of 406.3 only: "This rule covers how you cast a face-down card in exile, but really it should say play, as cards like Praetor's Grasp can let you play a face down, exiled land card."
  - **Effective 2022-02-18 (Kamigawa: Neon Dynasty), the first clause added:** "However, if a player is instructed to look at a card and then exile it face down, or once a player is allowed to look at a card exiled face down, that player may continue to look at that card as long as it remains exiled, even if the instruction allowing the player to do so no longer applies."
    - Jess Dunks, Comprehensive Rules Changes (2022-02-09; ormos, Wayback capture 2022-02-09): "This is the rule that says that once a player has been allowed to look at cards in exile, that player may continue to look at those cards. As written, the rule only applied to cards that were exiled already when you were given permission to look at them. It didn't apply to situations where you look at a card *and then* exile it face down. ..."
  - **Effective 2023-06-16, today's text:** "... until it leaves the exile zone or is part of a pile of cards that are shuffled, ...".
    - It is the same in the CRs effective 2023-11-17 and 2024-02-02, current when the ruling was compiled and published.
    - It is also the same in every later archived CR through the one effective 2026-04-17, and in the repo's.
    - This change's explanatory note wasn't found.
- **Rulings that agree with 406.3:** a player once allowed to look keeps looking after the permission stops applying to them.
  - Bane Alley Broker (2013-01-24): "Once you are allowed to look at a face-down card in exile, you are allowed to look at that card as long as it’s exiled. If you no longer control Bane Alley Broker when its last ability resolves, you can continue to look at the relevant cards in exile to choose one to return."
  - Colfenor's Plans ("You may look at the cards exiled with this enchantment, and you may play lands and cast spells from among those cards."), 2013-04-15: "If Colfenor's Plans leaves the battlefield, you can continue to look at cards exiled by it, but you can no longer play them." This is the closest analogue: a look-and-play permission ends, and the look stays while the play doesn't.
  - Grimoire Thief (2013-07-01): "Once a player is allowed to look at the exiled face-down cards, that player may continue to do so even after Grimoire Thief leaves the battlefield or another player gains control of it."
  - Kheru Mind-Eater (2017-08-25): "If another player gains control of Kheru Mind-Eater, that player can see all of the exiled cards and may play them. Once a player looks at a face-down exiled card, that player may look at that card any time later in the game."
  - Ugin, the Ineffable ("+1: Exile the top card of your library face down and look at it. ..."), 2019-05-03: "Once you look at the exiled face-down card once, you may look at it again any time you wish. If another player gains control of the Spirit token, that player can't look at the exiled card."
  - The Dragon-Kami Reborn (2022-02-18; the same words are in the Kamigawa: Neon Dynasty Release Notes, ormos): "You may look at the cards you exile with The Dragon-Kami Reborn's first two chapter abilities, and you may continue to do so after it has left the battlefield."
  - The hideaway note, on Howltooth Hollow and 11 other cards (2022-04-29), Collector's Cage (2024-04-12) and Evercoat Ursine (2024-07-26): "Any player who has controlled a permanent with a hideaway ability since a card was exiled with it may look at that card." Yet CR 702.75a, from the CR effective 2022-04-29 (before it: "Any player who has controlled ..."), grants the look only to "The player who controls the permanent that exiled this card".
  - Hoarding Broodlord (2023-04-14): "As long as that card remains exiled, you may look at it. This remains true after Hoarding Broodlord leaves the battlefield."
  - Look-then-exile cards (406.3's first clause):
    - Kayla's Music Box (its reminder text "(You may look at it any time.)"; "{T}: Until end of turn, you may play cards you own exiled with Kayla's Music Box."), 2022-10-14: "No other player may look at the face-down cards you own exiled with Kayla’s Music Box, even if another player takes control of it."
    - Lobelia, Defender of Bag End (2023-06-16): "You may look at these face-down exiled cards any time you wish. No other player may look at the face-down cards you exiled with Lobelia, Defender of Bag End, even if another player takes control of it."
- **Rulings that deny a look (none agrees with Scarlett's):**
  - Players never allowed to look:
    - Connecting the Dots, from the same release (its card: "(You can't look at it.)"), 2024-02-02: "... (You still can't look at them.) ...".
    - The processors' note (2015-08-25): "You can’t look at face-down cards in exile unless an effect allows you to."
    - Praetor's Grasp (2011-06-01) and Necropotence (2023-09-01).
  - A new controller: Ugin above, Clone Shell (2020-08-07), Ixhel, Scion of Atraxa (2023-02-04) and others.
  - A player who has left the game: Gonti, Lord of Luxury (2016-09-20): "If you leave the game, the card remains exiled face down indefinitely. No player may look at it." It recurs on 11 more cards in similar words through 2024-11-08, among them Outrageous Robbery, from the same release (2024-02-02): "If you leave the game, the exiled cards remain exiled face down indefinitely. No player may look at them."
- **Lumbering Laundry** ("{2}: Until end of turn, you may look at face-down creatures you don't control any time."), from the same release, 2024-02-02: "Once it has resolved, Lumbering Laundry's first ability lets you look at face-down creatures whenever you want until end of turn, even if you don't have priority. This action doesn't use the stack. Knowing what those cards are becomes part of the information you have access to, just like you can look at the cards in your hand. Once the turn ends, however, you can no longer look at those creatures."
  - It is right for face-down permanents: 708.5 gives them no continuing look.
  - It is the likely origin of Scarlett's wording, together with the leave-the-game note (an inference).
- **Kaya the Inexorable** (its emblem: "At the beginning of your upkeep, you may cast a legendary spell from your hand, from your graveyard, or from among cards you own in exile without paying its mana cost."), 2021-02-05: "If you own a face-down card in exile, Kaya’s emblem will let you cast it only if you can look at the card (and only if it’s legendary). If you aren’t allowed to look at the card, you can’t cast it, even if you know it’s legendary."
  - So the difference matters in play. Under 406.3 the emblem can cast a legendary card Scarlett exiled on a later upkeep; under the ruling it can't.

**Survey.** Scarlett is the only card whose Oracle text grants a time-limited look at a face-down exiled card (whole Oracle texts searched). On Kayla's Music Box, Lobelia, Urianger Augurelt, Apple of Eden and Three Wishes, the duration limits the play permission instead. Lumbering Laundry's and Revealing Wind's looks are at face-down creatures. So the item's template comparison has no other card. No ruling found takes the look from a player once allowed it; the searches covered "can't look", "no one/no player can/may look", "no longer look", "can no longer", "won't be able to look", "not allowed to look" and "Once the turn ends".

**Readings tried (none reconciles).**
- **"No one" meaning opponents only:** the first sentence already denies "you".
- **"You may look at and play that card this turn" not being an allowance under 406.3:** it is one, given after the card was exiled face down. That is the case the rule has covered since 2013 (the 2022 article: "cards that were exiled already when you were given permission").
- **The card's "this turn" overriding 406.3 (101.1):** a duration says nothing against looking later, and it isn't a "can't" (101.2). Read that way, every lapsed permission would override 406.3, and its "even if the instruction ... no longer applies" would have no case left.
- **The 2022-02-18 and 2023-06-16 changes:** one adds the look-then-exile case, the other an end point (a shuffled pile). Neither concerns a permission that ends.

**Engine** (checked; no change).
- **Compilation:** "You may look at and play that card this turn" compiles to two parts (`oracle/patterns/permission_grammar.rs`):
  - `MAY_LOOK_AT_EXILED`, via `zones::allow_look`: the look lasts until the card leaves exile, and `piles.rs` drops it when the card's face-down pile is shuffled;
  - a play permission until end of turn.
- **Reading the look:** `facedown::can_look_at` reads it, and mtg-api's observations go through it (`visibility::can_see` calls `player_control::can_see`).
- **The pinning test:** in `tests/cards/permission_scopes.rs`, `headliner_scarlett_card_cant_be_played_after_that_turn` (`cr!("611.2a", "406.3")`) checks P0's next turn: the card is still exiled face down and can't be cast, P0 can look at it, and P1 can't.
  - The test would fail if the look ended with the turn, because owners have no blanket look: `tests/rulings/r_p068_players.rs` `kaya_emblem_casts_a_face_down_exiled_card_only_if_you_may_look_at_it` asserts that P0 can't look at their own card exiled by P1's Gonti.
- **Test runs:** the three Scarlett tests and the two Kaya emblem tests pass (debug build).
- **Kaya:**
  - `mtg-tools unsupported --card` (debug build) reports no unsupported text for Kaya the Inexorable or Headliner Scarlett, and `docs/verified-cards.tsv` has both at "compiled".
  - So a test of the emblem casting a legendary card Scarlett exiled on a later upkeep is possible: the emblem's cast is gated on the same `allow_look` right. Not added, since this is a docs-only item.

**Verified and not.**
- **Verified:** every quote above, from `data/`, the archived CRs (by script), the two ormos articles and the Kamigawa: Neon Dynasty Release Notes.
- **Not read: the Murders at Karlov Manor Release Notes.** According to the investigator's search results, they carry the Ravnica: Clue Edition notes and give the ruling in the same words; the skeptic's own search found nothing.
  - Why: magic.wizards.com and media.wizards.com are outside this session's allowed network hosts (WebFetch: not resolvable; a direct download was refused by the session's permission check), and ormos ends in 2022.
  - So a copying error isn't ruled out word for word, though that kind would also need a ruling that contradicts the card's Oracle text.
- **Not re-checked:** the investigator's reports of XMage's implementation (it keeps the look) and Forge's (it ends the look with the turn).

**Skeptic's corrections.** Nothing in the investigator's conclusion was refuted. Corrected or confirmed:
1. **The item's "present in substance since the CR of 2016-01-16"** (from the P144 worker, repeated in its yield row): the continuing look is in every CR from the one effective 2013-02-01. This confirms the investigator's correction.
2. **The leave-the-game note:** it is on 11 more cards in similar words, not "the same words". Thief of Sanity's and Mindleecher's are longer notes.
3. **Template error:** the log's kind needs both a contradiction with the card's own Oracle text and a copied ruling, and this ruling has neither. Its likely origin, Lumbering Laundry's same-release note (right for face-down permanents), is a reused phrase, not a copied ruling.
4. **Clive, Ifrit's Dominant's copy of the hideaway note** is misattached (logged under Misattached rulings), so it isn't counted among the agreeing rulings.
5. **A code comment:** one in `oracle/patterns/permission_grammar.rs` cites "CR 406.3b" for this continuing look; the rule is 406.3. It is cosmetic and left for an engine item.

**For the user.** May the look half of Headliner Scarlett's ruling ("you can't look at" and "no one can look at it"; 2024-02-02) be set aside as inconsistent with CR 406.3 and the rulings above, as Arbiter of Knollridge's (Nov 5) and Fiery Annihilation's (Nov 7) were?

Recommendation: **yes**.
- Exempt that part in `docs/rulings-exemptions/rulings-P144.tsv`, with the decision's date.
- Cite the rest (the play half) in `headliner_scarlett_card_cant_be_played_after_that_turn`.
- Update the log entry's Recorded line.
- Optionally, add the Kaya emblem test.

The engine already follows 406.3, so nothing else changes. Following the ruling instead would end the look with the turn for this one card. 406.3 draws no such line ("even if the instruction allowing the player to do so no longer applies"), and doing so would set the engine against Colfenor's Plans' ruling and the others.
