# Card support, round 2: work plan for the remaining unsupported texts

Generated in the planning workflow from the release `mtg-tools` and the Scryfall data in `data/`; the inputs are the round-2 classification shards (every unplanned unsupported text with its failing piece, family and kind). Round 1: `card-support-plan.md`.

## Baseline

- Unsupported paper cards (sticker sheets excluded): **2,349**; distinct unsupported texts on them: **2,452** (`mtg-tools unsupported --paper --by-card`, numbers as N, the card's own name as ~). 48 sticker-sheet lines were skipped.
- Owned by round-1 items still in progress: **598 texts** (cards-history-conditions 213, cards-game-rules 145, cards-dice-un 116, cards-filters-noun-phrases 68, cards-mana 56). Not re-planned here.
- Round-1 manual candidates still unsupported: **56 texts**. Remainder (no item yet): **1,798 texts on 1,739 cards**; 1,717 of those cards are blocked only by remainder texts.
- This plan places all **1,854** texts (1,798 remainder + 56 manual candidates) exactly once (`check_plan.py` passes): **1,490** in 11 cards2 items, **109** in cards-manual-defs-2, **77** owned by gap-residual-1 (`round2/gap-owned.tsv`), **175** waiting for a round-1 item (`round2/pending-recheck.tsv`), **3** deferred (`round2/deferred.tsv`).

## Method

1. Classification (four shards of about 450 texts): each text was probed with the release compiler on one-card data directories, one component at a time (cost, trigger, intervening if, then sentences added one by one; variants with the value, condition, duration or object phrase simplified), to find the failing piece. Each text got a family label, a kind (grammar: the engine can do it, the parser can't; engine: an engine capability is missing; one-off; pending: a round-1 item in progress owns the construct; deferred: DEFERRED.md), the engine need, a second family for pending texts, and notes with the relevant rulings.
2. Reconciliation: the four shards used 1,266 different family labels. Every text was read and mapped to one of twelve areas chosen so that one implementer owns one parser area and two items rarely edit the same files; two small areas were merged into their neighbours (token creation into copies-tokens-counters, durations into triggers-delayed) and the goad family moved to combat. Inside each item the texts were grouped into 4-8 phrase families by hand (`fam/<slug>.py`), which the briefs list with counts, representative failing pieces and the engine need.
3. Pending texts: 37 pending texts whose second construct belongs to a cards2 area went to that item with "also needs <round-1 item>"; the 175 without a second construct are in `round2/pending-recheck.tsv` (re-check once that item lands).
4. One-off re-check: every one-off claim (210) and every round-1 manual candidate (56) was compared with all in-scope paper cards (rare 5-word shingles of the normalized text, then reading the matches) and with the families of the other texts. A construct that another card shares must be compiled, so 125 claims and 32 candidates moved to items (each brief lists them with the card that shares the construct), to gap-residual-1 or to the pending list; 85 one-offs and 24 candidates stay manual (cards-manual-defs-2, each with its reason).
5. Gap-owned: every card name mentioned in `gap-residual-1.md` (whitespace-normalized, so names broken across lines count) was matched by script against the cards of every remainder text; 11 matches were excluded after reading (cards the brief names only as already-compiled examples or rule references: Aeon Engine, Council Guardian, Custodi Squire, Expropriate, Selvala's Stampede, Split Decision, Trial of a Time Lord, Trial // Error, Two-Headed Giant, Paliano, the High City, Regicide). Result: 77 texts on 72 cards, tagged R1-R7.
6. Review (adversarial pass before the workers start): the one-off claims were searched against the card data again and 10 more moved out of the manual list (Invasion Plans, Dragon Hunter, Tromokratis, Portal Manipulator, Tenuous Truce and Wall of Caltrops to cards2-combat; Cover of Winter to cards2-costs-replacements; Deadpool, Trading Card to cards2-statics-characteristics; Goblin Game to cards2-choices-players; Falling Star to the pending list with Chaos Orb's dexterity flip, owned by cards-dice-un). 40 random texts and 15 more that looked mis-grouped were re-probed on clones of their cards: the failing pieces held except for five texts regrouped by what actually fails (Plague Reaver to cards2-zones-dig: "under that player's control"; The Eleventh Hour to cards2-copies-tokens-counters: a copy exception; Sauron, Lord of the Rings to cards2-followups: a comma series; Concussive Bolt to cards2-choices-players: "that player or that planeswalker's controller" inside an object phrase; Whirlwind Denial to cards2-filters-conditions: "each spell and ability"), and Comet, Stellar Pup's third result joined its siblings in cards2-copies-tokens-counters. Rule citations in the briefs were checked against the CR text and corrected where they pointed to the wrong subrule.
7. Counts: the card-to-text map comes from the by-card dump. "Touched" = paper cards with at least one of the item's texts; "unlocked alone" = cards whose every unsupported text belongs to the item and none of them also needs a round-1 item. Effort = texts x difficulty factor (1.0 = parser grammar over existing engine support; higher where engine work, shared-core risk or new decisions are expected).

## Work items (ordered by cards unlocked per unit of effort)

| # | Item | Texts | Paper cards touched | Unlocked alone | Factor | Unlocked / effort | Why the factor |
|---:|---|---:|---:|---:|---:|---:|---|
| 1 | cards2-followups | 132 | 133 | 128 | 1.0 | 0.97 | grammar over existing control flow (May, PayOptional, RepeatProcess, Reflexive, per-player frames); 4 engine texts |
| 2 | cards2-zones-dig | 125 | 126 | 120 | 1.0 | 0.96 | grammar on DigStep/search; a few new DigStep stops and selections |
| 3 | cards2-filters-conditions | 133 | 131 | 125 | 1.0 | 0.94 | grammar; phrases.rs is shared with round-1 cards-filters-noun-phrases |
| 4 | cards2-statics-characteristics | 104 | 104 | 98 | 1.1 | 0.86 | grammar plus the normalizer (affects many cards) and layers for objects outside the battlefield |
| 5 | cards2-referents-values | 207 | 205 | 192 | 1.1 | 0.84 | grammar plus recording the results of every primitive and last-known attachments |
| 6 | cards2-combat | 89 | 89 | 86 | 1.2 | 0.81 | 16 engine texts in combat.rs (unblocking, per-creature taxes, player-level requirements) |
| 7 | cards2-costs-replacements | 113 | 112 | 104 | 1.2 | 0.77 | 20 engine texts; costs.rs and replacement.rs are shared core files |
| 8 | cards2-copies-tokens-counters | 144 | 138 | 130 | 1.2 | 0.75 | 25 engine texts (copy targets, token substitution, untargeted distribution, static copy effects) |
| 9 | cards2-triggers-delayed | 175 | 173 | 159 | 1.3 | 0.70 | 58 engine texts: new Duration kinds and event causes; partly needs nothing but grammar |
| 10 | cards2-cast-permissions | 116 | 119 | 105 | 1.3 | 0.70 | 25 engine texts and the gap-residual-1 R7 dependency |
| 11 | cards2-choices-players | 152 | 134 | 129 | 1.4 | 0.61 | 46 engine texts: hidden choices, guesses, bids and per-player choices need new Decision kinds and agent-protocol changes |
| 12 | cards-manual-defs-2 | 109 | 106 | 94 | 2.0 | 0.43 | hand-written definitions with custom hooks, one test file per card |

Totals: 1,599 texts in 12 items (1,490 in the cards2 items, 109 manual); 1,470 cards unlocked by single items (a card counted in one item only when all its texts are there). 56 unsupported cards depend on two owners and 1 on three (round-1 items, gap-residual-1 and the cards2 items together).

Titles (each brief: `items/<slug>.md`, texts: `items/<slug>.texts.tsv`):

- **cards2-followups** — Follow-up grammar: "if [player] does/doesn't/can't", per-player results, "instead" alternatives, repeat loops, punishers and reflexive triggers
- **cards2-zones-dig** — Card-flow grammar IV: dig selections and the rest, reveal-until variants, Gifts and same-name searches, other players' hands and libraries, library positions, attach-on-entry moves
- **cards2-filters-conditions** — Target, filter and condition grammar III: variable and compound targets, relational filters against referents, noun-phrase qualities, comparison/parity/count conditions
- **cards2-statics-characteristics** — Characteristic and static grammar II: characteristic riders and becomes-bundles, type changes outside the battlefield, conditional and copied keywords, rules-modifying statics, granter-named grants, name normalization
- **cards2-referents-values** — Referent and value grammar III: "that many", "the difference", objects affected "this way" or paid as costs, batch and per-player referents, last-known attachments, aggregates over groups, X snapshots
- **cards2-combat** — Combat grammar III: damage assignment as though unblocked and unblocking, attack/block requirements, restrictions and taxes, scoped additional combats and entering attacking/blocking, goad durations, combat referents
- **cards2-costs-replacements** — Cost and replacement grammar III: activation cost parts and restrictions, X caps and cost choices, cost modifiers, regeneration and die riders, damage/prevention replacements, draw/counter/life/enter replacements, as-enters processing
- **cards2-copies-tokens-counters** — Copy, token and counter grammar II: spell copies and retargeting, copy exceptions and static copy effects, token replacements, entering with counters, counter kinds and moves
- **cards2-triggers-delayed** — Trigger grammar III and durations: delayed triggers from resolving abilities, durations bound to other players and objects, event causes, trigger limits
- **cards2-cast-permissions** — Casting permissions II: riders on spells cast "this way", keywords granted to spells, next-spell casting options, permissions over zones and players, casts during resolution, spending rules
- **cards2-choices-players** — Choice and player grammar II: hidden choices, guesses and bidding, votes, per-player stored choices, multi-value choices and notes, choices while casting, object choices by several players, player phrases
- **cards-manual-defs-2** — Hand-written card definitions II: 106 one-off cards (109 ability texts) on the existing src/cards/ mechanism

## Texts owned by gap-residual-1 (not in a cards2 item)

gap-residual-1 (`items/gap-residual-1.md`, already assigned to a worker) owns the cards it lists. Their remainder texts are in `round2/gap-owned.tsv` (columns: cards, text, example_card, gap): **77 texts** on 72 cards (70 unlocked when that item is done). cards2 items must leave these texts alone.

- R1 (control changes for several players at once): 6 texts — Aminatou, the Fateshifter; Cultural Exchange; Inniaz, the Gale Force; Perplexing Chimera; Scrambleverse; Sudden Substitution
- R1,R2 (both R1 and R2): 1 text — Order of Succession
- R2 (seats: left/right, nearest opponent in a direction): 7 texts — Barroom Brawl; Grenzo's Rebuttal; Mystic Barrier; Pramikon, Sky Rampart; Teyo, Geometric Tactician
- R3 (abilities granted with the granting object's name; names in quotes): 22 texts — Animal Friend; Archery Training; Blinding Powder; Deathpact Angel; Dire Flail // Dire Blunderbuss; Fishing Pole; Grothama, All-Devouring; Gutter Grime; Hankyu; Hellish Rebuke; Nettlevine Blight; Ozox, the Clattering King; Rekindling Phoenix; Sengir Nosferatu; Shifting Shadow; Shuriken; Spare Dagger; Surestrike Trident; The Dominion Bracelet; Toralf, God of Fury // Toralf's Hammer; Torrent of Lava; Trickster's Talisman
- R4 (activation terms on gained abilities): 7 texts — Drana and Linvala; Grell Philosopher; Mairsil, the Pretender; Nathan Drake, Treasure Hunter; Scheming Fence
- R5 (open-ended loops and per-player totals): 8 texts — Hypergenesis; Kamahl's Summons; Liege of the Hollows; Plague of Vermin; Struggle for Sanity; Tainted Pact; Thieves' Auction; Timesifter
- R6 (piles of exiled cards, face-down and several piles): 16 texts — Abstract Performance; Become Anonymous; Bend or Break; Boneyard Parley; Brilliant Ultimatum; Expose the Culprit; Hostile Negotiations; Make an Example; Mangara's Tome; Parallel Thoughts; Sauron's Ransom; The Celestial Toymaker; Triumph of Saint Katherine; Whims of the Fates
- R7 (spells cast "this way": effects tied to the permission used): 10 texts — Alaundo the Seer; Apple of Eden, Isu Relic; Fires of Mount Doom; Galea, Kindler of Hope; Havengul Lich; Ogre Battlecaster; Share the Spoils; Strago and Relm; The Ruinous Powers; Thundermane Dragon

Some of these texts also contain a construct outside the gap (for example Toralf's front face "the excess" damage, Ogre Battlecaster's "When you cast that spell"): the gap worker should hand those back to the cards2 item that owns the construct (cards2-referents-values, cards2-cast-permissions) instead of leaving them unsupported.

## Dependencies and scheduling

- **On gap-residual-1** (engine pieces that items build on, not their texts): cards2-cast-permissions (the "this way" rider family needs R7: CastInfo::permission and on-cast modifications in PlayTerms); cards2-statics-characteristics (granter-named grants need R3; the per-ability "only once each turn" on gained abilities, The Enigma Jewel, needs R4); cards2-followups (per-player numbers kept across loop passes: R5); cards2-zones-dig (per-player piles: Stand or Fall builds on R6); cards2-choices-players (Reins of Power's mutual control swap needs R1). Schedule those families after the gap pieces land, or coordinate.
- **On round-1 items still in progress**: 37 texts in cards2 items also need a round-1 item (cards-history-conditions 19, cards-filters-noun-phrases 7, cards-game-rules 4, cards-dice-un 4, cards-mana 3); the briefs list them under ALSO NEEDS. The 175 pending re-check texts below need only their round-1 item.
- **Shared files**: phrases.rs (cards2-filters-conditions with round-1 cards-filters-noun-phrases), oracle/triggers.rs (cards2-triggers-delayed), costs.rs and replacement.rs (cards2-costs-replacements; cards2-copies-tokens-counters adds a token-substitution action to replacement.rs), the normalizer in oracle/mod.rs (cards2-statics-characteristics: re-run card-coverage after changing it), combat.rs (cards2-combat), CastInfo in object.rs (cards2-referents-values snapshots, cards2-costs-replacements choices, gap-residual-1 R7 permission), decision.rs and the agent protocol (cards2-choices-players adds hidden-choice, guess and bid decisions: update docs/AGENT_PROTOCOL.md and mtg-api).
- **Durations**: cards2-triggers-delayed owns the new Duration kinds ("during [player]'s next turn", "until that player's next end step", while-conditions on other objects); cards2-combat (attack requirements during a next turn) and cards2-cast-permissions (permission windows) use them.
- **Suggested waves**: wave 1 — cards2-followups, cards2-zones-dig, cards2-filters-conditions, cards2-referents-values (mostly grammar, high ratio; referents-values lands the result recording others reuse) and cards-manual-defs-2 (independent). Wave 2 — cards2-statics-characteristics, cards2-combat, cards2-costs-replacements, cards2-copies-tokens-counters, cards2-triggers-delayed. Wave 3 — cards2-cast-permissions (after gap-residual-1 R7) and cards2-choices-players (new decisions and agent protocol).

## How far this gets

Expected cards supported among today's 2,349 unsupported paper cards, assuming the round-1 items in progress finish their own texts and each round-2 text (items, manual, gap-owned, pending re-check) is done with probability p, independently (simulation over the real card-to-text map): p = 85% → about 2,070; p = 90% → about 2,160; p = 95% → about 2,250. Three cards stay unsupported because of deferred texts (Steamflogger Boss, Pearl Collector, Tenacious Pup). The rest is a third, much smaller long tail: odd wordings the new grammar still misses and further one-offs found while implementing.

## Pending re-check list

175 texts whose only failing construct belongs to a round-1 item still in progress (`round2/pending-recheck.tsv`: cards, text, example_card, pending_item). When that item lands, re-run `mtg-tools unsupported --card "<Name>"` for these cards; whatever still fails goes to the matching cards2 item (or a round-3 follow-up).

### cards-history-conditions (107 texts)

- Activated Sleeper: "You may have ~ enter as a copy of any creature card in a graveyard that was put there from the battlefield this turn, except it's a Phyrexian in ..."
- Admiral Beckett Brass: "At the beginning of your end step, gain control of target nonland permanent controlled by a player who was dealt combat damage by three or more ..."
- Aegar, the Freezing Flame: "Whenever a creature or planeswalker an opponent controls is dealt excess damage, if a Giant, Wizard, or spell you controlled dealt damage to it this ..."
- Alania, Divergent Storm: "Whenever you cast a spell, if it's the first instant spell, the first sorcery spell, or the first Otter spell other than ~ you've cast this turn, you ..."
- Alchemist's Talent: "{N}{R}: Level N / Whenever you cast a spell, if mana from a Treasure was spent to cast it, ~ deals damage equal to that spell's mana value to each ..."
- Angelic Arbiter: "Each opponent who attacked with a creature this turn can't cast spells."
- Angelic Arbiter: "Each opponent who cast a spell this turn can't attack with creatures."
- Anowon, the Ruin Thief: "Whenever one or more Rogues you control deal combat damage to a player, that player mills a card for each N damage dealt to them. If the player mills ..."
- Antagonism: "At the beginning of each player's end step, ~ deals N damage to that player unless one of their opponents was dealt damage this turn."
- Backdraft: "Choose a player who cast one or more sorcery spells this turn. ~ deals damage to that player equal to half the damage dealt by one of those sorcery ..."
- Banon, the Returners' Leader: "Pray — Once during each of your turns, you may cast a creature spell from among cards in your graveyard that were put there from anywhere other than ..."
- Bat Colony: "When ~ enters, create a N/N black Bat creature token with flying for each mana from a Cave spent to cast it."
- Blaster Hulk: "~ costs {N} less to cast for each {E} you've paid or lost this turn."
- Blitzball: "GOOOOAAAALLL! — {T}, Sacrifice ~: Draw two cards. Activate only if an opponent was dealt combat damage by a legendary creature this turn."
- Bonecache Overseer: "{T}, Pay N life: Draw a card. Activate only if three or more cards left your graveyard this turn or if you've sacrificed a Food this turn."
- Boseiju, Who Shelters All: "{T}, Pay N life: Add {C}. If that mana is spent on an instant or sorcery spell, that spell can't be countered."
- Boxing Ring: "{T}: Create a Treasure token. Activate only if you control a creature that fought this turn."
- Brass's Tunnel-Grinder // Tecutlan, the Searing Rift: "Whenever you cast a permanent spell using mana produced by ~, discover X, where X is that spell's mana value."
- Cache Grab: "Mill four cards. You may put a permanent card from among the cards milled this way into your hand. If you control a Squirrel or returned a Squirrel ..."
- Call the Spirit Dragons: "At the beginning of your upkeep, for each color, put a +N/+N counter on a Dragon you control of that color. If you put +N/+N counters on five Dragons ..."
- Captain America, Living Legend: "Whenever a creature you control becomes tapped during your turn, if it's the first time that creature has become tapped this turn, untap it."
- Captain N'ghathrod: "At the beginning of your end step, choose target artifact or creature card in an opponent's graveyard that was put there from their library this ..."
- Cataclysmic Prospecting: "~ deals X damage to each creature. For each mana from a Desert spent to cast ~, create a tapped Treasure token."
- Cathedral Membrane: "When ~ dies during combat, it deals N damage to each creature it blocked this combat."
- Chandra, Fire of Kaladesh // Chandra, Roaring Flame: "{T}: ~ deals N damage to target player or planeswalker. If ~ has dealt N or more damage this turn, exile her, then return her to the battlefield ..."
- Check for Traps: "Target opponent reveals their hand. You choose a nonland card from it. Exile that card. If an instant card or a card with flash is exiled this way, ..."
- Churning Reservoir: "{N}, {T}: Create a N/N red Phyrexian Goblin creature token. Activate only if an oil counter was removed from a permanent you controlled this turn or ..."
- Coin of Mastery: "Each creature you control enters with an additional +N/+N counter on it for each mana from an artifact source spent to cast it."
- Combat Celebrant: "If ~ hasn't been exerted this turn, you may exert it as it attacks. When you do, untap all other creatures you control and after this phase, there is ..."
- Command the Stage: "At the beginning of each upkeep, if an opponent was dealt noncombat damage last turn, return ~ from your graveyard to your hand."
- Deadly Brew: "Each player sacrifices a creature or planeswalker of their choice. If you sacrificed a permanent this way, you may return another permanent card from ..."
- Demonic Junker: "When ~ enters, for each player, destroy up to one target creature that player controls. If a creature you controlled was destroyed this way, put two ..."
- Diseased Vermin: "At the beginning of your upkeep, ~ deals X damage to target opponent previously dealt damage by it, where X is the number of infection counters on it."
- Dream Salvage: "Draw cards equal to the number of cards target opponent discarded this turn."
- Emrakul, the World Anew: "Flying, protection from spells and from permanents that were cast this turn"
- Eye of Duskmantle: "You may play lands and cast spells from among cards in your graveyard you've surveilled this turn. If you cast a spell this way, you pay life equal ..."
- Faramir, Prince of Ithilien: "At the beginning of your end step, choose an opponent. At the beginning of that player's next end step, you draw a card if they didn't attack you ..."
- Flood of Tears: "Return all nonland permanents to their owners' hands. If you return four or more nontoken permanents you control this way, you may put a permanent ..."
- Fold into Aether: "Counter target spell. If that spell is countered this way, its controller may put a creature card from their hand onto the battlefield."
- Genesis of the Daleks: "{CHAPTER} IV — Target opponent faces a villainous choice — Destroy all Dalek creatures and each of your opponents loses life equal to the total power ..."
- Ghost of Ramirez DePietro: "Whenever ~ deals combat damage to a player, choose up to one target card in a graveyard that was discarded or put there from a library this turn. Put ..."
- Gimli, Mournful Avenger: "~ has indestructible as long as two or more creatures died under your control this turn."
- Grim Reminder: "Search your library for a nonland card and reveal it. Each opponent who cast a spell this turn with the same name as that card loses N life. Then ..."
- Grim Repriser: "{B}{R}: Return ~ from your graveyard to the battlefield with a finality counter on it. Activate only if an opponent has been dealt noncombat damage ..."
- Halls of Mist: "Creatures that attacked during their controller's last turn can't attack."
- Haunting Imitation: "Each player reveals the top card of their library. For each creature card revealed this way, create a token that's a copy of that card, except it's ..."
- Hope of Ghirapur: "Sacrifice ~: Until your next turn, target player who was dealt combat damage by ~ this turn can't cast noncreature spells."
- Idol of Oblivion: "{T}: Draw a card. Activate only if you created a token this turn."
- Impact Resonance: "~ deals X damage divided as you choose among any number of target creatures, where X is the greatest amount of damage dealt by a source to a ..."
- Insubordination: "At the beginning of the end step of enchanted creature's controller, ~ deals N damage to that player unless that creature attacked this turn."
- Jabari's Influence: "Gain control of target nonartifact, nonblack creature that attacked you this turn and put a -N/-N counter on it."
- Kagha, Shadow Archdruid: "Once during each of your turns, you may play a land or cast a permanent spell from among cards in your graveyard that were put there from your ..."
- Kalain, Reclusive Painter: "Other creatures you control enter with an additional +N/+N counter on them for each mana from a Treasure spent to cast them."
- Knights of the Black Rose: "Whenever an opponent becomes the monarch, if you were the monarch as the turn began, that player loses N life and you gain N life."
- Kumano Faces Kakkazan // Etching of Kumano: "If a creature dealt damage this turn by a source you controlled would die, exile it instead."
- Kutzil's Flanker: "When ~ enters, choose one — / • Put a +N/+N counter on ~ for each creature that left the battlefield under your control this turn. / • You gain N ..."
- Ludevic, Necro-Alchemist: "At the beginning of each player's end step, that player may draw a card if a player other than you lost life this turn."
- Lurker: "~ can't be the target of spells unless it attacked or blocked this turn."
- Master's Guide-Mural // Master's Manufactory: "{T}: Create a N/N white and blue Golem artifact creature token. Activate only if ~ or another artifact entered the battlefield under your control ..."
- Mila, Crafty Companion // Lukka, Wayward Bonder: "+N: You may discard a card. If you do, draw a card. If a creature card was discarded this way, draw two cards instead."
- Mr. Foxglove: "Whenever ~ attacks, draw cards equal to the number of cards in defending player's hand minus the number of cards in your hand. If you didn't draw ..."
- Mysterious Limousine: "Whenever ~ enters or attacks, exile up to one other target creature until ~ leaves the battlefield. If a creature is put into exile this way, return ..."
- Neriv, Crackling Vanguard: "Whenever ~ attacks, exile a number of cards from the top of your library equal to the number of differently named tokens you control. During any turn ..."
- Neyali, Suns' Vanguard: "Whenever one or more tokens you control attack a player, exile the top card of your library. During any turn you attacked with a token, you may play ..."
- Oblivious Bookworm: "At the beginning of your end step, you may draw a card. If you do, discard a card unless a permanent entered the battlefield face down under your ..."
- Ojer Axonil, Deepest Might // Temple of Power: "{N}{R}, {T}: Transform ~. Activate only if red sources you controlled dealt N or more noncombat damage this turn and only as a sorcery."
- Once Upon a Time: "If ~ is the first spell you've cast this game, you may cast it without paying its mana cost."
- Papalymo Totolymo: "{N}, {T}, Sacrifice ~: Each opponent who lost life this turn sacrifices a creature with the greatest power among creatures they control."
- Play with Fire: "~ deals N damage to any target. If a player is dealt damage this way, scry N."
- Power Surge: "At the beginning of each player's upkeep, ~ deals X damage to that player, where X is the number of untapped lands they controlled at the beginning ..."
- Premature Burial: "Destroy target nonblack creature that entered since your last turn ended."
- Primeval Spawn: "If ~ would enter and it wasn't cast or no mana was spent to cast it, exile it instead."
- Proctor of Potential: "{W}{U}: Return ~ from your graveyard to the battlefield with a finality counter on it. Activate only if you've scried or surveilled this turn."
- Puppet Master: "When enchanted creature dies, return that card to its owner's hand. If that card is returned to its owner's hand this way, you may pay {U}{U}{U}. If ..."
- Raul, Trouble Shooter: "Once during each of your turns, you may cast a spell from among cards in your graveyard that were milled this turn."
- Rise of the Witch-king: "Each player sacrifices a creature of their choice. If you sacrificed a creature this way, you may return another permanent card from your graveyard ..."
- Rix Maadi Guildmage: "{B}{R}: Target player who lost life this turn loses N life."
- Robber of the Rich: "Whenever ~ attacks, if defending player has more cards in hand than you, exile the top card of their library. During any turn you attacked with a ..."
- Rock Jockey: "You can't cast ~ if you've played a land this turn."
- Rowdy Crew: "When ~ enters, draw three cards, then discard two cards at random. If two cards that share a card type are discarded this way, put two +N/+N counters ..."
- Satoru, the Infiltrator: "Whenever ~ and/or one or more other nontoken creatures you control enter, if none of them were cast or no mana was spent to cast them, draw a card."
- Screaming Nemesis: "Whenever ~ is dealt damage, it deals that much damage to any other target. If a player is dealt damage this way, they can't gain life for the rest of ..."
- Sea Troll: "{U}: Regenerate ~. Activate only if ~ blocked or was blocked by a blue creature this turn."
- Sengir, the Dark Baron: "Whenever another player loses the game, you gain life equal to that player's life total as the turn began."
- Shaile, Dean of Radiance // Embrose, Dean of Shadow: "{T}: Put a +N/+N counter on each creature that entered the battlefield under your control this turn."
- Sidequest: Catch a Fish // Cooking Campsite: "At the beginning of your upkeep, look at the top card of your library. If it's an artifact or creature card, you may reveal it and put it into your ..."
- Sonic Shrieker: "When ~ enters, it deals N damage to any target and you gain N life. If a player is dealt damage this way, they discard a card."
- Sparring Dummy: "{T}: Mill a card. You may put a land card milled this way into your hand. You gain N life if a Lesson card is milled this way."
- Specter's Shriek: "Target opponent reveals their hand. You may choose a nonland card from it. If you do, that player exiles that card. If a nonblack card is exiled this ..."
- Sphinx's Tutelage: "Whenever you draw a card, target opponent mills two cards. If two nonland cards that share a color were milled this way, repeat this process."
- Spoils of the Hunt: "Target creature you control gets +N/+N until end of turn for each mana from a Treasure that was spent to cast ~. Then that creature deals damage ..."
- Stalwart Successor: "Whenever one or more counters are put on a creature you control, if it's the first time counters have been put on that creature this turn, put a ..."
- Stomping Slabs: "Reveal the top seven cards of your library, then put those cards on the bottom of your library in any order. If a card named ~ was revealed this way, ..."
- Storrev, Devkarin Lich: "Whenever ~ deals combat damage to a player or planeswalker, return to your hand target creature or planeswalker card in your graveyard that wasn't ..."
- Suffocation: "Cast ~ only if you were dealt damage this turn by a red instant or sorcery spell."
- Tainted Specter: "{N}{B}{B}, {T}: Target player discards a card unless they put a card from their hand on top of their library. If that player discards a card this ..."
- Tangled Colony: "When ~ dies, create X N/N black Rat creature tokens with "~ can't block", where X is the amount of damage dealt to it this turn."
- Taster of Wares: "When ~ enters, target opponent reveals X cards from their hand, where X is the number of Goblins you control. You choose one of those cards. That ..."
- Thaumaton Torpedo: "{N}, {T}, Sacrifice ~: Destroy target nonland permanent. This ability costs {N} less to activate if you attacked with a Spacecraft this turn."
- Thousand Moons Smithy // Barracks of the Thousand: "Whenever you cast an artifact or creature spell using mana produced by ~, create a white Gnome Soldier artifact creature token with "~'s power and ..."
- Tunnel Ignus: "Whenever a land enters under an opponent's control, if that player had another land enter the battlefield under their control this turn, ~ deals N ..."
- Unstable Glyphbridge // Sandswirl Wanderglyph: "Each opponent who attacked you or a planeswalker you control this turn can't cast spells."
- Vault 101: Birthday Party: "{CHAPTER} II, III — You may put an Aura or Equipment card from your hand or graveyard onto the battlefield. If an Equipment is put onto the ..."
- Vazi, Keen Negotiator: "Whenever an opponent casts a spell or activates an ability, if mana from a Treasure was spent to cast it or activate it, put a +N/+N counter on ..."
- Weathered Sentinels: "~ can attack players who attacked you during their last turn as though it didn't have defender."
- Wicked Akuba: "{B}: Target player dealt damage by ~ this turn loses N life."
- Wreck Hunter: "When ~ enters, choose target player. You create a tapped Powerstone token for each nonland card in that player's graveyard that was put there from ..."

### cards-mana (24 texts)

- Astral Cornucopia: "{T}: Choose a color. Add one mana of that color for each charge counter on ~."
- Charmed Pendant: "{T}, Mill a card: For each colored mana symbol in the milled card's mana cost, add one mana of that color."
- Cheering Crowd: "At the beginning of each player's first main phase, that player may put a +N/+N counter on ~. If they do, they add {C} for each counter on it."
- Creeping Peeper: "{T}: Add {U}. Spend this mana only to cast an enchantment spell, unlock a door, or turn a permanent face up."
- Damping Sphere: "If a land is tapped for two or more mana, it produces {C} instead of any other type and amount."
- False Dawn: "Until end of turn, spells and abilities you control that would add colored mana instead add that much white mana. Until end of turn, you may spend ..."
- Fangorn, Tree Shepherd: "Whenever one or more Treefolk you control attack, add twice that much {G}."
- Glissa Sunseeker: "{T}: Destroy target artifact if its mana value is equal to the amount of unspent mana you have."
- Hall of Gemstone: "At the beginning of each player's upkeep, that player chooses a color. Until end of turn, lands tapped for mana produce mana of the chosen color ..."
- Harvest Mage: "{G}, {T}, Discard a card: Until end of turn, if you tap a land for mana, it produces one mana of a color of your choice instead of any other type and ..."
- Incubation Druid: "{T}: Add one mana of any type that a land you control could produce. If ~ has a +N/+N counter on it, add three mana of that type instead."
- Obsidian Charmaw: "~ costs {N} less to cast for each land your opponents control that could produce {C}."
- Omnath, Locus of Mana: "~ gets +N/+N for each unspent green mana you have."
- Overgrown Zealot: "{T}: Add two mana of any one color. Spend this mana only to turn permanents face up."
- Pulse of Llanowar: "If a basic land you control is tapped for mana, it produces mana of a color of your choice instead of any other type."
- Qarsi Deceiver: "{T}: Add {C}. Spend this mana only to cast a face-down creature spell, pay a mana cost to turn a manifested creature face up, or pay a morph cost."
- Sasaya, Orochi Ascendant // Sasaya's Essence: "Whenever a land you control is tapped for mana, add an additional one mana of any type that land produced for each other land you control with the ..."
- Security Rhox: "You may pay {R}{G} rather than pay ~'s mana cost. Spend only mana produced by Treasures to cast it this way."
- Smoky Lounge // Misty Salon: "When you unlock this door, create an X/X blue Spirit creature token with flying, where X is the number of unlocked doors among Rooms you control."
- Spoils of Evil: "For each artifact or creature card in target opponent's graveyard, add {C} and you gain N life."
- Thieving Varmint: "{T}, Pay N life: Add two mana of any one color. Spend this mana only to cast spells you don't own."
- Thran Turbine: "At the beginning of your upkeep, you may add {C}{C}. This mana can't be spent to cast spells."
- Three Tree City: "{N}, {T}: Choose a color. Add an amount of mana of that color equal to the number of creatures you control of the chosen type."
- Tin Street Gossip: "{T}: Add {R}{G}. Spend this mana only to cast face-down spells or to turn creatures face up."

### cards-filters-noun-phrases (21 texts)

- Altar of the Pantheon: "{T}: Add one mana of any color. If you control a God, a Demigod, or a legendary enchantment, you gain N life."
- Ashling's Prerogative: "Each creature with mana value of the chosen quality has haste."
- Bello, Bard of the Brambles: "During your turn, each non-Equipment artifact and non-Aura enchantment you control with mana value N or greater is a N/N Elemental creature in ..."
- Conjured Currency: "At the beginning of your upkeep, you may exchange control of ~ and target permanent you neither own nor control."
- Descent into Madness: "At the beginning of your upkeep, put a despair counter on ~, then each player exiles X permanents they control and/or cards from their hand, where X ..."
- Desdemona, Freedom's Edge: "Whenever ~ attacks, target creature card in your graveyard that's an artifact or that has mana value N or less gains escape until end of turn. The ..."
- Endless Detour: "The owner of target spell, nonland permanent, or card in a graveyard puts it on their choice of the top or bottom of their library."
- Eternal Flame: "~ deals X damage to target opponent or planeswalker and half X damage, rounded up, to you, where X is the number of Mountains you control."
- Gollum, Riddle Master: "Whenever an opponent casts a spell with mana value of the chosen quality, choose one that hasn't been chosen — / • Put a +N/+N counter on ~. / • Each ..."
- Grand Master of Flowers: "+N: Target creature without first strike, double strike, or vigilance can't attack or block until your next turn."
- Mechtitan Core: "{N}, Exile ~ and four other artifact creatures and/or Vehicles you control: Create Mechtitan, a legendary N/N Construct artifact creature token with ..."
- Miles Morales // Ultimate Spider-Man: "Whenever you attack, double the number of each kind of counter on each Spider and legendary creature you control."
- Nashi, Moon's Legacy: "Whenever ~ attacks, exile up to one target legendary or Rat card from your graveyard and copy it. You may cast the copy."
- Nicol Bolas, Dragon-God: "+N: You draw a card. Each opponent exiles a card from their hand or a permanent they control."
- Rift Elemental: "{N}{R}, Remove a time counter from a permanent you control or suspended card you own: ~ gets +N/+N until end of turn."
- Sanwell, Avenger Ace: "Whenever ~ becomes tapped, exile the top six cards of your library. You may cast a Vehicle or artifact creature spell from among them. Then put the ..."
- Soldevi Adnate: "{T}, Sacrifice a black or artifact creature: Add an amount of {B} equal to the sacrificed creature's mana value."
- Twins of Discord: "Whenever you attack, choose odd or even. Creatures with mana value of that quality can't block this turn."
- Vision, Spectral Synthezoid: "Once during each of your turns, you may cast a noncreature or Robot spell from your hand without paying its mana cost."
- Volcanic Offering: "~ deals N damage to target creature you don't control and N damage to target creature of an opponent's choice you don't control."
- Zimone's Hypothesis: "You may put a +N/+N counter on a creature. Then choose odd or even. Return each creature with power of the chosen quality to its owner's hand."

### cards-game-rules (16 texts)

- Aether Spike: "Choose target spell. You get {E}{E} , then you may pay any amount of {E}. Counter that spell unless its controller pays {N} for each {E} paid this ..."
- Animus of Predation: "If you removed a creature card with flying from the draft with cards named ~, ~ has flying. The same is true for first strike, double strike, ..."
- Cogwork Grinder: "~ enters with X +N/+N counters on it, where X is the number of cards you removed from the draft with cards named ~."
- Cogwork Tracker: "~ attacks a player you noted for cards named ~ each combat if able."
- Extrapolate the Impossible: "You may reveal exactly two cards you own with different names from outside the game. An opponent chooses one of them. You put that card into your ..."
- Game Over: "~ costs {N} less to cast if a player's life total is less than or equal to half their starting life total."
- Izzet Generatorium: "{T}: Draw a card. Activate only if you've paid or lost four or more {E} this turn."
- Moraug, Fury of Akoum: "Landfall — Whenever a land you control enters, if it's your main phase, there's an additional combat phase after this phase. At the beginning of that ..."
- Noble Banneret: "As long as you control one or more creatures with a name you noted for cards named ~, ~ and those creatures get +N/+N and have lifelink."
- Paliano Vanguard: "Other creatures you control of a type you noted for cards named ~ get +N/+N."
- Path of Ancestry: "{T}: Add one mana of any color in your commander's color identity. When that mana is spent to cast a creature spell that shares a creature type with ..."
- Smuggler Captain: "When ~ enters, you may search your library for a card with a name you noted for cards named ~, reveal it, put it into your hand, then shuffle."
- Spire Phantasm: "When ~ enters, if you guessed correctly for a card named ~, draw a card."
- Study Hall: "{N}, {T}: Add one mana of any color. When you spend this mana to cast your commander, scry X, where X is the number of times it's been cast from the ..."
- Turtles Forever: "Search your library and/or outside the game for exactly four legendary creature cards you own with different names, then reveal those cards. An ..."
- War Room: "{N}, {T}, Pay life equal to the number of colors in your commanders' color identity: Draw a card."

### cards-dice-un (5 texts)

- Circuits Act: "Roll three six-sided dice. For each different result, create a N/N white Clown Robot artifact creature token."
- Clay Golem: "{N}, Roll a d8: Monstrosity X, where X is the result."
- Falling Star: "Flip ~ onto the playing area from a height of at least one foot. ~ deals N damage to each creature it lands on. Tap all creatures dealt damage by ~. If ~ ..." (Chaos Orb's dexterity flip, in cards-dice-un: whatever that item does with Chaos Orb, model or defer, applies here)
- Farideh, Devil's Chosen: "Dark One's Own Luck — Whenever you roll one or more dice, ~ gains flying and menace until end of turn. If any of those results was N or higher, draw ..."
- Pair o' Dice Lost: "Roll two six-sided dice. Return any number of cards with total mana value X or less from your graveyard to your hand, where X is the total of those ..."

### cards-filters-noun-phrases, cards-mana (1 text)

- Meeting of the Five: "Exile the top ten cards of your library. You may cast spells with exactly three colors from among them this turn. Add {W}{W}{U}{U}{B}{B}{R}{R}{G}{G}. ..."

### cards-history-conditions, cards-game-rules (1 text)

- Long Rest: "Return X target cards with different mana values from your graveyard to your hand. If eight or more cards were returned to your hand this way, your ..."

## Deferred texts

Not in any item (`round2/deferred.tsv`); they fall under decisions already recorded in docs/DEFERRED.md.

| Card | Text | Category |
|---|---|---|
| Pearl Collector | {N}{W}: Another target creature perpetually gains lifelink. | digital-only mechanic: perpetual (DEFERRED.md, Digital-only cards) |
| Steamflogger Boss | If a Rigger you control would assemble a Contraption, it assembles two Contraptions instead. | Contraptions (DEFERRED.md, out of scope) |
| Tenacious Pup | When ~ enters the battlefield, you gain N life. You get a one-time boon with "When you cast a creature spell, that creature enters the battlefield with an additional +N/+N counter, trample counter, and vigilance counter on it." | digital-only mechanic: boon (DEFERRED.md, Digital-only cards) |

Pearl Collector and Tenacious Pup are paper printings that use digital-only mechanics (perpetual, boons); their other texts are planned (Pearl Collector's "triggers only once" in cards2-triggers-delayed) but the cards stay unsupported until those mechanics are taken up. Conjure, also listed in DEFERRED.md, was implemented for its paper printings (kw/conjure.rs), so the user may want the same for these two; until then they stay deferred. Falling Star (a dexterity card, legal only in Old School) waits in the pending list with Chaos Orb, whose identical flip cards-dice-un owns: if that item defers Chaos Orb, record both in DEFERRED.md together.

## Manual (hand-written) definitions

cards-manual-defs-2: **109 texts on 106 cards** (94 of these cards need nothing else): 85 one-offs from the round-2 classification that survived the re-check and 24 round-1 manual candidates still unsupported. The brief lists every text with its reason and what the definition needs; the mechanism (src/cards/, `ManualAbility`, `mtg-tools manual-check`) landed in round 1 with 31 cards.

- static rules and continuous replacements (custom static / replacement hooks) (31 cards): Abandoned Sarcophagus; Altar of the Pantheon; Ashiok, Wicked Manipulator; Autumn's Veil; Bewitching Leechcraft; Bludgeon Brawl; Deification; Don't Blink; Fear of Sleep Paralysis; Haakon, Stromgald Scourge; Irencrag Feat; Jared Carthalion, True Heir; Liesa, Shroud of Dusk; Melira, the Living Cure; Panglacial Wurm; Piracy; River Song; Rootwater Shaman; Sanctum Lurker; Shaman's Trance; Susan Foreman; Swirl the Mists; Twinning Staff; Yurlok of Scorch Thrash; Zabaz, the Glimmerwasp; Undergrowth; Heirloom Epic; Snowblind; Retether; Soul Sculptor; Pyramids
- combat rules (custom combat hooks) (13 cards): Become the Pilot; Camouflage; Canal Courier; Cunning Giant; General Jarkeld; Johan; Pygmy Hippo; Raging River; Season of the Witch; Suppressor Skyguard; Tahngarth, First Mate; The Aetherspark; Winter's Chill
- triggers on unique events or with unique conditions (20 cards): Bomb Squad; Booby Trap; Grip of Chaos; Inferno of the Star Mounts; Kodama of the East Tree; Lagrella, the Magpie; Lifeline; Market Gnome; Psychic Battle; Quakebringer; Skophos Maze-Warden; The Temporal Anchor; Tyvar the Bellicose; Unbound Flourishing; The Celestial Toymaker; Suffocation; Undiscovered Paradise; Sylvan Library; Ertai's Meddling; Dimensional Breach
- one-shot effects with unique procedures (42 cards): Biomancer's Familiar; Call Damage Control; Cruel Entertainment; Equinox; Ezuri's Predation; Fireball; Flaming Gambit; Gale, Waterdeep Prodigy; Game of Chaos; Gilded Drake; Gluntch, the Bestower; Goblin Welder; Hand of Vecna; Hedron Alignment; Herald of Leshrac; Homura, Human Ascendant // Homura's Essence; Illusion of Choice; Kaya, Ghost Assassin; Magar of the Magic Strings; Master of the Wild Hunt; Mathemagics; Memories Returning; Minamo's Meddling; Multiple Choice; Niv-Mizzet Reborn; Oubliette; Phyrexian Splicer; Psychic Rebuttal; Raiding Party; Shambling Swarm; Shrouded Lore; Soul Echo; Sphinx Ambassador; Starlight Spectacular; Temporal Aperture; The Book of Vile Darkness; The Chain Veil; The Face of Boe; The Mimeoplasm; Triple Triad; Vodalian War Machine; Volcano Hellion

Moved out of the manual list (125 one-off claims and 32 round-1 manual candidates whose construct another card shares; the cards2 briefs name the card that shares each one):
- cards2-cast-permissions: 8 texts: Aminatou's Augury; Chiss-Goria, Forge Tyrant; Jorn, God of Winter // Kaldring, the Rimestaff; Maralen, Fae Ascendant; Mavinda, Students' Advocate; Shared Fate; Urianger Augurelt; Xanathar, Guild Kingpin
- cards2-choices-players: 28 texts: Alhammarret, High Arbiter; Archangel of Strife; Bell Borca, Spectral Sergeant; Bitter Feud; Celestial Judgment; Círdan the Shipwright; Day of the Moon; Dream Tides; Goblin Game; Greymond, Avacyn's Stalwart; Illusionary Terrain; Infernal Denizen; Long List of the Ents; Null Chamber; Prisoner's Dilemma; Psychic Paper; Rite of Ruin; Selective Obliteration; Storage Matrix; Stronghold Gambit; Teferi's Realm; Volo, Itinerant Scholar; Wheel of Misfortune
- cards2-combat: 19 texts: Dragon Hunter; Galactus, Devourer of Worlds; Glyph of Delusion; Glyph of Reincarnation; Heat Wave; Icatian Skirmishers; Illusionist's Gambit; Invasion Plans; Kemba's Legion; Okk; Portal Manipulator; Seeker of Slaanesh; Serene Sleuth; Sewers of Estark; Tenuous Truce; Total War; Tromokratis; Wall of Caltrops
- cards2-copies-tokens-counters: 14 texts: Benthic Anomaly; Blue, Loyal Raptor; Crystalline Giant; Cursed Mirror; Esix, Fractal Bloom; Gluttonous Hellkite; Goldberry, River-Daughter; Hulking Metamorph; Lathiel, the Bounteous Dawn; Sarpadian Empires, Vol. VII; Sin, Unending Cataclysm; Stangg, Echo Warrior; Vesuvan Shapeshifter; Zevlor, Elturel Exile
- cards2-costs-replacements: 23 texts: Baba Lysaga, Night Witch; Baron Helmut Zemo; Cosmic Intervention; Cover of Winter; Drought; Enduring Angel // Angelic Enforcer; Guile; Martyrdom; Merseine; Moxite Refinery; Ojer Axonil, Deepest Might // Temple of Power; Phyrexian Dreadnought; Professor Hojo; Protective Sphere; Rain of Gore; Reed Richards, Smartest Man; Rocket Launcher; Shared Fate; The Capitoline Triad; Thick-Skinned Goblin; Titania, Rugged Rumbler; Unpredictable Cyclone
- cards2-filters-conditions: 3 texts: C.A.M.P.; Helm of Kaldra; Magewright's Stone
- cards2-followups: 8 texts: Equipoise; Lim-Dûl's Vault; Oath of Ghouls; Queen Kayla bin-Kroog; Rally the Horde; Research // Development; Rottenmouth Viper; Worms of the Earth
- cards2-referents-values: 8 texts: Cemetery Prowler; Central Elevator // Promising Stairs; Odric, Blood-Cursed; Omnath, Locus of All; Savage Punch; Unforge; Volatile Chimera
- cards2-statics-characteristics: 10 texts: Ashiok, Dream Render; Deadpool, Trading Card; Honest Work; Old-Growth Troll; Omo, Queen of Vesuva; Outlaws' Merriment; Pardic Firecat; Princess Yue; Rebbec, Architect of Ascension; Teferi, Who Slows the Sunset
- cards2-triggers-delayed: 16 texts: Ashnod the Uncaring; Brazen Cannonade; Calix, Destiny's Hand; Erdwal Illuminator; Firesong and Sunspeaker; Foe-Razer Regent; Font of Agonies; Giant Oyster; Historian's Boon; Hushbringer; Nadu, Winged Wisdom; Niko, Light of Hope; Sacred Ground; Selfless Squire; Way of the Mind Sculptor; Widespread Panic
- cards2-zones-dig: 7 texts: Bruna, Light of Alabaster; Dream Harvest; Enchantment Alteration; Grifter's Blade; Myriad Landscape; Pyxis of Pandemonium; Warp World
- gap-residual-1 (R3): 2 texts: Shuriken; Torrent of Lava
- gap-residual-1 (R4): 1 text: Mairsil, the Pretender
- gap-residual-1 (R5): 3 texts: Struggle for Sanity; Tainted Pact; Timesifter
- gap-residual-1 (R6): 1 text: Whims of the Fates
- gap-residual-1 (R7): 1 text: Havengul Lich
- pending re-check (cards-dice-un): 1 text: Falling Star
- pending re-check (cards-history-conditions): 2 texts: Diseased Vermin; Suffocation
- pending re-check (cards-mana): 2 texts: Damping Sphere; Omnath, Locus of Mana

## Files

- `items/<slug>.md` — the briefs; `items/<slug>.texts.tsv` — each item's texts (cards blocked, normalized text, example card, piece kind, failing piece), grouped by phrase family within each card count.
- `round2/gap-owned.tsv`, `round2/pending-recheck.tsv`, `round2/deferred.tsv` (in `docs/agents/round2/`) — the texts outside the items (see above).
- `check_plan.py` (planning directory) — checks that every remainder and manual-candidate text is placed exactly once across the item tsv files and the three lists.
