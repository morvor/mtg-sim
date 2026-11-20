# sweep-nov20: full every-card sweep and tier refresh

Tooling item (effort high; review: check the numbers and the diff of docs/verified-cards.tsv for surprises, at max). The last full refresh predates wave 6; many items since changed compiled forms, fingerprints and behavior.

Run exactly CLAUDE.md's "Regenerate docs/verified-cards.tsv and docs/CARD_COVERAGE.md" sequence: the structure-log test run, the four every-card chunks (one command each, ~40 min, nothing else heavy alongside; CARGO_BUILD_JOBS=2), the concatenation, `verified --write`, `card-coverage --write`, and the round-trip list (`roundtrip --write` if that is how docs/ROUNDTRIP.md is refreshed). IMPORTANT: the container is reclaimed while a session sits idle, which kills background runs. Keep the session active: run each chunk in the foreground in pieces or wait on it with Monitor / short foreground waits; resume a killed chunk by re-running it.

Report: verified / round-trip / compiled counts and percentages before and after (from the committed files), every card that dropped a tier with the failed check, and any panic, hang, slow game or rules-check violation the sweep found (with its repro command). Investigate each panic/hang/violation enough to say whether it's a bug, and list it; fix only trivial ones, the rest go to the coordinator. Commit the regenerated files with a yield row; follow worker.md's one-item-per-session rule and notify the coordinator.

## Result (Nov 20)

**Sweep** (worker, 231ddc5): CLAUDE.md's sequence on the engine at cbf0113: the structure-log
`cargo test --workspace` (15 log files, as a whole workspace run writes), the four every-card
chunks (32,673 cards, 65,346 games, 141 min with 3 threads; numbers per chunk in
`docs/EVERY_CARD.md`, "Sweep for the card tiers (Nov 20)"), `roundtrip --write` (28,616 ->
28,618: Berserk and Tilonalli's Summoner, already passing), `verified --write`,
`card-coverage --write`. No panic, hang or rules-check violation; the only failed games are
Charismatic Conqueror's two slow ones (card 4509, the known open loop; it was fuzz-unclean,
slow, before too).

**Tiers** (of 32,111 paper cards; from the committed files):

| Tier | Before (Nov 5 file) | After (landed) |
|---|---:|---:|
| verified | 18,113 (56.4%) | 19,827 (61.7%) |
| round-trip | 10,457 (32.6%) | 8,901 (27.7%) |
| compiled | 3,506 (10.9%) | 3,348 (10.4%) |
| unsupported | 35 (0.1%) | 35 (0.1%) |

Upgrades (1,766): 1,606 round-trip -> verified (1,591 whose only failed check was an unexercised
structure, now exercised by the tests added since Nov 5; 15 cast this time), 120 compiled ->
verified and 40 compiled -> round-trip (the 160 cards that joined `docs/roundtrip-passing.txt`
since the old file, which counted 28,460; 2 left it, below). For the 9,582 cards that fail the
structure check in both files the number of structures is the same, and no card newly fails it
except Hylda (below). The fuzz-cast check moved both ways: 32 cards cast on Nov 5 weren't cast
now, 35 not cast then were.

**Drops** (15; 14 after the fix), each investigated:
- Aang, at the Crossroads // Aang, Destined Savior and Archangel Avacyn // Avacyn, the Purifier
  (round-trip -> compiled): tooling, not a regression. register-sweep-1's review (Nov 10)
  removed the round-trip equivalence of "at the beginning of the next upkeep" and "the next
  turn's upkeep" (`oracle/render/compare.rs`) and named these two cards; the old tier file
  predates it. Both wordings compile to `BeginningOf { Upkeep, Any }`, rendered "the next turn's
  upkeep".
- Hylda of the Icy Crown (verified -> round-trip, structure): test gap. Its trigger's current
  structure (it checks that you are the one tapping) was exercised by no test: its only test put
  the trigger on the stack and never resolved it. `tests/cards/td_tap_causes.rs` now pays for
  it, resolves a mode and declines once; verified again (incremental refresh, 2160b44).
- Twelve cards never cast (verified -> round-trip, fuzz-cast): eleven had no legal cast in their
  games (no Aura for Aura Finesse; no Mountain for Chained to the Rocks; no white or blue
  creature for Combust; one Vampire at most for Feast of Blood; five creatures at most for Hex;
  no power 4 or more for Humble the Brute and Radiant's Judgment; too few untapped lands for It
  That Betrays and Karador, Ghost Chieftain; only basic lands for Lava Blister), Hogaak's casts
  failed on the focus agent's random convoke and delve answers (the report's "offered but never
  chosen" also covers a cast chosen whose payment then failed), and Inspired Ultimatum hit an
  engine bug: the payment planner activates a mana ability without {T} at most once per payment
  (Diamond Kaleidoscope's "Sacrifice a Prism token: Add one mana of any color." with two
  Prisms). Old, not a regression; open in `docs/agents/engine-gaps.md` ("a repeatable mana
  ability is planned at most once per payment") with its repro and an approach.

**Review** (Nov 20): recounted the tiers in the tier file against its header and
`docs/CARD_COVERAGE.md` (every count and check total agrees; the 110 round-trip-or-better cards
missing from `docs/roundtrip-passing.txt` are all hand-written cards); classified every tier
change against origin by its failed checks (above; no card gained a tier without its failed
checks clearing, notes unchanged); sampled 14 upgrades (8 structure, 4 round-trip to verified, 2
to round-trip: each structure exercised by the card's own test, each round trip in the passing
list); re-replayed Chained to the Rocks, Combust, Lava Blister and Karador (both games each),
Hogaak and Inspired Ultimatum (verdicts hold; Karador's reason corrected in
`docs/EVERY_CARD.md`); confirmed the planner bug (the implementer's two tests fail; scratch
probes: with Ashnod's Altar and two creatures nothing pays {4}, with one creature and two lands
it does; Skirk Prospector with two other Goblins can't pay {R}{R}{R}) and the Aang/Avacyn
history (register-sweep-1's notes: 28,459 pass after the change, from 28,460). The implementer's
two `#[ignore]` tests were removed (no ignored tests in the suite; the bug is recorded instead);
Hylda's test now also checks that the reflexive ability uses the stack and that declining after
a second trigger does nothing (that part held vacuously). After merging origin (small-fixes-6),
the structure log was regenerated and the tiers refreshed with every sweep result carried over:
no card's tier changed (Bloodbraid Marauder, whose cascade small-fixes-6 moved to the stack,
stays verified).

**Follow-ups** (for the coordinator):
- The payment planner bug above (engine-gaps.md entry; size S-M), then re-sweep the cards it
  names with `--names`, Inspired Ultimatum included.
- A distinct trigger for "at the beginning of the next turn's upkeep" (CR 603.7b; first upkeep
  of the next turn) as register-sweep-1 proposed: Aang and Avacyn round-trip again, and the 57
  "next turn's upkeep" cards (Heal, Ritual of Steel, Bone Harvest, ...) stop firing at a second
  upkeep of the same turn.
- Sweep tooling (optional): record a cast chosen whose payment failed apart from "offered but
  never chosen"; let the focus agent pay convoke and delve in full more often (Hogaak); have the
  deck builder add the land an "Enchant <land type> you control" Aura names (Chained to the
  Rocks) and a nonbasic land for cards that target one (Lava Blister).
- Charismatic Conqueror's open loop (slow games, documented in `docs/EVERY_CARD.md`).
