# Dice, coins, Attractions and the draft

Judgment calls about rolling dice and flipping coins (CR 705, 706), rolling to visit
Attractions (CR 701.52, 717) and abilities that function during a draft (CR 905.2).
Format and kinds: `docs/agents/items/interpretation-register.md`; index:
`docs/INTERPRETATIONS.md`.

## dice-highest-result-ties: An opponent tied for the highest result "had the highest result"
- Kind: interpretation
- Status: decided
- Rules: 706.2 "After the roll, the number indicated on the top face of the die before any modifiers is the natural result. ... After considering all applicable modifiers, the final number is the result of the die roll." / Chaos Dragon ruling: "You roll a d20 as well as your opponents. If you had a higher result than all of your opponents, Chaos Dragon won't have any restrictions on who it may attack."
- Question: "Each player rolls a d20. If one or more opponents had the highest result, ~ can't attack those players ..." (Chaos Dragon): when several players tie for the greatest result, which of them "had the highest result" — and does a tie with you count?
- Decision: Every player whose result no other player's result exceeds had the highest result. An opponent tied with you (or with another opponent) had it, so the creature can't attack that opponent or planeswalkers they control this combat.
- Alternatives: Only a unique greatest result is "the highest result": a tie would leave no restriction, contradicting the ruling, which frees the creature only if your result was higher than all of your opponents'.
- Why: The ruling: only a result higher than every opponent's lifts the restriction.
- Affects: Chaos Dragon.
- Code: `crates/mtg-engine/src/dice.rs` (`HAD_HIGHEST`); `crates/mtg-engine/src/oracle/patterns/dice_coins_5.rs` (`opponents_had_highest`).
- Tests: `tests/cards/c5_dice.rs` (`chaos_dragon_tie_with_you_still_counts_as_that_opponent_having_the_highest_result`).

## next-roll-extra-die-same-roll: Several "the next time ... would roll" extra-die effects all apply to that one roll
- Kind: interpretation
- Status: provisional
- Rules: 614.1a "Effects that use the word “instead” are replacement effects. ..." / 616.1 "If two or more replacement and/or prevention effects are attempting to modify the way an event affects an object or player, the affected object’s controller (or its owner if it has no controller) or the affected player chooses one to apply ..." / 706.6 "If a player is instructed to ignore a roll, that roll is considered to have never happened. ..." / Bamboozling Beeble ruling: "You choose which roll to ignore after seeing all the results."
- Question: Two Bamboozling Beeble effects wait for the same player's next roll ("The next time target player would roll one or more dice this turn, instead they roll that many dice plus one and you choose one of those rolls to ignore"). Does the next roll use up both, and in what order are the rolls ignored? And with a static "instead roll that many dice plus one and ignore the lowest roll" (an extra die the roller ignores)?
- Decision: Each such effect applies once to the player's next roll instruction (or planar die roll) this turn, so all pending ones are used up by it: the player rolls one more die for each. The static extra-die effects' lowest rolls are ignored first; then each Beeble effect's controller, in the order the effects were created, chooses one of the remaining rolls to ignore, seeing all the results.
- Alternatives: Applying the effects one at a time in an order the rolling player chooses (CR 616.1) would let them choose which player ignores first; the dice rolled are the same, only who sees which rolls first differs. Ignoring the Beeble rolls first would let the static's "lowest roll" see fewer dice.
- Why: Each effect replaces the same event ("the next time ... would roll"); the order matters only for who picks among which dice, and the rolling player rarely benefits from the choice. Provisional until a ruling covers stacked effects.
- Affects: Bamboozling Beeble (with itself, Pixie Guide-style extra dice).
- Code: `crates/mtg-engine/src/dice.rs` (`NEXT_ROLL_EXTRA`, `take_next_roll_extras`, `roll_dice`, `roll_planar_natural`).
- Tests: `tests/cards/c5_dice.rs` (`two_bamboozling_beebles_both_apply_to_the_next_roll`).

## linked-reveal-trigger-as-reflexive: Priority Boarding's "whenever you reveal a card ... this way" is checked as the first ability reveals
- Kind: deviation
- Status: provisional
- Rules: 603.2 "Whenever a game event or game state matches a triggered ability’s trigger event, that ability automatically triggers. ..." / 603.3 "Once an ability has triggered, its controller puts it on the stack as an object that’s not a card the next time a player would receive priority. ..." / 603.12 "A resolving spell or ability may allow or instruct a player to take an action and create a triggered ability that triggers “when [a player] [does or doesn’t]” take that action ..."
- Question: "Whenever you roll a die, you may reveal the top card of your library. Do this only once each turn. Whenever you reveal a card with mana value less than the result this way, you may exile it. If you do, you may play it this turn." The second triggered ability triggers on the first one's reveal, with "the result" the first ability's roll. How is it modeled?
- Decision: The engine compiles the second ability into the first: after the reveal, if Priority Boarding is still on the battlefield and the revealed card's mana value is less than the result of the roll that triggered the first ability, a triggered ability triggers then (like a reflexive one, CR 603.12) and does the rest, chosen and resolved as usual.
- Alternatives: A separate triggered ability of the permanent, triggered by a reveal event tagged with the first ability: the same results, except if Priority Boarding loses its abilities, or another effect copies the first ability, between the trigger and the reveal (the second ability then wouldn't trigger, or would trigger for the copy).
- Why: The two abilities are linked (only this way's reveals count) and the second needs the first's result; the reflexive form keeps both in one place. The cases where they differ need an effect removing an enchantment's abilities mid-resolution.
- Affects: Priority Boarding.
- Code: `crates/mtg-engine/src/oracle/patterns/attractions_5.rs` (`reveal_then_whenever_you_reveal`).
- Tests: `tests/cards/c5_attractions.rs` (`priority_boarding_does_nothing_more_if_it_left_the_battlefield_before_the_reveal`).

## canal-dredger-default-recipient: Without a choice, a booster pack's last card goes to the next Canal Dredger drafter in the passing direction
- Kind: policy
- Status: decided
- Rules: 905.2a "During a draft, there is no active player or system of priority. If multiple players wish to take an action at the same time during the draft and can’t agree on an order, those actions are taken in a random order." / Canal Dredger ruling: "If multiple players have drafted a card named Canal Dredger, each player chooses which of those players to pass the last card to."
- Question: Each player chooses which Canal Dredger drafter gets the last card of a booster pack they pass. What happens when the player makes no choice (`Draft::choose_last_card_recipient`)?
- Decision: The card goes to the first player who drafted a card named Canal Dredger, counting from the passing player in the round's passing direction; a player who drafted one passes it to themselves (the card's ruling).
- Alternatives: A random drafter, or the passing player's usual neighbor if they drafted one: both are allowed choices; the nearest drafter in the direction the packs go is the closest to an ordinary pass.
- Why: A deterministic default is needed for drafts run without the choice.
- Affects: Canal Dredger.
- Code: `crates/mtg-engine/src/draft.rs` (`Draft::last_card_recipient`).
- Tests: `tests/cards/c5_game_start_draft.rs` (`canal_dredger_default_recipient_is_the_next_drafter_in_the_passing_direction`).
