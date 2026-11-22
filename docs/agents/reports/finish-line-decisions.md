# Decisions collected for the finish-line plan (Nov 9, 2026)

Read by a subagent of the `finish-line-plan` planner; the plan (docs/agents/finish-line-plan.md §5) presents the open ones. Kept here for the full quotes and the already-decided list.


Sources: docs/INTERPRETATIONS.md and docs/interpretations/*.md, docs/RULING_DISCREPANCIES.md,
docs/agents/items/{ruling-conflicts-1..4, register-decisions-nov5, register-decisions-nov5b,
user-questions-nov7, p005-flagged-bugs, verified-cards, verified-refresh-nov4, finish-line-plan}.md,
docs/agents/verification-plan.md, docs/DEFERRED.md, the last 60 commits, `scripts/rulings_batches.py`.
All quotes are verbatim from `data/comprehensive-rules.txt` (effective 2026-09-25),
`data/rulings.jsonl.gz` and `data/oracle_cards.jsonl.gz` (both 2026-09-24).

## State of the register

128 entries: 85 interpretations, 9 deviations, 34 policies. **Decided 128, provisional 0.**
(`grep -c "Status: provisional" docs/interpretations/*.md` is 0; the last provisional entry,
`exiled-card-power-as-it-last-existed-in-the-graveyard`, was decided by the user on Nov 9,
commit 93be8d8.) Nothing in the register awaits the owner.

## Open conflict cases (rulings the engine neither cites nor exempts)

Exactly one case is awaiting the owner. The four ruling-conflicts items (1–4), register-decisions-nov5
and -nov5b, user-questions-nov7 (Parts A and B) and p005-flagged-bugs have all landed with every
"For the user" question answered (Nov 4, 5, 7 and 9). The last 60 commits (rulings P061–P077, Nov 8)
report one contradiction "left uncited ... pending the conflict decision" (commit 6c28fc9, rulings-P068).
No docs/agents/reports/*.md dates from November (judge-audit-1 is Oct 14, structure-tests is older).

---

## Decisions

### 1. Share the Spoils: may its ruling be set aside as inconsistent with CR 603.10f?

**Question.** When a player controls an opponent's Share the Spoils and that opponent loses the game,
does the "whenever an opponent loses the game" ability trigger (the engine, CR 603.10f) or not (the
ruling)?

**Options.** (a) Recommended: set the ruling aside as inconsistent (an exemption line in the P068
exemption file citing this decision; the RULING_DISCREPANCIES entry stays, status "open for review"
for reporting to Wizards), exactly as Gnoll War Band's (Nov 5) and Arbiter of Knollridge's (Nov 5)
rulings were. The engine already follows the CR and is tested
(`tests/rulings/r_p068_players.rs`, `tests/cr/r600_look_back.rs`). (b) Follow the ruling: change the
engine so the leave-the-game procedure (800.4a) removes the object before the loss is checked for
triggers; contradicts 603.10f and the Sanctum of Serra ruling and would need a register deviation.

**Oracle, Share the Spoils:** "When this enchantment enters and whenever an opponent loses the game,
exile the top card of each player's library. / During each player's turn, that player may play a land
or cast a spell from among cards exiled with this enchantment, and they may spend mana as though it were
mana of any color to cast that spell. When they do, exile the top card of their library."

**Ruling, Share the Spoils (2021-07-23):** "If you control an opponent's Share the Spoils and that
player loses the game, Share the Spoils leaves the game with them and the first ability won't trigger."

**CR 603.10:** "Normally, objects that exist immediately after an event are checked to see if the event
matched any trigger conditions, and continuous effects that exist at that time are used to determine
what the trigger conditions are and what the objects involved in the event look like. However, some
triggered abilities are exceptions to this rule; the game “looks back in time” to determine if those
abilities trigger, using the existence of those abilities and the appearance of objects immediately
prior to the event. The list of exceptions is as follows:"
**CR 603.10f:** "Abilities that trigger when a player loses the game look back in time."
**CR 603.3a:** "A triggered ability is controlled by the player who controlled its source at the time it
triggered, unless it’s a delayed triggered ability. ..."
**CR 800.4a:** "When a player leaves the game, all objects (see rule 109) owned by that player leave the
game and any effects which give that player control of any objects or players end. Then, if that player
controlled any objects on the stack not represented by cards, those objects cease to exist. Then, if
there are any objects still controlled by that player, those objects are exiled. This is not a
state-based action. It happens as soon as the player leaves the game. ..."

**Rulings the other way (same shape: an object's own look-back trigger when 800.4a removes it):**
- Sanctum of Serra (2009-10-01): "If you planeswalk away from Sanctum of Serra because the player who
  owns Sanctum of Serra leaves the game, its first ability will still trigger and resolve."
- Thieving Amalgam (2019-08-23): "If a creature you control but don't own dies at the same time that its
  owner leaves the game, Thieving Amalgam's last ability triggers. No player loses 2 life, but you gain
  2 life."
- Grismold, the Dreadsower (2019-08-23): "In a multiplayer game, if a creature token dies at the same
  time its controller loses the game, Grismold's last ability triggers."
- Pendant of Prosperity (2019-08-23), the standard borrowed-permanent note that the ruling's first clause
  copies: "In a multiplayer game, if Pendant of Prosperity's owner leaves the game, Pendant of Prosperity
  leaves the game with them. If Pendant of Prosperity's controller leaves the game, Pendant of Prosperity
  is exiled (assuming that player wasn't also Pendant of Prosperity's owner)."

**Evidence gathered (rulings-P068, investigator + skeptic):** 603.10f dates from the CR effective
2016-11-11 and was in force on 2021-07-23, so not outdated; the Forgotten Realms release notes
(2021-07-12) have the same words, so not a copying error; it names the card, so not misattached; it
doesn't contradict the Oracle text, so not a template error. "the first ability won't trigger" appears
on no other card.

### 2. The "Not yet investigated" notes: conflict cases or not?

**Question.** docs/RULING_DISCREPANCIES.md "Not yet investigated" says "the coordinator decides whether
they become a conflict case"; the owner may prefer to decide. Three items, all already cited or exempt,
engine unaffected:

(a) **Geier Reach Sanitarium** (2016-07-13): "When you activate Geier Reach Sanitarium's last ability,
first each player draws a card. Then the player whose turn it is selects a card from their hand and sets
it aside without revealing it; proceeding in turn order, each other player does the same. Then the cards
that were set aside are discarded at once." Oracle: "{2}, {T}: Each player draws a card, then discards a
card." The ruling says "When you activate"; the engine draws as the ability resolves (CR 602.2: "To
activate an ability is to put it onto the stack and pay its costs, so that it will eventually resolve and
have its effect."). Stays cited.
(b) **Kami of the Crescent Moon** (2005-06-01): "The additional draw triggers just after the normal draw
for the turn happens." Oracle: "At the beginning of each player's draw step, that player draws an
additional card." By 603.2b it triggers as the step begins and goes on the stack after the turn-based
draw; indistinguishable in play. Stays cited.
(c) **Spatial Contortion, Circle of Flame, Hada Freeblade**: a Scryfall (not Wizards) note, source
"scryfall" (2021-02-01): "Spatial Contortion is banned in Pauper (not just non-legal), due to a special
case on MTGO: https://magic.wizards.com/en/articles/archive/news/january-21-2019-banned-and-restricted-announcement"
against the data's legalities (Pauper `not_legal`). Exempt as tournament policy; a report would go to
Scryfall, not Wizards.
(d) **Dragon-Kami's Egg** (2022-02-18): "hatchling counters" for the card's "hatching counter": Wizards'
typo in the release notes too; stays cited.

**Options.** Recommended: no conflict case for any (none contradicts the CR on anything observable);
record (a) and (b) as "narrower than stated"/wording notes in the log only if the owner wants them
reported to Wizards; otherwise leave as is. Alternative: run (a) and (b) through the investigator/skeptic
process.

### 3. Human review of the discrepancy log (48 entries, all "open for review")

**Question.** Does the owner want to review and report the log to Wizards now (per the Nov 5 decision
"so they can be checked by a human, and reported to Wizards of the Coast if erroneous"), and should
anyone draft the report? Counts by kind: template errors 6 (Homicide Investigator, Rimescale Dragon,
Combat Celebrant, Citanul Druid, Acolyte Hybrid, Sakashima's Protege); copying errors 2 (Run Out of Town,
Sanguine Spy); inconsistent 5 (Gnoll War Band, Araumi of the Dead Tide, Arbiter of Knollridge, Fiery
Annihilation, Share the Spoils); narrower than stated 4; misattached 6; outdated 25. Statuses available:
*open for review*, *reported*, *confirmed*. The finish-line-plan item asks for the ten most consequential
entries; no engine behavior depends on this. Recommended: the plan produces the report; the owner decides
whether and where to send it.

### 4. Loops and loop detection (verification-plan "Later", user Oct 21)

**Question.** How far should loop handling go before "done": (1) keep today's state-repeat detection
(decided entries `mandatory-loop-detection`, `growing-loops-run-to-the-limit`); (2) also recognize
growing loops early (needs recognition by repeated event sequences, not states); (3) automate shortcuts
and proposed loops (CR 732.2), which a UI would need.

**Information the owner needs.** Today the engine fingerprints the game state at each priority and calls
a loop of mandatory actions a draw on the third repeat after at least 8 passes; a fragmented loop (732.3)
is broken by the active player; a loop that grows the state (two Charismatic Conquerors a side) runs to
the action limit (default 200,000 decisions) and is then a draw. The Nov 4 refresh found and the engine
has since fixed a bookkeeping bug that stopped detection after any forced choice (`game_end.rs` now
compares `loop_mark` with `actions_taken - forced_asks`). Option 2 needs a model of "the same sequence
of events"; option 3 needs an agent-protocol extension for proposing/shortening a shortcut.
Recommended: (1) for the finish line; (2) and (3) as a named post-finish item.

**CR 104.4b:** "If a game that’s not using the limited range of influence option (including a two-player
game) somehow enters a “loop” of mandatory actions, repeating a sequence of events with no way to stop,
the game is a draw. Loops that contain an optional action don’t result in a draw."
**CR 104.4f:** "In a multiplayer game using the limited range of influence option, if the game somehow
enters a “loop” of mandatory actions, repeating a sequence of events with no way to stop, the game is a
draw for each player who controls an object that’s involved in that loop, as well as for each player
within the range of influence of any of those players. Only those players leave the game; the game
continues for all other players."
**CR 732.1b:** "Occasionally the game gets into a state in which a set of actions could be repeated
indefinitely (thus creating a “loop”). In that case, the shortcut rules can be used to determine how many
times those actions are repeated without having to actually perform them, and how the loop is broken."
**CR 732.2a (first sentences):** "At any point in the game, the player with priority may suggest a
shortcut by describing a sequence of game choices, for all players, that may be legally taken based on
the current game state and the predictable results of the sequence of choices. This sequence may be a
non-repetitive series of choices, a loop that repeats a specified number of times, multiple loops, or
nested loops, and may even cross multiple turns. It can’t include conditional actions, where the outcome
of a game event determines the next action a player takes. ..."
**CR 732.2b:** "Each other player, in turn order starting after the player who suggested the shortcut,
may either accept the proposed sequence, or shorten it by naming a place where they will make a game
choice that’s different than what’s been proposed. (The player doesn’t need to specify at this time what
the new choice will be.) This place becomes the new ending point of the proposed sequence."
**CR 732.2c:** "Once the last player has either accepted or shortened the shortcut proposal, the shortcut
is taken. The game advances to the last proposed ending point, with all game choices contained in the
shortcut proposal having been taken. If the shortcut was shortened from the original proposal, the player
who now has priority must make a different game choice than what was originally proposed for that player."
**CR 732.3:** "Sometimes a loop can be fragmented, meaning that each player involved in the loop performs
an independent action that results in the same game state being reached multiple times. If that happens,
the active player (or, if the active player is not involved in the loop, the first player in turn order
who is involved) must then make a different game choice so the loop does not continue."
**CR 732.4:** "If a loop contains only mandatory actions, the game is a draw. (See rules 104.4b and 104.4f.)"
**CR 732.5:** "No player can be forced to perform an action that would end a loop other than actions
called for by objects involved in the loop."
**CR 732.6:** "If a loop contains an effect that says “[A] unless [B],” where [A] and [B] are each
actions, no player can be forced to perform [B] to break the loop. If no player chooses to perform [B],
the loop will continue as though [A] were mandatory."

### 5. Raising the verified count (verification-plan "Later", user Oct 21: "leave it as it is for now")

**Question.** Which approach, and is it a finish-line criterion? Today (docs/CARD_COVERAGE.md): verified
18,113 of 32,111 paper cards (56.4%); round-trip-or-better 28,570 (89.0%); compiled 32,076 (99.9%).
11,522 compiled cards fail only/also the structure check; of the 10,457 round-trip-tier cards about
9,100 miss verified by exactly one unexercised structure (3,984 "1 of 1", 3,416 "1 of 2", 1,439 "1 of 3",
286 "1 of 4"). Structure coverage: 13,192 of 28,232 distinct structures exercised (46.7%); no unexercised
structure is shared by 3+ cards (structure-tests report), so each further test verifies about 1–3 cards.

**Options.** (1) Test whole shapes: about 3,000 tests to cover the remaining single-card structures (the
verification plan's estimate; routine, many sessions). (2) Compositional coverage: every AST component and
combination at depth ~1 exercised by some test, then count a card structure-verified when all its
components are (the Oct 21 recommendation; changes the definition of "verified", so it is a policy change
the owner must accept). (3) Leave it; report verified as is. Recommended: (2), with a register policy entry
describing the new criterion, since (1) measures bookkeeping more than correctness. No CR text turns on this.

### 6. New cards after a data refresh (verification-plan "Later", user Nov 4: "No new card data for now")

**Question.** When does the owner refresh `data/` (Scryfall oracle cards, rulings, tags; possibly a newer
CR), and what scope do the new cards get? Information: the current data is 2026-09-24 / CR 2026-09-25;
189 paper cards from sets not yet released at fetch time (75 from `trk` Star Trek and three Commander
products) are today deferred as "not legal in any format" and become in-scope on refresh; a newer CR may
renumber rules, which breaks `cr!` citations until fixed; the every-card sweep (~2.5 h) and the tier file
must be regenerated afterwards. Only the owner updates the data (Nov 4). Recommended: refresh after the
finish-line criteria are met, then a card-support round plus a CR-diff item. No quotes turn on this.

### 7. Compile every hand-written card (verification-plan "Later", user Nov 4)

**Question.** Is this a finish-line criterion or post-finish, and does the owner accept that some
`custom_*` hooks become general compiled constructs? Information: 153 cards have a `ManualAbility`
(`crates/mtg-engine/src/cards/`), all with tests, all passing manual-check and counted as round-trip; they
are the main unaudited code path (finish-line-plan names them a risk). Recommended: plan it as a
card-support round after rulings coverage, as the user said; not a blocker for "done".

### 8. Deferred categories: revisit at the finish line? (user Nov 4: "Deferred categories stay as they are")

**Question.** Confirm the deferrals stand for "done", or pick any to pull in. What each keeps out
(cards by `scripts/rulings_batches.py deferred()`, ruling texts with status DEFERRED now; CARD_COVERAGE
counts 48 sticker sheets separately and the 35 unsupported paper cards below are the only ones):
- **Digital-only cards** (decided 2026-10-01): 855 cards, 4 deferred ruling texts, 939 unsupported texts
  (`cards-digital-alchemy.md`). Three paper cards stay unsupported with the mechanics: Pearl Collector
  "Deathtouch, Lifelink / At the beginning of your second main phase, if you gained 4 or more life this
  turn, conjure a card named Mox Pearl into your hand. This ability triggers only once. / {2}{W}: Another
  target creature perpetually gains lifelink."; Tenacious Pup "When Tenacious Pup enters the battlefield,
  you gain 1 life. You get a one-time boon with "When you cast a creature spell, that creature enters the
  battlefield with an additional +1/+1 counter, trample counter, and vigilance counter on it.""; Sigardian
  Evangel "When Sigardian Evangel enters the battlefield, conjure a card named Sigardian Evangel into your
  hand. Discard that card at the beginning of the next end step. / When Sigardian Evangel enters the
  battlefield, tap target permanent you don't control." (all three legal only in Arena formats).
- **Cards not legal in any format** (2026-10-02): 2,341 cards by the script's rule (DEFERRED.md: about
  2,050 — playtest 737, Un-set 474, planes/phenomena 212, unreleased sets 189, schemes 102, banned
  everywhere 34, vanguards, memorabilia), 1,812 deferred ruling texts: by far the largest block.
  Sub-question: Un-cards (acorn, Unknown Event 500, Unstable 216, Unfinity 125, Unhinged 112, Unsanctioned
  85) — many compile already; pulling them in would mean counting and verifying them.
- **Sticker sheets** (2026-10-01): 49 sheets + 24 cards deferred with them, 30 deferred ruling texts.
  CR 123.2c: "Each player has access to only the stickers on the chosen sheets during the game, and those
  sticker sheets remain revealed." CR 123.3: "If an effect instructs a player to put a sticker on an
  object, that player chooses a sticker that is not currently on any objects they own from among the
  stickers they have access to and puts it on that object."
- **Contraptions, host/augment** (2026-10-01): out of scope, 662 ruling texts; not legal anywhere.
- **Dexterity cards** (2026-10-21): 2 cards, 2 deferred texts. Chaos Orb: "{1}, {T}: If this artifact is
  on the battlefield, flip it onto the battlefield from a height of at least one foot. If this artifact
  turns over completely at least once during the flip, destroy all nontoken permanents it touches. Then
  destroy this artifact." Falling Star: "Flip Falling Star onto the playing area from a height of at least
  one foot. Falling Star deals 3 damage to each creature it lands on. Tap all creatures dealt damage by
  Falling Star. If Falling Star doesn't turn completely over at least once during the flip, it has no
  effect." (legal only in `oldschool`).
- **Conspiracy face-down draft cards** (user, 2026-10-28): 5 cards, 16 deferred texts. CR 905.2: "Some
  cards have abilities that function during the draft." Illusionary Informant: "Draft this card face up.
  / During the draft, you may turn this card face down. If you do, look at the next card drafted by a
  player of your choice. / Flying".
Recommended: keep every deferral (the user reaffirmed them Nov 4); the only one worth a second look at
the finish line is the 189 unreleased-set cards, which the data refresh (decision 6) resolves by itself.

### 9. Verified tier and strict mode: definition stands?

**Question.** `verified-cards.md` left no policy open; the only owner-facing choice is whether "verified"
stays the headline and default (`mtg-sim --cards verified`, `ProtocolOptions::min_card_tier`) at 56.4%
for "done", or whether a done criterion is set on it (ties to decision 5). The item's remaining note
("the recipe still says three chunks ... Left for the user to update (CLAUDE.md)") is done: CLAUDE.md
now has four chunks. Recommended: keep the definition; set the target via decision 5.

---

## Already decided — don't re-ask

- **All 128 register entries** (Oct 21, Nov 4, Nov 5 morning and evening, Nov 7, Nov 9), including:
  `combat-tax-of-zero-is-a-cost`, `attacking-player-with-shared-team-turns` (805.10c), `growing-loops-run-to-the-limit`,
  `conspiracy-draft-needs-three-players`, `range-combat-trigger-events`, `illegal-action-reversal-restores-libraries`
  (plus the 733.1 fix, `reversal-library-exceptions`), `end-of-combat-on-your-next-turn-without-combat`,
  `trample-reassignment-counts-only-assigned-damage`, `copy-values-fixed-when-determined`, `venture-limit-looks-at-whole-turn`,
  `changeling-cda-only`, the 25 accepted Nov 4, the three fixed (`register-fixes-1`: library-arranged-by-instructing-player,
  linked-reveal-trigger-as-reflexive, Nihiloor), `qualified-landwalk-is-that-landwalk`, `trigger-x-before-targets`,
  `dependent-target-checked-against-an-illegal-target` (Nov 7: Fiery Annihilation's ruling set aside; no CR
  distinction between attachment and control), `player-named-twice-draws-twice` (Nov 7: one instruction, twice as many),
  `exiled-card-power-as-it-last-existed-in-the-graveyard` (Nov 9).
- **Conflict cases decided Nov 5:** Homicide Investigator, Rimescale Dragon, Combat Celebrant, Citanul Druid
  (template errors give way to Oracle text, policy `ruling-contradicting-oracle-gives-way`); Gnoll War Band,
  Arbiter of Knollridge, Araumi of the Dead Tide (set aside as inconsistent); Run Out of Town, Sanguine Spy
  (copying errors); misattached-rulings policy; Caller of the Untamed (engine follows 601.2b); Lich, Corruption of
  Towashi, the MDFC "can't transform" family, Stone Calendar and the other outdated entries; Grip of Chaos
  (a random reselection of the same target doesn't retrigger ward/"becomes the target"); Krosan Adaptation,
  Jade Statue, Deranged Assistant, Mossfire Egg, Oculus Whelp (superseded by the CR, confirmed in ruling-conflicts-1).
- **Policies:** the CR and rulings should not contradict — every apparent contradiction is investigated
  (investigator + skeptic, effort max) before any exemption; every erroneous ruling is logged in
  docs/RULING_DISCREPANCIES.md for human review and reporting to Wizards.
- **Scope:** deferred categories stay (Nov 4); digital-only and digital versions out, every paper card counted
  (Oct 28, `paper-scope-1`); dexterity cards deferred (Oct 21); Conspiracy draft cards deferred (Oct 28).
- **Process:** no new card data until the owner refreshes it (Nov 4); implementers at effort high, reviewers,
  investigations and register calls at max; fresh sessions past ~300k tokens; no A/B measurement; not chosen:
  C (Forge/XMage comparison), G (mutation gate), H (compiled-form snapshots), I (set intake), J (long-tail cards
  as data), K (play-frequency priority); the verified count stays as is until "later" (Oct 21).
- **Standard update format and "present decisions with the texts quoted"** (Nov 5).
- **p005-flagged-bugs** (castability under continuous effects, default X with X-dependent costs/targets,
  graveyard casts targeting the card itself): landed; the "Flagged by rulings-P005" section is gone from
  engine-gaps.md.
