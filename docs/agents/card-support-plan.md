# Card support to 100%: work plan

Generated from the prebuilt `mtg-tools` (HEAD 85f020d) and the Scryfall data in `data/`. Reproduce with the scripts in this directory (see "Method").

## Baseline

- Paper cards (card-coverage denominator): **31,099**; fully supported **22,714 (73.0%)**; unsupported **8,385**.
- Distinct unsupported ability texts (`mtg-tools unsupported`, numbers as N): **9,979**. 9,049 of them occur on paper cards; **930 occur only on digital-only cards** (Alchemy / Arena), which card-coverage doesn't count.
- 7,611 unsupported paper cards have exactly one unsupported text; 676 have two; 98 have three or more. After clustering, 7,761 unsupported paper cards depend on a single work item and 624 need two or more items.
- Out of scope: 1 text (Steamflogger Boss: "If a Rigger you control would assemble a Contraption, it assembles two Contraptions instead"). Host/augment and Contraption cards are not legal in any format, so they aren't in the denominator; Steamflogger Boss is the only legal card touching Contraptions.

## Method

1. Exact text→card map: the prebuilt mtg-tools run on a one-card data directory (`MTG_DATA_DIR`) for every card (`exact_map.py` → `exact_cards.json`, `textinfo.json`, `cardtexts.json`).
2. Failing piece per text, found with the real compiler (`probe.py`, `probe_round2.py`, `probe_round3.py`, `probe_generic.py`, `run_whole.py`, `run_quote.py`, `probe_round4.py`): synthetic cards are compiled with one component at a time — the cost, the trigger condition, the intervening if, then sentences added one by one under a working frame — to find the first failing component; then variants of the failing sentence (value phrase replaced by a number, condition removed, duration removed, object phrase replaced by "target creature", quoted ability compiled alone, "A, then B" split, player subject replaced by "you") show which construct fails.
3. Classification (`rules3.py`, `classify3.py` → `classes3.json`): the failing piece and the variant results decide the construct; regex rules on the piece map constructs to work items. Families inside each item: `families.py` → `famstats.json`. Outputs: `assemble.py`, `write_plan.py`.
4. Spot checks: random samples of every item were read; classification follows the first failing construct, so roughly 10–15% of texts also contain a second failing construct owned by another item (briefs tell workers to fix small ones and list the rest).

## Work items (ordered by cards unlocked per unit of effort; manual-defs first)

Effort = texts × difficulty factor (1.0 = parser grammar over existing engine support; higher where engine work or shared-core risk is expected). "Unlocked" = paper cards whose every unsupported text belongs to the item.

| # | Item | Texts | Paper cards touched | Unlocked alone | Factor | Unlocked / effort |
|---:|---|---:|---:|---:|---:|---:|
| 1 | cards-manual-defs (mechanism + first 32 of 83 one-off cards) | 92 | 83 | 70 | – | – |
| 2 | cards-search | 187 | 190 | 171 | 0.8 | 1.14 |
| 3 | cards-costs-activated | 288 | 292 | 239 | 0.8 | 1.04 |
| 4 | cards-values-counts | 397 | 399 | 349 | 0.9 | 0.98 |
| 5 | cards-conditions-referents | 457 | 457 | 422 | 1.0 | 0.92 |
| 6 | cards-graveyard-hand | 347 | 353 | 317 | 1.0 | 0.91 |
| 7 | cards-basic-effects | 306 | 315 | 275 | 1.0 | 0.90 |
| 8 | cards-counters | 231 | 225 | 185 | 0.9 | 0.89 |
| 9 | cards-player-subjects | 329 | 336 | 292 | 1.0 | 0.89 |
| 10 | cards-grants | 273 | 273 | 241 | 1.0 | 0.88 |
| 11 | cards-restrictions | 401 | 402 | 345 | 1.0 | 0.86 |
| 12 | cards-zone-moves | 342 | 348 | 294 | 1.0 | 0.86 |
| 13 | cards-choices | 317 | 318 | 288 | 1.1 | 0.83 |
| 14 | cards-library-dig | 275 | 280 | 246 | 1.1 | 0.81 |
| 15 | cards-triggers-combat-cast | 333 | 328 | 266 | 1.0 | 0.80 |
| 16 | cards-conditions-state-unless | 301 | 302 | 258 | 1.1 | 0.78 |
| 17 | cards-permanent-status | 329 | 331 | 282 | 1.1 | 0.78 |
| 18 | cards-filters-relational | 318 | 323 | 290 | 1.2 | 0.76 |
| 19 | cards-tokens-copies | 223 | 219 | 186 | 1.1 | 0.76 |
| 20 | cards-delayed-reflexive | 220 | 220 | 198 | 1.2 | 0.75 |
| 21 | cards-iteration-modal | 325 | 322 | 286 | 1.2 | 0.73 |
| 22 | cards-characteristics | 189 | 191 | 166 | 1.2 | 0.73 |
| 23 | cards-values-results | 391 | 393 | 342 | 1.2 | 0.73 |
| 24 | cards-triggers-events | 295 | 293 | 228 | 1.1 | 0.70 |
| 25 | cards-costs-spells | 157 | 169 | 129 | 1.2 | 0.68 |
| 26 | cards-mana | 81 | 81 | 72 | 1.3 | 0.68 |
| 27 | cards-history-conditions | 326 | 328 | 285 | 1.3 | 0.67 |
| 28 | cards-filters-noun-phrases | 127 | 126 | 108 | 1.3 | 0.65 |
| 29 | cards-replacements | 388 | 406 | 318 | 1.4 | 0.59 |
| 30 | cards-cast-permissions | 350 | 352 | 266 | 1.3 | 0.58 |
| 31 | cards-game-rules | 207 | 201 | 165 | 1.5 | 0.53 |
| 32 | cards-dice-un | 237 | 177 | 172 | 1.5 | 0.48 |
| 33 | cards-digital-alchemy | 939 | 8 | 8 | 2.0 | 0.00 |

