# structure-tests: final report

Stream brief: `docs/agents/items/structure-tests.md`. Coverage report: `docs/STRUCTURE_COVERAGE.md`.

## 1. Summary

Every ability of every supported card compiles to a *structure* (its AST with literal
parameters abstracted, `crates/mtg-engine/src/structure.rs`). Abilities with the same structure
run the same engine code, so a test of one card covers all cards that share it. The stream wrote
card tests that run the most widely shared structures no test had run yet, and checked that each
card does what its text says.

- 10 items (`structure-tests-1` to `-10`), each taking about 60 structures of one or two ability
  kinds: 600 structures targeted, all 600 now exercised.
- 595 tests in 31 files:
  `crates/mtg-engine/tests/cards/structure_{triggered,static,activated,spell,mana,keyword}_<N>*.rs`.
- Landed commits: 1 `6a8296f`, 2 `88bb262`, 3 `ec25bc7`, 4 `9bfce2a`, 5 `d3257b8`, 6 `3c9f7e7`,
  7 `5997993`, 8 `afe89fc`, 9 `7e2f13d`, 10 `37ee5e0`. Final coverage doc: `170c278`.
- Before the stream, 548 unexercised structures were shared by 3 or more cards. Now there are
  none. The most widely shared unexercised structure now has 2 cards.

### Numbers

Each measurement is a full `cargo test --workspace` run with `MTG_STRUCTURE_LOG` set, followed by
`mtg-tools structure-coverage`. "Measured at" is the origin commit whose code was measured. The
regenerated doc was committed by the next item to land.

| Measured at | Items landed | Distinct structures exercised | Card abilities exercised | Supported cards structure-verified | Unexercised structures shared by 3+ cards |
|---|---|---:|---:|---:|---:|
| `06aa0f8` (start) | none | 8929 / 25071 (35.6%) | 34641 / 53480 (64.8%) | 12524 / 28381 (44.1%) | 548 |
| `88bb262` | 1–2 | 9100 | | 13134 | |
| `ec25bc7` | 1–4 | 9354 | | 13621 | |
| `afe89fc` | 1–6, 8 | 9819 | | 14317 | |
| `7e2f13d` (final) | 1–10 | 10089 / 25358 (39.8%) | 37820 / 53939 (70.1%) | 14743 / 28605 (51.5%) | 0 |

The totals grew during the stream because about 220 more cards became supported through other
work. Those cards brought new structures, most of them unexercised. Over the stream, exercised
structures went up by 1160, exercised card abilities by 3179 and structure-verified cards by 2219.

## 2. Bugs found and fixed

### Engine and parser

**Tokens created "with changeling" lacked changeling's characteristic-defining ability.** This
affected Birthing Boughs, Stalactite Dagger, Belonging, Crib Swap, Irregular Cohort, Springleaf
Parade, Formless Genesis and every other card that creates such a token. The token got the
changeling keyword but not the CDA that makes it every creature type (CR 702.73a). CR 604.3a(2)
counts an ability "granted to the token it affects by the effect that created the token" as a
CDA. In practice:

- Party counts missed the token. With a Cleric and a changeling token, Archpriest of Iona had
  power 1 instead of 2.
- Under "loses all abilities" effects (Humility, Ovinize) the token would have lost its creature
  types. A real changeling keeps them, because the CDA applies in layer 4, before layer 6 removes
  the ability.

The fix is in the token-spec parser: `oracle/effects.rs` and
`oracle/patterns/tokens_copies_create.rs` now go through `compile_keyword`, which adds the CDA.
The Oracle round-trip renderer was changed to match. Regression tests:

- Stalactite Dagger's token under Humility (`structure_triggered_9.rs`).
- Birthing Boughs' token with Archpriest of Iona, then Ovinize (`structure_activated_10.rs`).

An alternative fix was rejected: letting `Characteristics::has_subtype` read the bare keyword. The
CR does not give creature types to an object that has only the keyword, so that fix would have
been wrong (see section 4).

### Coverage logging only (no change to game behavior)

- Static "Skip your draw step" effects were applied without being recorded, so their structures
  always showed as unexercised (`skip.rs`, `40cbbcd`).
