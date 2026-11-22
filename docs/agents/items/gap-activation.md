# gap-activation: Activated abilities: complete cost modification and activation timing permissions/restrictions

GOAL: Activated-ability costs and timing are handled by fixed code paths: cost modifiers ignore targets, colored reductions, alternative activation costs, ordering and the 'not less than one mana' floor, and nothing can relax or tighten activation timing (instant-speed equip/loyalty, twice-per-turn loyalty, as-though-haste, 'Activate only as an instant' mana abilities). Many well-known cards (Heartstone, Zirda, Training Grounds, Power Artifact, The Wandering Emperor, Oath of Teferi, Lion's Eye Diamond) depend on this.

GAPS:

[G03] Activated-ability cost modification is incomplete
- Rules: CR 602.2b (applies 601.2b-h to abilities), 601.2f (total cost: increases, then reductions;
  target-dependent changes are known because targets are chosen first), 118.7, 118.9 (alternative costs
  exist for activated abilities too: "rather than pay the equip cost").
- Engine now: `Game::ability_total_cost` (casting.rs:1846-1920) applies modifiers in static order (a
  reduction can be applied before a later increase), ignores `ReduceColored` and `AlternativeCost`
  (`_ => {}` at casting.rs:1900), and matches only the source (`self.matches(src, f)`), never the
  ability's targets — it isn't even given the stack object, although targets are chosen before costs
  (casting.rs:2063-2069).
  There is also no "This effect can't reduce the mana in that cost to less than one mana" floor anywhere
  in the engine (no hits outside oracle/).
- Cards (UNS): Heartstone, Zirda, the Dawnwaker, Training Grounds, Biomancer's Familiar, Power Artifact
  ("cost {2} less ... can't reduce the mana in that cost to less than one mana"); Kíli the Resourceful
  ("pay {0} rather than pay the equip cost of the first equip ability you activate each turn"), Highway
  Reaver (unearth), New Perspectives (cycling); Dwarven Mauler and 3 others ("Equip abilities you activate
  that target ~ cost {N} less"), Kopala, Warden of Waves ("Abilities your opponents activate that target a
  Merfolk you control cost {2} more").
- Rulings: Training Grounds "takes the total cost to activate ... into account, not just the cost printed
  on it" (Suppression Field example: increases first), "If an activated ability ... costs no generic mana
  ... it won't increase the cost to include a mana payment of {1}"; Power Artifact "Only affects the generic
  mana part"; Zirda: same floor wording.
- Approach: give `ability_total_cost` the stack id (targets) and the 601.2f ordering used for spells
  (collect increases, then reductions), support `ReduceColored` and a `floor_one_mana` flag on
  reductions (applies only if the cost has mana; never adds mana), and add alternative activation costs
  as an announced choice (with "first ... each turn" tracking).
- Size: M

[G04] Permissions to activate abilities outside their normal timing or limits
- Rules: CR 602.5d (sorcery-timing activation), 606.3 (loyalty abilities once per turn, sorcery timing),
  302.6 / 609.4 ("as though it had haste" for {T} abilities).
- Engine now: `can_activate` (casting.rs:1698-1790) hard-codes the sorcery check
  (`act.timing == Sorcery || act.is_loyalty`, casting.rs:1729-1732) and loyalty's once-per-turn limit
  (casting.rs:1761-1767); no static or effect can relax them (`kw::activation_allowed` can only forbid). The {T}/{Q} payability check accepts only the Haste keyword
  (casting.rs:2316-2317); there is no "as though it had haste" permission (as_though.rs implements only
  spend-as-any-color and other-graveyards).
- Cards (UNS): Leonin Shikari, Forge Anew (equip at instant speed), The Wandering Emperor, Teferi,
  Temporal Archmage (emblem), Teferi, Master of Time, Jace's Machinations (loyalty at instant speed on any
  turn), Oath of Teferi, Urza, Planeswalker (loyalty twice each turn), Thousand-Year Elixir ("activate
  abilities of creatures you control as though those creatures had haste").
- Rulings: Leonin Shikari "You're still subject to any other restrictions on activating equip abilities";
  Oath of Teferi "you may activate the same ability of a planeswalker twice", "If you somehow control more
  than one Oath of Teferi, you won't be able to activate ... more than twice"; Thousand-Year Elixir "doesn't
  actually grant haste ... nor does it let you attack with them as though they had haste".
- The opposite restriction is missing too: "Activate only as an instant" on a mana ability (CR 602.5e) must
  stop it being activated while a spell is being cast or paid for (CR 605.3a would otherwise allow it);
  `ActivationTiming::Instant` is the unrestricted default (ability.rs:209-225), so there's no way to say
  it. Cards (UNS): Lion's Eye Diamond, Diamond Lion, Rhystic Cave (ruling LED: "it can only be activated at
  times when you can cast an instant. Yes, this is a bit weird").
- Approach: a `PlayerModification`/static "activation permission" (filter on ability kind/source:
  equip, loyalty of filter, all abilities of filter) with fields instant_timing / any_turn / per_turn_limit
  / ignore_summoning_sickness, consulted in `can_activate` and the {T} payability check; an
  `ActivationTiming::AsInstant` that requires priority with no payment in progress.
- Size: M

ESTIMATED SIZE: G03 M, G04 M.

OVERLAPS: gap-spell-costs and gap-mana-payment also edit casting.rs cost totals and payment (`total_cost_with`, `ability_total_cost`, `pay_total_cost`): share one 601.2f ordering helper; gap-rule-statics adds payment restrictions (can't pay life/sacrifice) in the same functions; gap-combat adds the attack-side 'as though it had haste'.

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