Totals: 9,979 texts = 8,947 in the 31 paper items + 939 in cards-digital-alchemy (930 digital-only texts plus 9 paper texts using Alchemy wording) + 92 manual + 1 out of scope. Sum of "unlocked alone" over the paper items: 7,681 cards; plus 70 cards whose only unsupported texts are manual.

Titles and one-line themes:

- **cards-manual-defs** — Hand-written card definitions: src/cards/ registry, compiler hook, coverage reporting and guards; first 32 of 83 one-off cards
- **cards-search** — Search grammar: multi-card and multi-zone searches, dynamic filters, other players' libraries, split destinations — 187 texts / 190 cards (171 unlocked alone)
- **cards-costs-activated** — Cost grammar I: activated-ability costs and activation cost modifiers — 288 texts / 292 cards (239 unlocked alone)
- **cards-values-counts** — Value grammar I: counts and aggregates ("for each [X]", "where X is the number of [X]", "greatest/total power among", "number of [types/colors] among") — 397 texts / 399 cards (349 unlocked alone)
- **cards-conditions-referents** — Condition grammar I: conditions about referents ("if it/that creature/the sacrificed card ...", "if you do/can't", "... this way") — 457 texts / 457 cards (422 unlocked alone)
- **cards-graveyard-hand** — Card-flow grammar II: hand, graveyard and library card actions (exile from graveyards, discard/draw variants, reveal from hand, graveyard statics) — 347 texts / 353 cards (317 unlocked alone)
- **cards-basic-effects** — Basic effect verbs with complex arguments: counter spells/abilities with qualifiers, damage variants, "any number" selections, multi-target removal — 306 texts / 315 cards (275 unlocked alone)
- **cards-counters** — Counter grammar: put/remove/move/double counters with complex placement, counter-qualified groups, counter-linked abilities — 231 texts / 225 cards (185 unlocked alone)
- **cards-player-subjects** — Instruction grammar: any player as the subject ("each player may ...", "target opponent reveals ...", "that player exiles the top ...", "each player who ...", "you and target opponent each ...") — 329 texts / 336 cards (292 unlocked alone)
- **cards-grants** — Ability-granting grammar: quoted abilities on objects/tokens, conditional and group keyword grants — 273 texts / 273 cards (241 unlocked alone)
- **cards-restrictions** — Restriction grammar: activation, casting, combat, targeting and rules prohibitions — 401 texts / 402 cards (345 unlocked alone)
- **cards-zone-moves** — Card-flow grammar III: zone moves ("return/put [cards] from [zone] to/onto [zone] [modifiers]") — 342 texts / 348 cards (294 unlocked alone)
- **cards-choices** — Choice grammar: choose objects/players/types/names, random choices, "of the chosen type", "same name", named options — 317 texts / 318 cards (288 unlocked alone)
- **cards-library-dig** — Card-flow grammar I: dig and distribute ("from among them", "the rest", "reveal until", "revealed this way", piles, library positions) — 275 texts / 280 cards (246 unlocked alone)
- **cards-triggers-combat-cast** — Trigger grammar II: cast, attack, block, damage and targeting trigger conditions — 333 texts / 328 cards (266 unlocked alone)
- **cards-conditions-state-unless** — Condition grammar III: game-state conditions and "unless" (punisher choices, payment taxes, "as long as" states, comparisons between players) — 301 texts / 302 cards (258 unlocked alone)
- **cards-permanent-status** — Permanent status changes: attach/unattach, control changes, transform/convert, face-down/face-up, attachment relations — 329 texts / 331 cards (282 unlocked alone)
- **cards-filters-relational** — Object-phrase grammar I: relational filters ("with lesser mana value", "with the greatest power among", "total mana value N or less", "that shares a creature type with", "other than", history-qualified objects) — 318 texts / 323 cards (290 unlocked alone)
- **cards-tokens-copies** — Token and copy grammar: token creation modifiers, "become a copy ... except", copying cards/spells — 223 texts / 219 cards (186 unlocked alone)
- **cards-delayed-reflexive** — Delayed and reflexive triggers, durations and extra phases — 220 texts / 220 cards (198 unlocked alone)
- **cards-iteration-modal** — Iteration and structure: "for each [player/object], [instruction]", "starting with ...", modal headers, multi-line blocks (levels, Seasons, chapters), repeat loops, flavor words — 325 texts / 322 cards (286 unlocked alone)
- **cards-characteristics** — Characteristic-changing grammar (layers 4-7): becomes [P/T color type] creature, base P/T, P/T CDAs, type/color/supertype changes, ability loss — 189 texts / 191 cards (166 unlocked alone)
- **cards-values-results** — Value grammar II: amounts from earlier instructions and events ("this way", "that much/many", excess damage, "life you gained this turn", referent stats, X paid) — 391 texts / 393 cards (342 unlocked alone)
- **cards-triggers-events** — Trigger grammar I: zone-change, counter, player-action and state trigger conditions — 295 texts / 293 cards (228 unlocked alone)
- **cards-costs-spells** — Cost grammar II: spell additional/alternative costs and spell cost modifiers — 157 texts / 169 cards (129 unlocked alone)
- **cards-mana** — Mana grammar: spend restrictions, mana production variants, mana-spent conditions, mana-ability modifications — 81 texts / 81 cards (72 unlocked alone)
- **cards-history-conditions** — Condition grammar II: game-history conditions ("if you've cast/attacked/sacrificed ... this turn", "if a creature died this turn", mana spent, cast status) — 326 texts / 328 cards (285 unlocked alone)
- **cards-filters-noun-phrases** — Object-phrase grammar II: noun-phrase core (disjunctions with distinct adjectives, color/quality predicates, elliptical "and a [counter] on [Y]" clauses) — 127 texts / 126 cards (108 unlocked alone)
- **cards-replacements** — Replacement and prevention grammar ("if ... would ..., instead", "the next time ... would", prevention/redirection, ETB replacements "as ~ enters", "enter as a copy") — 388 texts / 406 cards (318 unlocked alone)
- **cards-cast-permissions** — Casting permissions: play/cast from zones, impulse durations, "from among cards exiled with ~", flash grants, mana flexibility — 350 texts / 352 cards (266 unlocked alone)
- **cards-game-rules** — Game-rules mechanics: energy amounts, Conspiracy draft-matters, commander zone, loyalty rules, dungeons/initiative/monarch/Ring, villainous choice, win/lose and hand size — 207 texts / 201 cards (165 unlocked alone)
- **cards-dice-un** — Dice, coins and Un-mechanics: d20 result tables, dice/coin modifiers, sticker sheets ({TK} tickets, name/art stickers), Attractions (visit/open) — 237 texts / 177 cards (172 unlocked alone)
- **cards-digital-alchemy** — Digital-only cards (Alchemy mechanics: conjure, seek, perpetual, spellbooks, intensity, specialize, boons; and other digital-only printings) — outside the paper card-coverage metric — 939 texts (930 only on digital-only cards; 8 paper cards)