- Two other cases were never recorded: cast keywords on back faces (More Than Meets the Eye), and
  Ascend on instants and sorceries (`structure.rs`, `kw/ascend.rs`).

### Lessons for future test writers

Reviewers kept finding the same problems in test drafts:

- **Citing rules the test doesn't exercise.** `cr!` counts toward CR coverage, so cite only what
  the assertions check.
- **Illegal-target checks that test the harness, not the engine.** `TestGame` swaps an illegal
  answer for a legal one, so "the cast succeeded with a legal target" proves nothing. Assert on
  the candidates the decision offered instead.
- **Vacuous "doesn't trigger" checks.** Triggered abilities wait in `pending_triggers` until a
  player would receive priority. A check made before then passes whether or not the ability
  triggered.
- **Missing decline and unpayable cases for "may" and "you may pay".** Test both branches.
- **Faking a condition by editing engine state directly.** Set up the condition the way a game
  would.
- **Checking only one side of an "A or B" outcome.** For example, the right card is in the
  graveyard, but nothing checks that the other card is still in the hand.

## 3. Open test gaps

Each of these was checked against the current code and tests.

1. **Brutal Deceiver**, "{1}: Look at the top card of your library."
   (`structure_activated_2.rs`, `brutal_deceiver_looks_at_the_top_card`). The test only checks
   that the cost is paid and that nothing moves. Nothing in the engine records that a player has
   seen a card through a one-shot look. `zones::can_see_in_library` knows only about revealed
   top cards and the static "you may look at the top card of your library any time" permission
   (`zones.looked_top`). So the test has nothing to assert about who saw the card.

2. **Armillary Sphere**, "... search your library for up to two basic land cards, reveal them,
   put them into your hand, then shuffle." (`structure_activated_2.rs`). The test checks the
   search, the destination and the shuffle, but not the reveal. The brief said this was because
   the engine has no reveal event. That is no longer true: `reveal::reveal_in` emits
   `Event::Custom { name: reveal::REVEALED }`, and the search path calls it when the effect says
   "reveal" (`resolve.rs`, `search_rules.rs`). Other tests already assert on that event
   (`k702_143_foretell.rs`, `a701_058_cloak.rs`). This gap can be closed by adding an assertion.

3. **Desperate Ravings**, "Draw two cards, then discard a card at random."
   (`structure_spell_6.rs`). The test shows that one of the two drawn cards is discarded and that
   no player is asked to choose. It does not show that the discard is random. With a single seed
   it can't tell random from "always the first card".

4. **One-shot skip effects** ("you skip your next draw step"; Molten Firebird and Ivory Gargoyle
   have it in a dies trigger, Avizoa has "you skip your next untap step"). The skip works in
   games. Coverage doesn't track it: `skip.rs` records a skip only when it comes from a static
   ability (a `uid` is present). A skip created by a resolving effect is applied with no `uid`,
   so nothing is logged. The creating ability counts as exercised as soon as it resolves, so
   coverage can't tell whether any test saw the step actually skipped. No test uses Molten
   Firebird or Ivory Gargoyle.

5. **Rebel Salvo**, "... deals 5 damage to target creature or planeswalker. That permanent loses
   indestructible until end of turn." (`structure_keyword_8.rs`). Only its Affinity for Equipment
   keyword was in scope, so the test casts it at creatures without indestructible. Soul Sear
   shares the spell structure (`f2ed4ff1980dbc6a`). That structure now counts as exercised, but
   only because Rebel Salvo's affinity test resolves the spell. No test asserts that the target
   loses indestructible, or that it gets indestructible back at end of turn. No test uses Soul
   Sear.

## 4. Open decision: changeling granted without the CDA

### The behavior

An object that has the changeling **keyword** without changeling's **characteristic-defining
ability** is, for example, a creature given changeling by an effect. The engine treats such an
object inconsistently:

- **Treated as every creature type** (`kw::changeling::every_creature_type_by_keyword`, via
  `every_creature_type`):
  - `Filter::Subtype` in `eval.rs` ("a Goblin", "Elf you control", ...).
  - `Filter::SharesCreatureType` (`eval.rs`).
  - Prowl (`kw/prowl.rs`).
  - Partner's lone-Doctor check (`kw/partner.rs`, `is_lone_doctor`).
  - The mana spend context for spells (`casting.rs`, `kw/assist.rs`), which decides "spend this
    mana only on a [type] spell".
  - Target groups (`target_groups.rs`).
  - Creature-type aggregates such as party and "creature types among" (`aggregates.rs`).
  - "Creatures that share a creature type" (`oracle/patterns/creatures_share_a_type.rs`).
- **Not treated as every creature type**, because these read only the layer-4 flag
  `all_creature_types` that the CDA sets:
  - `Characteristics::has_subtype` (`object.rs`). Its keyword fallback was removed in `37ee5e0`.
  - `kw/type_counts.rs` ("the number of creature types it has").
  - The Skemfar Shadowsage count in `custom.rs` ("greatest number of creatures you control that
    have a creature type in common").
  - "Of a chosen type" in `kw/choice_grammar.rs`.

The test `a_creature_given_changeling_by_an_effect_is_every_creature_type`
(`tests/keywords/k702_073_changeling.rs`) asserts the first behavior, through `Filter::Subtype`
and a test-only grant.

A printed changeling has the CDA (`oracle::keywords::compile_keyword`). So do changeling tokens
(fixed in this stream), and so does changeling gained through a copy effect, such as
Omni-Changeling's "except it has changeling" or a copy of a changeling. Those objects behave the
same on both paths. The question concerns only an object whose keyword came from any other
effect.

### What the Comprehensive Rules say

- **702.73a:** "Changeling is a characteristic-defining ability. 'Changeling' means 'This object
  is every creature type.' This ability works everywhere, even outside the game. See rule 604.3."
- **604.3** (part): "Some static abilities are characteristic-defining abilities. ...
  Characteristic-defining abilities function in all zones."
- **604.3a:** "A static ability is a characteristic-defining ability if it meets the following
  criteria: (1) It defines an object's colors, subtypes, power, or toughness; (2) it is printed
  on the card it affects, it was granted to the token it affects by the effect that created the
  token, or it was acquired by the object it affects as the result of a copy effect or
  text-changing effect; (3) it does not directly affect the characteristics of any other
  objects; (4) it is not an ability that an object grants to itself; and (5) it does not set the
  values of such characteristics only if certain conditions are met."
- **707.9a:** "Some copy effects cause the copy to gain an ability as part of the copying
  process. This ability becomes part of the copiable values for the copy ..."
- **707.9d** example: a Glasspool Mimic copying a creature with changeling "will have changeling
  and will have all creature types."
- **205.3m:** "Creatures and kindreds share their lists of subtypes; these subtypes are called
  creature types." 205.3d: "An object can't gain a subtype that doesn't correspond to one of that
  object's types."
- **613.1** (part): characteristics are determined by starting with the actual object, "Then all
  applicable continuous effects are applied in a series of layers in the following order:"
  **613.1d** "Layer 4: Type-changing effects are applied. ..."; **613.1f** "Layer 6:
  Ability-adding effects, keyword counters, ability-removing effects, and effects that say an
  object can't have an ability are applied."
- **613.3:** "Within layers 2–6, apply effects from characteristic-defining abilities first (see
  rule 604.3), then all other effects in timestamp order (see rule 613.7). ..."
- **613.7a:** "A continuous effect generated by a static ability has the same timestamp as the
  object the static ability is on, or the timestamp of the effect that created the ability,
  whichever is later. ..."
- **613.8a:** dependency exists only between effects "applied in the same layer". 113.10 defines
  "gains"/"has" as adding an ability.

From these rules:

- Under 604.3a(2), changeling granted by an effect, other than token creation, a copy effect or a
  text-changing effect, is **not** a characteristic-defining ability. 702.73a calls changeling a
  CDA, but 604.3a's criteria decide whether a given instance is one.
- The ability is added in layer 6 (613.1f). Its meaning, "this object is every creature type",
  is a type-changing effect, and those are applied in layer 4 (613.1d), which comes earlier in
  613.1's order.
