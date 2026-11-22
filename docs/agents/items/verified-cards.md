# verified-cards: make "verified" (passes every check) the headline card number, and keep unverified cards out of simulations by default

WHY: the goal (user, Oct 14) is a simulation that follows the rules precisely, and being as sure of that as possible. "Fully supported" only means every ability compiled. About 3,100 of those cards fail the Oracle round trip, about half have an ability whose structure no test runs, and simulations use all of them. The user chose to count only cards that pass every check as verified, and to add a strict mode that leaves the rest out of simulations.

1. Tiers. Define a card's tier in one place that mtg-tools, mtg-sim and mtg-api can all use (e.g. a small module in mtg-engine with the loader for the generated file below). The tiers, lowest first:
   - `unsupported`: some ability isn't compiled.
   - `compiled`: fully supported, today's meaning (every ability on every face compiled or hand-written).
   - `round-trip`: compiled, and the Oracle round trip passes (`docs/roundtrip-passing.txt`). A card with hand-written abilities counts when `manual-check` passes for it (it has its test).
   - `verified`: round-trip, and also
     - structure-verified: every ability structure on the card is exercised by some test, as `structure-coverage` computes it from a structure log (a card with no abilities qualifies); and
     - fuzz-clean: in the latest every-card sweep the card was cast or played, and no panic, hang, slow game or rules-check violation involved it. A card the sweep can't cast at all (no mana cost and no other way, `docs/EVERY_CARD.md` "Not used") is exempt from "cast or played", with the reason recorded.
   Record which checks each card fails, so every report can say why a card isn't verified.
2. Data. Add `mtg-tools verified --write` (a new subcommand in the style of `roundtrip` and `structure-coverage`). It writes `docs/verified-cards.tsv`: one line per paper card legal in some format, with name, tier and failed checks. Its inputs:
   - the live compile status;
   - `docs/roundtrip-passing.txt`;
   - a structure log directory, from `MTG_STRUCTURE_LOG=<dir> cargo test --workspace`;
   - an every-card report, from `mtg-sim --every-card --report FILE`. Extend the report if it doesn't record every card's outcome, clean cards included.

   Take the two expensive inputs as paths. Produce them once in this item: the sweep takes about an hour with `--threads 3`, so run nothing else heavy alongside it. Do this at the end, after merging origin, because other items change round trips and structure coverage all the time. Document the whole regeneration recipe in the report.
3. Headline. Regenerate `docs/CARD_COVERAGE.md`. It is stale: it still says 74.2% fully supported, against about 93% now. Make it lead with verified cards (count and % of the 31,099 paper cards), then a table of the tiers and how many cards fail each check, per format as today. Update the README and any other headline that says "supported" so that verified comes first. `card-coverage`'s existing numbers stay as the compiled tier.
4. Strict mode:
   - mtg-sim: `--cards verified|round-trip|compiled` sets the lowest tier a deck's cards may have. The default is `verified` for ordinary games (`--deck` files and the default deck).
     - The fuzz modes keep their current pools by default, because their job is to find bugs in every supported card. They still accept the flag: `--random-decks --cards verified` draws only verified cards.
     - A deck with a card below the bar is rejected before the game starts, listing every such card and the checks it fails.
     - If the default deck has unverified cards, replace them with verified cards that fill the same role.
   - mtg-api: apply the same bar, default `verified`, wherever decklists become cards for a game (`Session`, the agent protocol), with an explicit option to lower it. Document it in `docs/AGENT_PROTOCOL.md`.
   - `TestGame` and the engine's `Game` don't enforce it, because tests must be able to use any card.
   - A card missing from `docs/verified-cards.tsv` (card data newer than the last regeneration) counts as at most `compiled`. If the file and the live status disagree (a card no longer compiles, or no longer round-trips), the lower tier wins.
5. Keep it current: add the regeneration recipe to CLAUDE.md's command list, and a note in CARD_COVERAGE.md on when to regenerate (before each milestone, and after a new card set or CR update).
6. Tests:
   - A card failing each check lands in the right tier, with the right reasons.
   - mtg-sim rejects a deck with an unverified card and accepts it with `--cards compiled`.
   - `--random-decks --cards verified` draws only verified cards.
   - The mtg-api option works.

DONE when `docs/verified-cards.tsv` and the regenerated CARD_COVERAGE.md exist, the headline is verified cards, strict mode works in mtg-sim and mtg-api with `verified` as the default for ordinary games, and the regeneration recipe is documented. Report the count in each tier and the most common reasons cards miss `verified`.