### Scheduling notes

- The ratio ordering is close to flat (0.5–1.1 unlocked cards per effort unit), so parallel waves matter more than strict order. Suggested first wave beside cards-manual-defs: cards-values-counts, cards-conditions-referents, cards-player-subjects and cards-filters-relational — they build grammar (value phrases, Builder-aware conditions, player subjects, relational filters) that the other items reuse, and they land hooks in shared core files early.
- Shared-file pairs to keep apart or coordinate: cards-filters-relational / cards-filters-noun-phrases (phrases.rs: relational adds a suffix hook, noun-phrases owns the core); cards-costs-activated / cards-restrictions / cards-costs-spells (costs.rs is owned by costs-activated; the others add hooks); cards-triggers-events / cards-triggers-combat-cast (oracle/triggers.rs: land the subject-parser refactor first); cards-delayed-reflexive / cards-cast-permissions (effects.rs duration_suffix); cards-conditions-* (one Builder-aware condition entry point, landed first by cards-conditions-referents); cards-values-results / cards-history-conditions (one event-query Value/Condition AST); cards-tokens-copies / cards-grants (parse_token_description).
- cards-digital-alchemy runs last: most of its non-Alchemy texts share wording with paper cards and will disappear when the paper items land; the rest needs new engine features (perpetual, seek, spellbooks) and doesn't affect card-coverage.

