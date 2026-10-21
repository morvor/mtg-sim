# Card support, round 3: work plan for the remaining unsupported texts

Generated in the planning item `cards3-plan` (Oct 21) from the release `mtg-tools` and the Scryfall data in `data/`; refreshed in its review after cards2-triggers-delayed landed (see "Review changes"). Round 2: `card-support-plan-2.md`; round 1: `card-support-plan.md`. The inputs, lists and scripts are in `round3/`.

## Baseline

- `mtg-tools card-coverage` (after cards2-triggers-delayed landed): **30,041 / 31,051** paper cards fully supported (96.7%; 32 with hand-written abilities); abilities 57,421 / 58,476. 48 sticker sheets are deferred and not counted.
- Unsupported paper cards: **1,010**, with **1,053** distinct unsupported texts (1,055 card-text lines; `mtg-tools unsupported --paper --by-card`, numbers as N, the card's own name as ~). Snapshot: `round3/baseline.tsv`. (The first snapshot, before cards2-triggers-delayed landed, had 1,087 cards and 1,131 texts; the BASELINE filter counts in the first seven briefs were taken from it.)
- Owned by round-2 items still in progress: **236 texts** — cards2-copies-tokens-counters 131, cards-manual-defs-2 105 (`round3/inflight.tsv`). 220 cards are blocked only by these texts. Not re-planned here. Every other round-1 and round-2 item, and the gap-residual items, have landed: their texts that are still unsupported are planned again below (the briefs name the earlier item per text).
- Remainder (no in-progress owner): **804 texts** (cards3 items, manual, deferred).
- This plan places all 1,053 texts exactly once (`round3/check_plan.py --live` passes against a fresh `mtg-tools unsupported --paper --by-card`): **768** in 8 cards3 items, **6** in cards3-manual-defs, **236** in-flight (above), **13** waiting for an in-flight item's construct (`round3/pending-recheck.tsv`), **30** deferred under DEFERRED.md (`round3/deferred.tsv`). 11 cards have texts with two different owners; none has three.

## Method

1. Snapshot: `mtg-tools unsupported --paper --by-card` (a new flag: one `card<TAB>text` line per unsupported text, with `--raw` the exact text) for the normalized and raw texts; texts owned by in-flight items were matched by normalized text against their `.texts.tsv` (and checked by card name: no in-flight text had changed wording).
2. Probing (`round3/autoprobe.py`, built on `round3/probe.py`, which compiles edited card texts as one-face probe cards in a temporary `MTG_DATA_DIR` with one run of the release mtg-tools): for every remainder text, with the card's other paragraphs kept as context, the cost alone ("COST: Draw a card."), the trigger condition alone (the longest "TRIGGER, draw a card." that compiles, so a compiling intervening if stays with the trigger), the effect sentences added one at a time (the first failing prefix gives the failing sentence), and for modal abilities the header with plain modes and each mode alone. A second pass replaced the failing piece with a filler and probed again: for 440 texts nothing else fails, for 263 another piece fails (column `note`, "then also: ..."; some of these only refer to the replaced words), 26 are table or multi-line cases drilled by hand (d20 results tables row by row, spell-mastery and adamant paragraphs). The failing piece is sentence-level (or cost, trigger, mode, table row); workers drill further with `probe.py`.
3. Families: every text was read with its failing piece and assigned by hand to one of seven parser areas, chosen so that one implementer owns one area and two items rarely edit the same files, and to 5-9 phrase families inside each item (column `family`). The briefs give the counts, representative failing pieces and the engine need per family.
4. Shared-gap check (review): for every family a minimal text with the family's construct was probed on a neutral card (47 probes). Most fail generally (for example "~ deals 2 damage to any target. If a player is dealt damage this way, scry 1.", "Target player who lost life this turn loses 1 life.", "As ~ enters, note your life total.", "Spend this mana only to turn permanents face up.", "Only the chosen creatures can attack this turn.", "Reveal ~ as you draft it."), which confirms the texts share the gap; where the minimal form compiles, the specific failing words were drilled and are named in the brief (Deadly Brew: "another permanent card"; Cemetery Desecrator: "exile another card from a graveyard"; Rally the Horde: "the last card exiled"; Zimone's Hypothesis: "power of the chosen quality"; Cover of Winter and Helmut Zemo: the combination of two pieces that compile alone; d20 tables: the referents of individual rows).
5. One-off check: every failing piece was compared with the Oracle text of all in-scope paper cards (4-word shingles; the 120 texts with the fewest shared shingles were read, and the rarest constructs were searched by phrase). Unique wording is not a one-off when the grammar is shared (Survivor's Med Kit's "loses all rad counters", Combat Celebrant's "hasn't been exerted this turn", Comet's extra loyalty activations, Deadpool's text-box exchange: "text box" appears on 14 paper cards), so those stay in items. Four texts on three cards remain genuine one-offs (Aeon Engine, Jeweled Amulet ×2, Keen-Eared Sentry; the review moved Verrak, Warped Sengir to cards3-triggers-durations because "if life was paid to activate it" is the construct of Ashnod the Uncaring and Way of the Mind Sculptor too), plus the two dexterity cards (Chaos Orb, Falling Star), which need a user decision. All go to cards3-manual-defs.
6. Pending and deferred: 13 texts whose failing piece is a construct that an in-flight round-2 item owns (spell and permanent copies, counter distribution and kinds; the 6 that waited for delayed triggers and trigger grammar were re-checked in the review) wait in `round3/pending-recheck.tsv`; 30 texts fall under DEFERRED.md (sticker sheets 27, digital-only mechanics 2, Contraptions 1).
7. Counts: "touched" = paper cards with at least one of the item's texts; "unlocked alone" = cards whose every unsupported text is in the item. Effort = texts × difficulty factor (1.0 = parser grammar over existing engine support; higher where engine work, shared-core risk or new decisions are expected).

## Work items (ordered by cards unlocked per unit of effort)

| # | Item | Texts | Paper cards touched | Unlocked alone | Factor | Unlocked / effort | Why the factor |
|---:|---|---:|---:|---:|---:|---:|---|
| 1 | cards3-referents-values | 128 | 128 | 127 | 1.1 | 0.90 | mostly Builder grammar (referents, results, values); a few new result records (damage recipients) |
| 2 | cards3-cast-zones | 108 | 107 | 105 | 1.2 | 0.81 | grammar over permissions and dig steps; per-player permission sets and permission riders (CastInfo) |
| 3 | cards3-combat-statics | 69 | 68 | 67 | 1.3 | 0.75 | combat.rs restrictions/requirements and layer-6 ability copying; protection from computed sets |
| 4 | cards3-triggers-durations | 68 | 66 | 66 | 1.3 | 0.75 | trigger causes and event records, delayed triggers bound to referents, "for as long as" durations, activation-cost records; triggers.rs is shared core |
| 5 | cards3-history-conditions | 76 | 76 | 73 | 1.3 | 0.74 | new history records (tapped, fought, counters, zone of origin, damage by source quality, turn-start snapshots) |
| 6 | cards3-followups-choices | 121 | 118 | 115 | 1.3 | 0.73 | per-player values in simultaneous steps, loops, hidden object choices (a new Decision kind and agent protocol) |
| 7 | cards3-costs-mana-replacements | 114 | 112 | 109 | 1.4 | 0.68 | mana provenance, triggered mana abilities, spend restrictions for special actions, mana and draw replacements; costs.rs, mana.rs and replacement.rs are shared core |
| 8 | cards3-game-rules-dice | 84 | 74 | 73 | 1.4 | 0.62 | draft-time hooks in draft.rs, results tables, dice/flip replacements, outside-the-game model decisions |
| 9 | cards3-manual-defs | 6 | 5 | 5 | 2.0 | 0.42 | hand-written definitions with custom hooks; two dexterity cards need a user decision first |

Totals: 774 texts in 9 items (768 in the cards3 items, 6 manual); 740 cards unlocked by single items. 11 cards depend on two owners (ALSO BLOCKED BY in each brief): 6 on a cards3 item and an in-flight item, 4 on two cards3 items (Scheming Fence, Uba Mask, Shared Fate, Sasaya), 1 on in-flight, pending or deferred owners only.

Titles (each brief: `items/<slug>.md`, texts: `items/<slug>.texts.tsv`):

- **cards3-referents-values** — Referent and value grammar IV: results "this way", referents to earlier objects, computed values, "instead"/"otherwise" alternatives, last-known information
- **cards3-cast-zones** — Casting-permission and card-flow grammar III: permissions from among exiled and graveyard cards, each player's permissions, riders on spells cast "this way", alternative costs and keywords for spells, attaching on entry, cards exiled with an object, digs, reveal-until and shuffle-then-reveal moves
- **cards3-combat-statics** — Combat and static grammar IV: attack requirements and taxes, entering attacking or blocking, combat history, additional combats, characteristic statics and granted abilities that name the granter
- **cards3-history-conditions** — History and condition grammar IV: game-history queries, player and object predicates by history, state conditions, filter qualities
- **cards3-followups-choices** — Follow-up and choice grammar III: per-player outcomes, reflexive triggers and modal headers, loops, punishers, object and player choices, notes, hidden choices, votes and villainous choices
- **cards3-costs-mana-replacements** — Cost, mana and replacement grammar IV: cost parts and modifiers, mana production and riders, mana spent and spend restrictions, energy paid, damage prevention, draw and other replacements
- **cards3-game-rules-dice** — Game-rules grammar III: draft-matters cards, dice results and tables, coin flips, Attractions, outside the game and opening hand, commander, loyalty activations, dungeons, emblems, teams, speed and rad counters
- **cards3-triggers-durations** — Trigger and duration grammar IV: trigger events and causes, delayed and "this turn" triggers created by effects, "for as long as" and next-turn durations, triggers that trigger additional times or not at all, costs paid to activate the triggering ability
- **cards3-manual-defs** — Hand-written card definitions III: three one-off cards and the two dexterity cards

## Dependencies and scheduling

- **On in-flight round-2 items**: the 13 pending texts need only cards2-copies-tokens-counters. Six cards have a cards3 text and an in-flight text (Altar of the Pantheon, Suffocation, Deification, River Song: cards-manual-defs-2; Paleontologist's Pick-Axe, Comet, Stellar Pup: cards2-copies-tokens-counters; see ALSO BLOCKED BY).
- **Between cards3 items**: cards3-history-conditions owns the new history records (zone of origin, damage by source quality, first-time-this-turn); cards3-cast-zones' graveyard permissions ("cards in your graveyard that were milled this turn") and cards3-costs-mana-replacements' Kumano/Runesword replacements read them. cards3-costs-mana-replacements owns mana provenance (which source produced each mana); cards3-cast-zones' keyword grants by mana spent (Shadow the Hedgehog, Rain of Riches) and cards3-game-rules-dice's Mr. House read it. cards3-followups-choices adds hidden object choices to decision.rs and the agent protocol (docs/AGENT_PROTOCOL.md, mtg-api).
- **Shared files**: oracle/effects.rs (referents-values, followups-choices: merge often), phrases.rs (history-conditions filters), costs.rs, mana.rs and replacement.rs (costs-mana-replacements), combat.rs and layers.rs (combat-statics), oracle/triggers.rs, triggers.rs and until.rs (triggers-durations; costs-mana-replacements' triggered mana abilities read the same trigger grammar: merge often), draft.rs and dice.rs (game-rules-dice).
- **Suggested waves**: wave 1 — cards3-referents-values, cards3-cast-zones, cards3-history-conditions (history records others read) and cards3-manual-defs (independent; ask the dexterity question first). Wave 2 — cards3-combat-statics, cards3-followups-choices, cards3-costs-mana-replacements, cards3-triggers-durations. Wave 3 — cards3-game-rules-dice (draft hooks; can start any time, it touches few shared files).

## How far this gets

Expected cards supported among today's 1,010 unsupported paper cards, if each planned or in-flight text (items, manual, in-flight, pending re-check) is done with probability p, independently (simulation over the real card-to-text map; deferred texts stay unsupported): p = 85% → about 830; p = 90% → about 881; p = 95% → about 932. 27 cards stay unsupported because of deferred texts (the sticker cards listed in DEFERRED.md, Pearl Collector, Tenacious Pup, Steamflogger Boss). The rest is the long tail again: texts whose second construct the probes didn't reach (280 texts have a noted second piece) and wordings the new grammar still misses. Round 2 found the same: 1,128 of the 1,131 texts unsupported at the first snapshot were in an earlier plan's lists.

## Pending re-check list

13 texts (`round3/pending-recheck.tsv`: cards, text, example_card, pending_item, failing_piece) whose failing construct belongs to a round-2 item still in progress. When that item lands, re-run `mtg-tools unsupported --card "<Name>"`; whatever still fails goes to the cards3 item for its area (or a round-4 list).

- cards2-copies-tokens-counters (13): Activated Sleeper (enter as a copy, with a history filter); Ambitious Augmenter (put ~'s counters on that token); Crashing Wave and Feast of the Victorious Dead (distribute counters among non-targets); Hofri Ghostforge and The Apprentice's Folly (copy exceptions); Jaya's Phoenix (copy the next loyalty ability); Miles Morales (double each kind of counter); Mystic Reflection (enter as copies of the chosen creature); Semester's End and Storyweave (entering with additional counters); Stumpsquall Hydra (distribute among it and commanders); Unbound Flourishing (copy that spell or ability).
- cards2-triggers-delayed (6, re-checked in the review after it landed): Storybook Ride is now supported; Great Train Heist, Kaya, Spirits' Justice, Nuka-Nuke Launcher, Rendmaw, Creaking Nest and The Masamune still fail and are in cards3-triggers-durations.

## Deferred texts

Not in any item (`round3/deferred.tsv`); they fall under decisions already recorded in docs/DEFERRED.md.

- Sticker sheets (27 texts on 24 cards; DEFERRED.md lists every one of these cards as deferred with the sheets): A Good Day to Pie; Ambassador Blorpityblorpboop; Baaallerina; Clandestine Chameleon; Done for the Day; Fight the _____ Fight; Goblin Airbrusher; Last Voyage of the _____; Lineprancers; Make a _____ Splash; Pin Collection (2); Proficient Pyrodancer; Roxi, Publicist to the Stars (2); Scampire; Sword-Swallowing Seraph; Tusk and Whiskers; Unlawful Entry; Wee Champion; Wizards of the _____; Wolf in _____ Clothing; _____ Balls of Fire; _____ Goblin; _____ _____ Rocketship (2); _____ _____ _____ Trespasser.
- Digital-only mechanics: Pearl Collector (perpetual), Tenacious Pup (boon).
- Contraptions: Steamflogger Boss.

Proposed for a user decision (not deferred yet, placed in items with that question): the two dexterity cards (Chaos Orb, Falling Star) in cards3-manual-defs, and five Conspiracy cards whose draft abilities turn them face down or change how packs are drafted (Illusionary Informant, Whispergear Sneak, Archdemon of Paliano, Agent of Acquisitions, Deal Broker) in cards3-game-rules-dice.

## Manual (hand-written) definitions

cards3-manual-defs: **6 texts on 5 cards** — Aeon Engine (reverse turn order; the planes Temple of Atropos and Time Distortion share the words but aren't legal in any format), Jeweled Amulet (2 texts: note the type of mana spent, add mana of the last noted type), Keen-Eared Sentry (a venture limit per opponent per turn), Chaos Orb and Falling Star (dexterity flips; user decision first). Each brief lists the reason; the mechanism (src/cards/, `ManualAbility`, `mtg-tools manual-check`) exists, and cards-manual-defs-2 is still in progress (coordinate on shared hooks).

## Review changes

The independent review (after the first version of this plan) found that cards2-triggers-delayed had landed on the integration branch while the plan was made (merged at 10:34 on Oct 21; the plan's merge of the integration branch brought it in), so its 166 texts were not in flight any more. Changes:

- Snapshot refreshed from the current release mtg-tools: 78 texts (77 of cards2-triggers-delayed, Storybook Ride from the re-check list) are now supported and were dropped from every list; nothing new appeared.
- The 94 texts of cards2-triggers-delayed that are still unsupported (89 from its list, 5 from the re-check list) were probed again (`round3/autoprobe.py`, plus a second pass for the next failing piece) and placed: 67 in the new item cards3-triggers-durations (trigger events and causes, delayed and "this turn" triggers, durations, trigger modifiers), 27 in the existing items by the area of their failing piece (each brief lists them under ADDED IN REVIEW; the rows say "added in review" in column note).
- Verrak, Warped Sengir moved from cards3-manual-defs to cards3-triggers-durations: "if life was paid to activate it" is the construct of Ashnod the Uncaring ("if one or more permanents were sacrificed to activate it") and Way of the Mind Sculptor ("if you removed two or more loyalty counters to activate it"), so it is not a one-off.
- Phosphorescent Feast moved from cards3-costs-mana-replacements (mana-production) to cards3-referents-values (values): it counts mana symbols in revealed cards and produces no mana.
- `round3/check_plan.py --live` now fails when a text in a fresh `mtg-tools unsupported --paper --by-card` is placed nowhere, so the plan is checked against the current compiler and not only against the snapshot.

## Files

- `items/cards3-<slug>.md` — the briefs; `items/cards3-<slug>.texts.tsv` — each item's texts (cards blocked, normalized text, example card, piece kind, failing piece, family, previous item, note), grouped by family.
- `round3/baseline.tsv` — the snapshot (card, text) the plan was made from.
- `round3/inflight.tsv`, `round3/pending-recheck.tsv`, `round3/deferred.tsv` — the texts outside the cards3 items (see above).
- `round3/check_plan.py` — checks that every snapshot text is placed exactly once across the cards3 item lists and the three lists, with matching card counts; `--live` also compares the snapshot with the current release mtg-tools (texts supported or new since).
- `round3/probe.py` — compile edited card texts (`--card NAME --text TEXT`, or `--file` with one JSON probe per line) with the release mtg-tools; `round3/autoprobe.py` — the component probing used for the failing pieces.
