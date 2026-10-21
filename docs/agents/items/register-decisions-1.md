# register-decisions-1: apply the user's decisions on the four provisional entries of the interpretation register

WHY: on Oct 21 the user reviewed the four provisional entries in docs/INTERPRETATIONS.md and decided each of them. Two decisions change the engine so that it follows the Comprehensive Rules exactly; the other two confirm what the engine already does. Every changed entry should record "Decided by the user, Oct 21".

1. **`combat-tax-scaled-to-nothing`: follow CR 118.5.** An applicable tax to attack or block is a cost even when it comes to {0}. That covers a scaled tax whose multiplier is 0 (Collective Restraint when its controller has no basic land types), as well as a written {0} or a chosen X = 0 (War Tax). So:
   - the creature is one that "can't attack unless a player pays a cost" (508.1d), so requirements such as "attacks each combat if able" don't force it (and the same for blocking, 509.1c);
   - a player who attacks or blocks with it acknowledges paying {0} (118.5, 508.1h, 509.1d).

   Implementation:
   - Remove the written-{0} distinction (`combat::is_written_zero` and the code that relies on it, combat.rs around lines 960, 1619 and 1648), so that every applicable tax counts as a cost.
   - Flip this entry's test to the CR behavior, and check every card the change affects, along with its tests and rulings.
   - Rewrite the entry as a decided interpretation. Merge it with `combat-tax-written-zero-is-a-cost` if the two now say the same thing, keeping every id that tests cite valid (or updating those tests).
2. **`attacking-player-is-an-active-player`: implement CR 805.10c fully.** In games with shared team turns, "the attacking player" means one specific attacking player.
   - In an ability of a blocking creature, or in a spell or ability that refers to both a blocking creature and an attacking player, it is the player who controls the attacking creature that blocking creature is blocking, unless the text says otherwise.
   - When a spell or ability could apply to several blocking creatures, the attacking player is determined separately for each blocker.
   - When several attacking players could be chosen, the controller of the spell or ability chooses one (also 805.9).
   - Games without shared team turns are unchanged (506.2).

   Tests in a Two-Headed Giant game, citing 805.10c:
   - Souls of the Faultless;
   - a blocker that blocks attackers of both opponents;
   - at least one more card from the entry's list (Contested Game Ball, Norn's Decree, Invasion Plans, Karazikar, the Eye Tyrant).

   Rewrite the entry as decided, no longer a deviation.
3. **`growing-loops-run-to-the-limit`: decided as it is.** An unendable loop ends the game in a draw (104.4b), and reaching that draw at the action limit is accepted for now. Mark it decided, and note in the entry that loops and loop detection will be revisited once the rules work is settled. That means recognizing such loops early, and automating loops and shortcuts, which a future UI will need. The action item is in docs/agents/verification-plan.md under "Later".
4. **`conspiracy-draft-needs-three-players`: decided as it is.** Keep rejecting two-player Conspiracy Draft games, consistent with 905.3 and 100.1b. Mark it decided.

Then regenerate docs/INTERPRETATIONS.md (`mtg-tools interpretations --write`).

DONE when none of the four entries is provisional, the engine follows 118.5 and 805.10c with tests citing them, and the register index is regenerated. Report what changed for which cards.