## Missing engine capabilities found

- Game-long history ("this game": spells named ~ cast this game, times the Ring tempted you, commander casts) — only the current and last turn are recorded (TurnHistory/turn_events). Several per-turn facts aren't recorded (cards cycled/discarded per player, cards leaving graveyards, counters put by a player, cards exiled this turn, total power of attackers). → cards-history-conditions (one event-query AST also used by cards-values-results).
- Values compared against referents or aggregates in filters ("with greater power", "with the greatest power among") and group constraints on multiple targets ("total mana value 6 or less", CR 601.2c) → cards-filters-relational.
- Aggregate values: least/greatest toughness, total power/toughness/mana value, distinct mana values/creature types among a filter → cards-values-counts (small eval.rs additions).
- Recording of results of every primitive (affected sets, damage/excess amounts) for "this way" values and conditions; "can't" outcomes of impossible instructions → cards-values-results, cards-conditions-referents.
- Punisher "unless [player] [non-mana action]" chosen by the affected player; quantified player conditions ("an opponent has more ... than you") → cards-conditions-state-unless.
- Targets chosen per iteration of "for each opponent, ... up to one target creature that player controls" and ordered sequential choices ("starting with you") → cards-iteration-modal.
- Multi-step dig distributions and opponent-chosen selections in Dig → cards-library-dig.
- Statics affecting cards in graveyards/hands/libraries ("cards in graveyards lose all abilities", "land cards in your library are basic") — check layers.rs → cards-graveyard-hand, cards-characteristics.
- An outside-the-game source per player (wishes, Spawnsire), loyalty-activation count changes ("twice each turn") → cards-game-rules.
- Sticker sheets as playable objects with ticket costs ({TK}) — the Stickers layout lines are counted as cards today; roll modifiers (add value, choose among rolls) → cards-dice-un (flag scope to the owner).
- Countering abilities by source filter (stack-object filters) → cards-basic-effects; spend restrictions keyed to ability kinds (equip, disturb, unlock door, turn face up) → cards-mana.
- Alchemy: perpetual modifications, seek, spellbooks/draft-from-spellbook, intensity, specialize, boons, heist (spellbook contents may not be in the bulk data) → cards-digital-alchemy.

## Manual (hand-written) definitions

`manual-candidates.tsv`: **92 texts on 83 cards** (70 of these cards need nothing else), chosen conservatively from the texts whose wording is most unique in the whole card pool (`uniq_scores.json`: share of word trigrams that occur on only this card), then read one by one: a card is a candidate only if its construct is a genuine one-off (no other card shares it). Families shared by two or more cards were left to the items even when rare (e.g. bidding, "last chosen", "then do the same for", Rhystic "unless any player pays", volvers, wishes, d20 tables, Pardic Firecat/Diligent Farmhand).

### Mechanism (item cards-manual-defs)

MECHANISM (make it concrete; adjust names to the code you find):

1. Registry `crates/mtg-engine/src/cards/`: one file per card (`<snake_case_card_name>.rs`), auto-included by build.rs exactly like `src/kw/` (add `gen(&root.join("src/cards"), &["mod"], &out.join("card_mods.rs"), "pub ")` and `include!(concat!(env!("OUT_DIR"), "/card_mods.rs"));` in `cards/mod.rs`; `pub mod cards;` in lib.rs). `cards/mod.rs` defines

       pub struct ManualAbility {
           pub card: &'static str,     // Oracle card name (full name for multi-face cards)
           pub face: usize,            // face/half index (0 = front/left)
           pub text: &'static str,     // the exact normalized ability block it replaces, as it
                                       // appears in FaceDef::unsupported today (reminder text
                                       // stripped, self-references "~", original numbers)
           pub build: fn(&crate::oracle::CompileContext) -> Vec<crate::ability::Ability>,
           pub reason: &'static str,   // why this is hand-written (shown in reports)
       }
       inventory::collect!(ManualAbility);
       pub fn lookup(full_name: &str, face: usize, block: &str) -> Option<&'static ManualAbility>

   Per-ability (block) granularity, not per card: most manual cards have one unique ability and their other abilities still compile (and items may fix them). A card file may register several ManualAbility entries.

