# Card support, round 3: work plan for the remaining unsupported texts

Generated in the planning item `cards3-plan` (Oct 21) from the release `mtg-tools` and the Scryfall data in `data/`; refreshed in its review after cards2-triggers-delayed landed (see "Review changes") and again after cards2-copies-tokens-counters landed (see "Refresh after cards2-copies-tokens-counters landed"). Round 2: `card-support-plan-2.md`; round 1: `card-support-plan.md`. The inputs, lists and scripts are in `round3/`.

## Baseline

- `mtg-tools card-coverage` (after cards2-triggers-delayed and cards2-copies-tokens-counters landed): **30,152 / 31,051** paper cards fully supported (97.1%; 32 with hand-written abilities); abilities 57,548 / 58,487. 48 sticker sheets are deferred and not counted.
- Unsupported paper cards: **899**, with **937** distinct unsupported texts (939 card-text lines; `mtg-tools unsupported --paper --by-card`, numbers as N, the card's own name as ~). Snapshot: `round3/baseline.tsv`. (The first snapshot, before cards2-triggers-delayed landed, had 1,087 cards and 1,131 texts; the second, before cards2-copies-tokens-counters landed, 1,010 cards and 1,053 texts. The BASELINE filter counts in the briefs were taken from these earlier snapshots.)
- Owned by a round-2 item still in progress: **103 texts** of cards-manual-defs-2 (`round3/inflight.tsv`). 96 cards are blocked only by these texts. Not re-planned here. Every other round-1 and round-2 item, and the gap-residual items, have landed: their texts that are still unsupported are planned again below (the briefs name the earlier item per text).
- Remainder (no in-progress owner): **834 texts** (cards3 items, manual, deferred).
- This plan places all 937 texts exactly once (`round3/check_plan.py --live` passes against a fresh `mtg-tools unsupported --paper --by-card`): **798** in 8 cards3 items, **6** in cards3-manual-defs, **103** in flight (above), **0** waiting for an in-flight item's construct (`round3/pending-recheck.tsv` is empty: everything in it waited for cards2-copies-tokens-counters), **30** deferred under DEFERRED.md (`round3/deferred.tsv`). 9 cards have texts with two different owners; none has three.

## Method

1. Snapshot: `mtg-tools unsupported --paper --by-card` (a new flag: one `card<TAB>text` line per unsupported text, with `--raw` the exact text) for the normalized and raw texts; texts owned by in-flight items were matched by normalized text against their `.texts.tsv` (and checked by card name: no in-flight text had changed wording).
2. Probing (`round3/autoprobe.py`, built on `round3/probe.py`, which compiles edited card texts as one-face probe cards in a temporary `MTG_DATA_DIR` with one run of the release mtg-tools): for every remainder text, with the card's other paragraphs kept as context, the cost alone ("COST: Draw a card."), the trigger condition alone (the longest "TRIGGER, draw a card." that compiles, so a compiling intervening if stays with the trigger), the effect sentences added one at a time (the first failing prefix gives the failing sentence), and for modal abilities the header with plain modes and each mode alone. A second pass replaced the failing piece with a filler and probed again: for 440 texts nothing else fails, for 263 another piece fails (column `note`, "then also: ..."; some of these only refer to the replaced words), 26 are table or multi-line cases drilled by hand (d20 results tables row by row, spell-mastery and adamant paragraphs). The failing piece is sentence-level (or cost, trigger, mode, table row); workers drill further with `probe.py`.
3. Families: every text was read with its failing piece and assigned by hand to one of seven parser areas, chosen so that one implementer owns one area and two items rarely edit the same files, and to 5-9 phrase families inside each item (column `family`). The briefs give the counts, representative failing pieces and the engine need per family.
4. Shared-gap check (review): for every family a minimal text with the family's construct was probed on a neutral card (47 probes). Most fail generally (for example "~ deals 2 damage to any target. If a player is dealt damage this way, scry 1.", "Target player who lost life this turn loses 1 life.", "As ~ enters, note your life total.", "Spend this mana only to turn permanents face up.", "Only the chosen creatures can attack this turn.", "Reveal ~ as you draft it."), which confirms the texts share the gap; where the minimal form compiles, the specific failing words were drilled and are named in the brief (Deadly Brew: "another permanent card"; Cemetery Desecrator: "exile another card from a graveyard"; Rally the Horde: "the last card exiled"; Zimone's Hypothesis: "power of the chosen quality"; Cover of Winter and Helmut Zemo: the combination of two pieces that compile alone; d20 tables: the referents of individual rows).
5. One-off check: every failing piece was compared with the Oracle text of all in-scope paper cards (4-word shingles; the 120 texts with the fewest shared shingles were read, and the rarest constructs were searched by phrase). Unique wording is not a one-off when the grammar is shared (Survivor's Med Kit's "loses all rad counters", Combat Celebrant's "hasn't been exerted this turn", Comet's extra loyalty activations, Deadpool's text-box exchange: "text box" appears on 14 paper cards), so those stay in items. Four texts on three cards remain genuine one-offs (Aeon Engine, Jeweled Amulet ×2, Keen-Eared Sentry; the review moved Verrak, Warped Sengir to cards3-triggers-durations because "if life was paid to activate it" is the construct of Ashnod the Uncaring and Way of the Mind Sculptor too), plus the two dexterity cards (Chaos Orb, Falling Star), which need a user decision. All go to cards3-manual-defs.
6. Pending and deferred: 13 texts whose failing piece was a construct that an in-flight round-2 item owned (spell and permanent copies, counter distribution and kinds) waited in `round3/pending-recheck.tsv` until cards2-copies-tokens-counters landed (the 6 that waited for delayed triggers and trigger grammar were re-checked in the review); all 19 have been re-checked and placed, and the list is empty; 30 texts fall under DEFERRED.md (sticker sheets 27, digital-only mechanics 2, Contraptions 1).
7. Counts: "touched" = paper cards with at least one of the item's texts; "unlocked alone" = cards whose every unsupported text is in the item. Effort = texts × difficulty factor (1.0 = parser grammar over existing engine support; higher where engine work, shared-core risk or new decisions are expected).

## Work items (ordered by cards unlocked per unit of effort)

| # | Item | Texts | Paper cards touched | Unlocked alone | Factor | Unlocked / effort | Why the factor |
|---:|---|---:|---:|---:|---:|---:|---|
| 1 | cards3-referents-values | 133 | 133 | 132 | 1.1 | 0.90 | mostly Builder grammar (referents, results, values); a few new result records (damage recipients) |
| 2 | cards3-cast-zones | 108 | 107 | 105 | 1.2 | 0.81 | grammar over permissions and dig steps; per-player permission sets and permission riders (CastInfo) |
| 3 | cards3-triggers-durations | 72 | 70 | 70 | 1.3 | 0.75 | trigger causes and event records, delayed triggers bound to referents, "for as long as" durations, activation-cost records; triggers.rs is shared core |
| 4 | cards3-history-conditions | 83 | 83 | 80 | 1.3 | 0.74 | new history records (tapped, fought, counters, zone of origin, damage by source quality, turn-start snapshots) |
| 5 | cards3-combat-statics | 73 | 72 | 70 | 1.3 | 0.74 | combat.rs restrictions/requirements and layer-6 ability copying; protection from computed sets; static copy effects (layer 1) |
| 6 | cards3-followups-choices | 124 | 121 | 118 | 1.3 | 0.73 | per-player values in simultaneous steps, loops, hidden object choices (a new Decision kind and agent protocol) |
| 7 | cards3-costs-mana-replacements | 118 | 116 | 113 | 1.4 | 0.68 | mana provenance, triggered mana abilities, spend restrictions for special actions, mana, draw and one-shot entering replacements; costs.rs, mana.rs and replacement.rs are shared core |
| 8 | cards3-game-rules-dice | 87 | 74 | 74 | 1.4 | 0.61 | draft-time hooks in draft.rs, results tables, dice/flip replacements, outside-the-game model decisions |
| 9 | cards3-manual-defs | 6 | 5 | 5 | 2.0 | 0.42 | hand-written definitions with custom hooks; two dexterity cards need a user decision first |

Totals: 804 texts in 9 items (798 in the cards3 items, 6 manual); 767 cards unlocked by single items. 9 cards depend on two owners (ALSO BLOCKED BY in each brief): 4 on a cards3 item and cards-manual-defs-2 (Deification, River Song, Suffocation, Unbound Flourishing), 5 on two cards3 items (Paleontologist's Pick-Axe, Sasaya, Scheming Fence, Shared Fate, Uba Mask).

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

- **On in-flight round-2 items**: only cards-manual-defs-2 is still in flight. Four cards have a cards3 text and a text it owns (Deification, River Song, Suffocation, Unbound Flourishing; see ALSO BLOCKED BY).
- **Between cards3 items**: cards3-history-conditions owns the new history records (zone of origin, damage by source quality, first-time-this-turn); cards3-cast-zones' graveyard permissions ("cards in your graveyard that were milled this turn") and cards3-costs-mana-replacements' Kumano/Runesword replacements read them. cards3-costs-mana-replacements owns mana provenance (which source produced each mana); cards3-cast-zones' keyword grants by mana spent (Shadow the Hedgehog, Rain of Riches) and cards3-game-rules-dice's Mr. House read it. cards3-followups-choices adds hidden object choices to decision.rs and the agent protocol (docs/AGENT_PROTOCOL.md, mtg-api).
- **Shared files**: oracle/effects.rs (referents-values, followups-choices: merge often), phrases.rs (history-conditions filters), costs.rs, mana.rs and replacement.rs (costs-mana-replacements), combat.rs and layers.rs (combat-statics), oracle/triggers.rs, triggers.rs and until.rs (triggers-durations; costs-mana-replacements' triggered mana abilities read the same trigger grammar: merge often), draft.rs and dice.rs (game-rules-dice).
- **Suggested waves**: wave 1 — cards3-referents-values, cards3-cast-zones, cards3-history-conditions (history records others read) and cards3-manual-defs (independent; ask the dexterity question first). Wave 2 — cards3-combat-statics, cards3-followups-choices, cards3-costs-mana-replacements, cards3-triggers-durations. Wave 3 — cards3-game-rules-dice (draft hooks; can start any time, it touches few shared files).

## How far this gets

Expected cards supported among today's 899 unsupported paper cards, if each planned or in-flight text (items, manual, in-flight) is done with probability p, independently (expectation over the real card-to-text map; deferred texts stay unsupported): p = 85% → about 737; p = 90% → about 782; p = 95% → about 827. 27 cards stay unsupported because of deferred texts (the sticker cards listed in DEFERRED.md, Pearl Collector, Tenacious Pup, Steamflogger Boss). The rest is the long tail again: texts whose second construct the probes didn't reach (157 texts in the cards3 items have a noted second piece in column note) and wordings the new grammar still misses. Round 2 found the same: 1,128 of the 1,131 texts unsupported at the first snapshot were in an earlier plan's lists.

## Pending re-check list

Empty now (`round3/pending-recheck.tsv` keeps its header: cards, text, example_card, pending_item, failing_piece). It held 19 texts whose failing construct belonged to a round-2 item still in progress; both of those items have landed and every text was re-checked:

- cards2-triggers-delayed (6, re-checked in the review): Storybook Ride is now supported; Great Train Heist, Kaya, Spirits' Justice, Nuka-Nuke Launcher, Rendmaw, Creaking Nest and The Masamune still fail and are in cards3-triggers-durations.
- cards2-copies-tokens-counters (13, re-checked in the refresh after it landed): Crashing Wave and Feast of the Victorious Dead are now supported; the other 11 still fail and are placed by their failing piece (see "Refresh after cards2-copies-tokens-counters landed").

## Deferred texts

Not in any item (`round3/deferred.tsv`); they fall under decisions already recorded in docs/DEFERRED.md.

- Sticker sheets (27 texts on 24 cards; DEFERRED.md lists every one of these cards as deferred with the sheets): A Good Day to Pie; Ambassador Blorpityblorpboop; Baaallerina; Clandestine Chameleon; Done for the Day; Fight the _____ Fight; Goblin Airbrusher; Last Voyage of the _____; Lineprancers; Make a _____ Splash; Pin Collection (2); Proficient Pyrodancer; Roxi, Publicist to the Stars (2); Scampire; Sword-Swallowing Seraph; Tusk and Whiskers; Unlawful Entry; Wee Champion; Wizards of the _____; Wolf in _____ Clothing; _____ Balls of Fire; _____ Goblin; _____ _____ Rocketship (2); _____ _____ _____ Trespasser.
- Digital-only mechanics: Pearl Collector (perpetual), Tenacious Pup (boon).
- Contraptions: Steamflogger Boss.

Proposed for a user decision (not deferred yet, placed in items with that question): the two dexterity cards (Chaos Orb, Falling Star) in cards3-manual-defs, and five Conspiracy cards whose draft abilities turn them face down or change how packs are drafted (Illusionary Informant, Whispergear Sneak, Archdemon of Paliano, Agent of Acquisitions, Deal Broker) in cards3-game-rules-dice.

## Manual (hand-written) definitions

cards3-manual-defs: **6 texts on 5 cards** — Aeon Engine (reverse turn order; the planes Temple of Atropos and Time Distortion share the words but aren't legal in any format), Jeweled Amulet (2 texts: note the type of mana spent, add mana of the last noted type), Keen-Eared Sentry (a venture limit per opponent per turn), Chaos Orb and Falling Star (dexterity flips; user decision first). Each brief lists the reason; the mechanism (src/cards/, `ManualAbility`, `mtg-tools manual-check`) exists, and cards-manual-defs-2 is still in progress (coordinate on shared hooks). The refresh after cards2-copies-tokens-counters landed added no manual claims: every leftover text's failing piece is shared grammar.

## Review changes

The independent review (after the first version of this plan) found that cards2-triggers-delayed had landed on the integration branch while the plan was made (merged at 10:34 on Oct 21; the plan's merge of the integration branch brought it in), so its 166 texts were not in flight any more. Changes:

- Snapshot refreshed from the current release mtg-tools: 78 texts (77 of cards2-triggers-delayed, Storybook Ride from the re-check list) are now supported and were dropped from every list; nothing new appeared.
- The 94 texts of cards2-triggers-delayed that are still unsupported (89 from its list, 5 from the re-check list) were probed again (`round3/autoprobe.py`, plus a second pass for the next failing piece) and placed: 67 in the new item cards3-triggers-durations (trigger events and causes, delayed and "this turn" triggers, durations, trigger modifiers), 27 in the existing items by the area of their failing piece (each brief lists them under ADDED IN REVIEW; the rows say "added in review" in column note).
- Verrak, Warped Sengir moved from cards3-manual-defs to cards3-triggers-durations: "if life was paid to activate it" is the construct of Ashnod the Uncaring ("if one or more permanents were sacrificed to activate it") and Way of the Mind Sculptor ("if you removed two or more loyalty counters to activate it"), so it is not a one-off.
- Phosphorescent Feast moved from cards3-costs-mana-replacements (mana-production) to cards3-referents-values (values): it counts mana symbols in revealed cards and produces no mana.
- `round3/check_plan.py --live` now fails when a text in a fresh `mtg-tools unsupported --paper --by-card` is placed nowhere, so the plan is checked against the current compiler and not only against the snapshot.

## Refresh after cards2-copies-tokens-counters landed

cards2-copies-tokens-counters landed on the integration branch at the same time as this plan (the plan's merge brought it in), so its 131 in-flight texts and the 13 re-check texts that waited for it were stale. Changes:

- Snapshot refreshed from the current release mtg-tools (`round3/baseline.tsv`): 116 texts are now supported and were dropped from every list — 110 of cards2-copies-tokens-counters' list, 2 of the re-check list (Crashing Wave, Feast of the Victorious Dead), 2 of cards-manual-defs-2's list (Altar of the Pantheon's devotion text, Ashiok, Wicked Manipulator) and 2 cards3 texts (Bloodthirsty Adversary in cards3-followups-choices, Altar of the Pantheon in cards3-history-conditions). Nothing new appeared.
- The 32 texts that are still unsupported (21 from its list, 11 from the re-check list) were probed again (`round3/autoprobe.py`, a second pass with the failing piece replaced by a filler, and about 100 drill probes on neutral cards to find the failing words). They are 32, not 40+, and their failing pieces are spread over several areas, so no new item: each went to the existing cards3 item for the area of its failing piece. Each affected brief lists them under ADDED AFTER CARDS2-COPIES-TOKENS-COUNTERS LANDED with the drilled failing words; the rows say "added after cards2-copies-tokens-counters landed" in column note, with "drilled: ...".
  - cards3-history-conditions 8: zone-history 2 (Activated Sleeper, The Fourteenth Doctor: "card in a graveyard that was put there from ... this turn" as the object of "enter as a copy of"), damage-history 1 (Shelob), state-conditions 2 (Reflecting Mirror "if that target is you", Unbound Flourishing "mana cost ... contains {X}"), filters 3 (Chef's Kiss, The Tale of Tamiyo, Miles Morales).
  - cards3-referents-values 5: referents 4 (Hofri Ghostforge, Spirit of Resilience, Semester's End, Ulalek), last-known 1 (Ambitious Augmenter "put ~'s counters on that token").
  - cards3-combat-statics 4, new family `copy-effects`: static copy effects (Metamorphic Alteration, Paleontologist's Pick-Axe) and copy exceptions the grammar misses (Hulking Metamorph, The Apprentice's Folly).
  - cards3-costs-mana-replacements 4: other-replacements 3 (Mystic Reflection and Storyweave: "the next time one or more ... enter this turn" one-shot entering replacements; Ludevic: a second sentence in an "As ~ transforms" replacement), mana-spent 1 (The Everflowing Well: "using mana produced by ~").
  - cards3-followups-choices 4, object-choices: Boltbender, Feather, Stumpsquall Hydra, Vesuvan Shapeshifter (choosing sets of stack objects or untargeted objects; "As ~ enters, you may choose another creature").
  - cards3-triggers-durations 4: trigger-events 1 (Imodane), delayed-effect-triggers 3 (Dynaheir, Zevlor, Jaya's Phoenix: "when you next activate/cast ... this turn").
  - cards3-game-rules-dice 3, dice-tables: Comet, Stellar Pup's die-result rows: the row form "1 or 2 — ..." / "3 — ..." (without "|") fails even with plain rows, then the loyalty notation "[+2]"/"[−1]" (its fourth row was already in the item).
- `round3/inflight.tsv` now lists only cards-manual-defs-2 (103 texts); `round3/pending-recheck.tsv` is empty.

## Files

- `items/cards3-<slug>.md` — the briefs; `items/cards3-<slug>.texts.tsv` — each item's texts (cards blocked, normalized text, example card, piece kind, failing piece, family, previous item, note), grouped by family.
- `round3/baseline.tsv` — the snapshot (card, text) the plan was made from.
- `round3/inflight.tsv`, `round3/pending-recheck.tsv`, `round3/deferred.tsv` — the texts outside the cards3 items (see above).
- `round3/check_plan.py` — checks that every snapshot text is placed exactly once across the cards3 item lists and the three lists, with matching card counts; `--live` also compares the snapshot with the current release mtg-tools (texts supported or new since).
- `round3/probe.py` — compile edited card texts (`--card NAME --text TEXT`, or `--file` with one JSON probe per line) with the release mtg-tools; `round3/autoprobe.py` — the component probing used for the failing pieces.
