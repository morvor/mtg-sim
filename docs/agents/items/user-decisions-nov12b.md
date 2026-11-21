# user-decisions-nov12b: per-player targets, Torpor Orb's family, reflexive token triggers, ordering replacements on entering

Run the usual implementer and reviewer flow (worker.md). The reviewer runs at max.

The user answered on Nov 12: "I agree with all recommendations". Record each answer as "Decided by the user, Nov 12".

## Part A: register entries decided as they stand (docs only)

Mark these decided as recommended, then regenerate docs/INTERPRETATIONS.md:
- `per-player-required-targets` (docs/interpretations/stack-casting-costs.md; ruling-conflicts-7): a required per-player target with no legal choice for some player means the spell can't be cast and a triggered ability is removed from the stack.
- `reflexive-trigger-per-specified-token` (triggers-durations.md): "Create N [tokens]. When you do" triggers once per N tokens created as specified (Doubling Season on "create two" triggers it twice).
- `entering-would-be-controller-chooses` (replacement-prevention.md): the would-be controller orders the replacement effects on an entering permanent, including for a card with no controller in its previous zone (Reanimate on an opponent's card), as in the Gather Specimens ruling.

## Part B: Torpor Orb, Panharmonicon, Elesh Norn and Yarok follow the Strict Proctor reading

**The user agreed:** like Strict Proctor (user-decisions-nov12, part B), an ability triggered by a card *leaving a graveyard* (or another zone) as it enters isn't caused by "a permanent entering". So:
- Torpor Orb ("Creatures entering don't cause abilities to trigger.") doesn't stop Desecrated Tomb's ability on a reanimation.
- Panharmonicon, Yarok, the Desecrated and Elesh Norn, Mother of Machines don't make it trigger an additional time.
- Elesh Norn's "Permanents entering don't cause abilities of permanents your opponents control to trigger" doesn't stop it either.

Hushbringer is unchanged: "dying" includes leaving the battlefield, and its rulings (2019-10-04) read it so.

**Task.**
1. Use the same matched-event test as Strict Proctor (`matched_zone_left`, `tag_triggering_event` in triggers.rs) in `kw::torpor` and `Game::additional_triggers`.
2. Add tests for each of the four cards with Desecrated Tomb on a reanimation.
3. Keep their existing tests for enters abilities passing.
4. Update `ability-triggered-cause-is-the-event` to name all five cards. Remove the open question from finish-line-plan §5 ("Torpor Orb and Panharmonicon on a reanimation"), since it's now decided.

## Part C: the Vorinclex counter-ordering deviation stays, to be fixed later

The user accepted "fix it later". Leave `entering-counter-modifiers-after-enters-with` as a registered deviation, add "(to be fixed later; the user, Nov 12)" to it, and add it to the "Revisit later" list in finish-line-plan §5.

Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.