2. Compiler hook (oracle/mod.rs `compile`): before `parse_ability(&block, ctx)`, `if let Some(m) = crate::cards::lookup(ctx.full_name, ctx.face_index, &block)` → append `(m.build)(ctx)` and record the block in a new `Compiled::manual: Vec<String>` (plumbed to `FaceDef::manual` and `CardDef::manual_text()`); skip parsing. Exact-text matching means an Oracle update makes the definition stale and the block reverts to unsupported (safe default). `is_fully_supported()` is unchanged (manual blocks are not unsupported).

3. Behaviour the AST can't express: the card file may also implement the existing `KeywordRules` trait (kw/mod.rs) with `kinds() -> &[]` and register it with `inventory::submit! { KeywordRegistration(&XRules) }`; it then receives the registry-wide hooks — `custom_effect/custom_condition/custom_trigger/custom_value/custom_filter` (already dispatched over the whole registry, kw/mod.rs ~891-969; name them `card:<Card Name>:<what>` and reference them from the AST via `Effect::Custom`, `Condition::Custom`, `TriggerCond::Custom`, `Value::Custom`, `Filter::Custom`, `StaticEffect::Custom`/`Restriction::Custom`), plus global hooks such as `state_based_actions`, `static_state_checks`, `combat_damage_assigner`, `activation_cost`, `block_allowed`, `attack_declaration_ok`, `damage_prevention`, `on_event`, `after_draw`. For hooks the engine dispatches only to keyword kinds present on an object, add one generic dispatch (iterate all registrations) — a small focused core edit, never a card-name check in the engine. Hooks must find their source permanents themselves (by the custom ability on the object, not by name, so copies and text changes behave: a copy of Trinisphere copies its ability and therefore its rule).

4. Reporting and guards (mtg-tools): `card-coverage` prints "Fully supported: N (compiled: A, with hand-written abilities: B)" and lists hand-written cards with their reasons in docs/CARD_COVERAGE.md; a new `mtg-tools manual-check` (run by a test or by `cargo test` via a test in crates/mtg-engine/tests/cards/) fails when (a) a ManualAbility matches no block of its card in the current data (stale), (b) the compiler can now parse the block on its own (delete the manual definition: compile-first policy), (c) a manual card has no test (a test in tests/cards/ that loads the card by name, e.g. a `manual_cards_tested` list or grep for `"<Card Name>"`), (d) two entries claim the same block.

5. Tests: one file per card, `crates/mtg-engine/tests/cards/m_<card>.rs`, using TestGame with the real card and asserting the in-game behaviour of the hand-written ability (cite cr!/ruling! where the test exercises a rule or ruling; rulings coverage benefits).

6. Round-trip interplay (item oracle-roundtrip): hand-written abilities are verified by their tests, not by rendering; the renderer should report them as "manual" (counted separately, not as mismatches). Mention this in docs.

7. Policy (document in the cards/mod.rs header and CLAUDE.md "Extending"): hand-written definitions are only for genuine one-offs (no other card shares the construct). Any construct shared by two or more cards must be compiled. Every entry states its reason. Card items may add entries (with tests) for one-offs they find, listing them in their report.


## How close can compiling get?

- If every item compiles 85% / 90% / 95% of its texts with faithful behaviour, the paper coverage becomes about 95.6% / 97.1% / 98.5% (simulation over the real card→text map, `assemble.py` stats; cards with texts in several items need all of them).
- The residue after the items (roughly 3–7% of today's unsupported texts) will be a second, smaller long tail: odd wordings the grammar still misses (follow-up items reusing the same pattern files) and further one-offs. Based on the uniqueness scan, I expect the hand-written set to end around 150–300 cards (83 identified now), i.e. 0.5–1% of paper cards.
- Estimate: compiling can reach about **98.5–99.5% of paper cards**; hand-written definitions close the rest to 100% (excluding Steamflogger Boss's Contraption ability). Digital-only cards are outside this metric.

## Files

- `card-items.json` — the items (kind, slug, branch, title, notes), ordered.
- `items/<slug>.md` — briefs; `items/<slug>.texts.tsv` — each item's texts with cards blocked, example card and failing piece (ship these with the briefs); `items/MANUAL.texts.tsv`, `items/OUT_OF_SCOPE.texts.tsv`.
- `clusters.tsv` — every line of unsupported-all.txt: text, cards blocked, item slug / MANUAL / OUT_OF_SCOPE.
- `manual-candidates.tsv` — card, text, why.
