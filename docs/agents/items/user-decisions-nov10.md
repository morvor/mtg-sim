# user-decisions-nov10: the user's answers to finish-line-plan §5 and the round's provisional entries

The user answered on Nov 10. Parts A and B change the engine. Part C is bookkeeping that the coordinator has already done (listed so the implementer doesn't redo it). Run the usual implementer and reviewer flow (worker.md), with the reviewer at max.

## Part A: Share the Spoils follows its ruling (D1)

**The user (Nov 10):** "D1 doesn't seem inconsistent - Thieving Amalgam and Grismold the Dreadsower are permanents YOU own, trigger from something happening to a creature your opponent owns as they leave the game. Whereas with Share the Spoils, the permanent WITH the trigger is the one that is leaving the game. We should take the ruling here."

**Ruling, Share the Spoils (2021-07-23):** "If you control an opponent's Share the Spoils and that player loses the game, Share the Spoils leaves the game with them and the first ability won't trigger."

**Task.**
1. Make the ability not trigger in that case: a look-back "loses the game" trigger (CR 603.10f) whose source leaves the game with the losing player (CR 800.4a) doesn't trigger on that player's loss.
   - Keep it as narrow as the user's distinction. It applies to the object that carries the trigger and leaves with the loser. It doesn't touch abilities of objects that stay in the game but watch something the loser owns (Thieving Amalgam, Grismold, the Gods: keep their tests passing).
   - Find the case through the look-back check, not by card name.
2. Register it in docs/interpretations/triggers-durations.md as decided ("Decided by the user, Nov 10"), kind interpretation:
   - give the user's reasoning;
   - quote CR 603.10, 603.10f and 800.4a and the ruling;
   - note Sanctum of Serra (2009-10-01) as the nearest ruling the other way. It's a plane that leaves with its owner and still triggers, but on a different event (planeswalking away), and it predates 603.10f (2016-11-11).
   - Cite the entry with `interp!` in the tests.
3. Flip `share_the_spoils_borrowed_from_a_player_who_loses_still_triggers` (tests/rulings/r_p068_players.rs) and the matching case in `tests/cr/r600_look_back.rs` to the ruling's outcome, and cite the ruling with `ruling!`.
4. Remove the Share the Spoils entry from docs/RULING_DISCREPANCIES.md. The user judged it consistent, so it's no longer a discrepancy. Run `cargo test -p mtg-data`.

## Part B: Stinging Study, the player chooses the commander

**The user (Nov 10):** "yes, the player should be able to pick which commander"

Register entry `a-commander-you-own-is-the-greatest` (docs/interpretations/players-multiplayer.md) becomes alternative (A).

**Ruling, Stinging Study (2021-04-16):** "If you own multiple commanders on the battlefield and/or in the command zone, you choose one of them as Stinging Study resolves to determine the value of X."

**Task.**
1. Add a value that has the controller choose one object from a set as the effect resolves (CR 608.2c, 702.124e) and uses its characteristic. Keep it general: any "the [characteristic] of a [object] you [own/control]" that names a single object picked from several needs it, so compile it rather than hand-writing it.
2. Compile Stinging Study's "the mana value of a commander you own on the battlefield or in the command zone" to it. Validate the answer; the default is the first listed candidate (`default-first-listed-choice`).
3. Leave "the greatest mana value of a commander you own" (Cloudkill, Majestic Genesis) as it is.
4. Rewrite the register entry:
   - id `a-commander-you-own-is-chosen`;
   - "Decided by the user, Nov 10";
   - keep the old reading as a rejected alternative;
   - update every `interp!` that cites the old id.
5. Tests:
   - choosing the smaller commander gives the smaller X;
   - the default answer;
   - a commander another player controls still counts.
6. Run the round trip (`mtg-tools roundtrip --check`).

## Part C: done by the coordinator (Nov 10)
- `target-restriction-source-attachment-as-announced` (Plasma Caster) and `star-without-defining-ability-is-zero` (Tarmogoyf under Dress Down) marked decided: the user accepted both recommendations.
- finish-line-plan §5 records the answers to D1–D6.