- No rule says that an ability added in layer 6 causes an effect to be applied back in layer 4.
  The CR does not address this case in words, and 613.8 dependencies only work within one
  layer. Taken alone, 613.1's order means the granted ability doesn't exist yet when layer 4 is
  applied.

(Everyone's an Imposter, "All creatures have changeling.", carries the reminder text "Yes, we
know this doesn't actually work." That is card text on a card legal in no format, not a rule.)

### Options

- **A. Keyword confers types everywhere.** Keep the keyword path and make `has_subtype`,
  `type_counts.rs`, `custom.rs` and `choice_grammar.rs` agree with it. This is consistent and
  matches the intent of a card that grants changeling. No CR text supports it, though: 604.3a(2)
  says the granted ability isn't a CDA, and nothing applies it in layer 4. This also reverses the
  reasoning behind `37ee5e0`.
- **B. Only the CDA confers types.** Remove `every_creature_type_by_keyword`, so
  `every_creature_type` reads only `all_creature_types`. Change `Filter::Subtype` to plain
  `has_subtype`. Change the existing test to assert that a creature given only the keyword is
  *not* every creature type. This is the smallest change, leaves one source of truth (the layer-4
  flag) and follows the layer order in 613.1.
- **C. Model a granted changeling as a non-CDA static ability with a layer-4 effect.** It would
  have its own timestamp (613.7a) and dependency handling. This needs the layer system to apply
  in layer 4 an ability that only exists after layer 6, which the CR doesn't describe. It is the
  most work and has no rule text to model it on.

### Which cards are affected

The card data was searched for cards that give changeling, or "all/every creature type", to
other objects:

- **Grant the changeling keyword itself:**
  - Everyone's an Imposter ("All creatures have changeling") and Oddric, Lunar Marquis, which
    grants changeling among other keywords. Both are legal in no format and so out of scope
    (`docs/DEFERRED.md`).
  - Brigid, Who's Seen Some Stuff (thoughtweft, which shares "printed keyword abilities") is
    also legal in no format.
  - Volrath's Shapeshifter (text-changing; would get the CDA under 604.3a anyway) is legal but
    unsupported.
  - No supported, legal card grants the bare keyword.
- **Copy exceptions:** Omni-Changeling and Moritte of the Frost are supported and legal. The copy
  gets the CDA (`etb_choices_others.rs`, 707.9a), so all options treat them the same.
- **Tokens "with changeling":** Birthing Boughs, Stalactite Dagger, Belonging, Crib Swap,
  Formless Genesis, and others. These get the CDA since this stream's fix (604.3a(2)), so all
  options treat them the same.
- **"Gains/is every (all) creature type(s)":** Amorphous Axe, Runed Stalactite, Arachnoform,
  Mirror Entity, Volatile Claws, Amoeboid Changeling, Glamer Gifter, Blades/Shields/Wings of
  Velis Vel, Oko (Lorwyn Liege), Mutavault, Faceless Haven, Soulstone Sanctuary, Undercover
  Skrull, Mistform Ultimus, and others. These compile to `Modification::AllCreatureTypes` (layer
  4), not to the keyword, and are unaffected by every option. Maskwood Nexus, Omo, Queen of
  Vesuva (the creature half is unsupported text) and Snazzy Aether Homunculus (digital-only) are
  not supported.

So **no supported, legal card behaves differently under A, B or C today**. The choice decides
which way the engine is internally consistent, plus one existing test
(`k702_073_changeling.rs`). It also decides what a future grant of the bare keyword would do.

### Recommendation

**Option B.** It is the reading the rule text supports: 604.3a(2) says the granted ability isn't
a CDA, 613.1 applies layer 4 before layer 6, and no rule applies a layer-6 grant back in layer 4.
It makes `has_subtype` and the filters agree, keeps a single source of truth (the layer-4
`all_creature_types` flag), and changes no supported card's behavior. The cost is rewriting the
one test that asserts the current keyword behavior.

Option C should be considered only if a supported card ever needs it, together with a ruling
that says how such a grant works.
