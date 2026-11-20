# small-fixes-7: repeatable mana abilities in the payment planner, and "the next turn's upkeep"

Implementation item (effort high; review at max). Quote every CR rule and ruling you rely on verbatim; check every ruling of the cards involved before calling anything provisional.

1. **The auto-payment planner uses a repeatable mana ability without {T} at most once per payment** (sweep-nov20; docs/agents/engine-gaps.md "Residual: a repeatable mana ability is planned at most once per payment"): mana_abilities.rs sources_inner gives one ManaSource per ability. Repros: Diamond Kaleidoscope with two Prisms; Ashnod's Altar with two creatures can't pay {4}; Skirk Prospector with two Goblins can't pay {R}{R}{R}; Inspired Ultimatum; about 27 cards. Plan repeatable abilities as many times as their costs can be paid (CR 605, 601.2g–h, 106.4), keeping the planner's preferences (`automatic-mana-payment`) and making the pairwise sacrifice check sound with repeats (it already offers a {3} spell with Blood Pet and Ashnod's Altar, then reverses the cast). Tests per repro. Then re-run the every-card sweep only for the affected cards (`--names`, CLAUDE.md's incremental refresh) and commit the tier changes.
2. **"At the beginning of the next turn's upkeep" vs "the next upkeep"** (sweep-nov20; register-sweep-1's review stopped equating them in the round-trip compare): Aang, at the Crossroads and Archangel Avacyn dropped to compiled. Give the next turn's upkeep its own delayed trigger timing (CR 603.7b; it differs from "the next upkeep" when created during an upkeep, or in multiplayer), compile the cards to it, check every card using either wording (Paradox Haze was the earlier example) and every ruling on them, and restore their round trip.
3. Optional sweep tooling from sweep-nov20's follow-ups, only if small: a "chosen but failed to pay" label in the every-card report; the focus agent paying convoke/delve in full.

`cargo test --workspace` passes; yield row; follow worker.md's one-item-per-session rule and notify the coordinator.

## Done (implementer, Nov 20)

1. Repeatable mana abilities: `mana_abilities.rs` lists a mana ability whose cost neither taps nor
   sacrifices its permanent nor includes mana once per activation a payment could make
   (`repeat_bound`), matches what all the planned activations sacrifice or exile to distinct objects
   (`match_sacrifices`; it replaces the pairwise check inside the planner), and `pay_mana` keeps what the
   later activations need out of each activation's choices (`Game::mana_keep`). Register:
   `repeatable-mana-abilities-planned-per-activation` (policy, decided); `automatic-mana-payment` points
   to it. Tests: `tests/cr/r605_003_repeatable_mana_abilities.rs`. Still never planned (separate gaps,
   in engine-gaps.md): costs that tap other untapped permanents, discard or exile from hand, and
   Squandered Resources (its mana depends on the land chosen).
2. "At the beginning of the next turn's upkeep": `turn_structure::next_turns_upkeep` (the first upkeep
   of the turn after the one the delayed ability was created in; the creation turn reaches the
   condition as `DELAYED_CREATED_TURN`, set in `triggers.rs`), parsed by `delayed_grammar` and
   `triggers_delayed`, rendered by `delay_text`; "the next upkeep" stays `BeginningOf { Upkeep, Any }`
   and now renders as itself. Aang and Archangel Avacyn round-trip again (28,618 -> 28,620). Register:
   `next-turns-upkeep-is-the-next-turns-first` (interpretation, provisional: with the next turn's upkeep
   skipped it never triggers). Tests: `tests/cr/r603_007b_next_turns_upkeep.rs`.
3. Sweep tooling: the report says "never cast: chosen, but never completed" for a cast the focus agent
   chose that never went through (`Usage::chosen_cast`), apart from "offered but never chosen"; the
   focus agent answers convoke, improvise and delve choices for its card with the engine's default
   (which pays as much that way as the rest needs) 70% of the time.
4. Partial sweep (`--names`, 88 cards: the repeatable mana-ability cards, Inspired Ultimatum, Hogaak and
   every card worded "the next upkeep" or "the next turn's upkeep"; 176 games, no failure) merged over the
   carried-over results with a fresh structure log: verified 19,827 -> 19,835, round-trip 8,901 -> 8,895,
   compiled 3,348 -> 3,346. Up: Aang, at the Crossroads and Archangel Avacyn (compiled -> verified, with
   structure tests in `tests/cards/structure_next_upkeep_transformers.rs`), Diamond Kaleidoscope, Hogaak,
   Inspired Ultimatum, Phyrexian Altar, Treasonous Ogre, Workhorse (round-trip -> verified). No drops.

## Review (Nov 20)

- Planner: `plan_with_filter` planned a filter's own cost and the rest of the payment apart, so
  Ashnod's Altar paying Celestial Prism's {2} and three activations of Phyrexian Altar were planned
  with three creatures: Yore-Tiller Nephilim was offered and the cast reversed (a regression of the
  repeats). The combined plan is now matched too.
- Planner: the matching took the planner's order of preference for the order of activation, so Molt
  Tender (tapped for mana, ranked after the Altars) couldn't be tapped and then sacrificed: two Altars,
  a Bears and Molt Tender paid {4}, which the test pinned as the most the rules allow; it's {5}.
  `Planner::exec_key` (activations that take no other object first, then those whose permanent
  another cost could take, then the rest) is the order the plan is matched in and carried out in.
- Adversarial probes (scratch, not kept): Krark-Clan Ironworks alone pays {2} not {4}, with Sol Ring
  {6}; two Krark-Clans and an Ornithopter {6}; Kozilek's Translator and Mana Bloom ("Activate only
  once each turn") once; Treasonous Ogre at exactly 9 life pays {R}{R}{R} (to 0); two Thermopods with
  Ashnod's Altar {4}; Workhorse's four counters then the Altar sacrificing it {6}; Blood Pet with
  Phyrexian Altar and a Bears {1}{B}, not {1}{B}{B}; Pentad Prism. Timing (debug): 150 permanents
  with five sacrifice outlets, about 0.1 s for a castability check, 0.18 s to cast Emrakul; a failing
  {R}x8 search with three outlets and seven Goblins, under 1 ms. Release: 100 games of an altar mirror
  with rules checks in 26 s (the default deck's 40 games take 3.6 s, the altar mirror's 7.3 s); 300
  random-deck games clean; the 88-card partial sweep re-run with the final code gives the same report.
- "The next turn's upkeep": every card's ruling read (all dates); none speaks to additional or
  skipped upkeeps. 614.10a was missing from the register: its first sentence ("Anything scheduled for a
  skipped step ... won't happen") supports the reading for a skipped upkeep, its second ("Anything
  scheduled for the 'next' occurrence of something waits for the first occurrence that isn't
  skipped") makes a skipped turn not the next turn (500.11, "as though it didn't exist"), now pinned
  (`a_skipped_turn_isnt_the_next_turn`, Chronatog). Still provisional (alternative (b)).
- Left (engine-gaps.md, the residual's entry): convoked or improvised creatures stay in the planner's
  sacrifice pools; chains of consumers in another order; a cost's own sacrifice parts aren't kept
  from its mana's plan (Bone Splinters with Phyrexian Altar and one creature is offered and reversed,
  on origin too; Fling with two creatures since the repeats).
