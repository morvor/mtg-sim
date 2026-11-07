# Deferred and out of scope

What the project has decided not to support for now, recorded so it can be picked up
later. Agents: don't implement these. If your work runs into one, add a note here rather
than building it.

## Not supported for now

### Contraptions (Unstable)

Decided 2026-10-01.

- **Scope:** Contraption cards, the assemble action, sprockets, and abilities that refer to
  Contraptions (for example Steamflogger Boss's Contraption ability).
- **Status:** these cards stay unsupported. Contraptions aren't legal in any format, so
  `card-coverage` doesn't count them. `scripts/rulings_batches.py` leaves their rulings
  out of the batches (reported as out of scope).

### Host and augment (Unstable)

Decided 2026-10-01.

- **Scope:** host creatures and augment cards (Scryfall layouts `host` and `augment`), and
  combining them.
- **Status:** same as Contraptions: unsupported, not counted by `card-coverage`, rulings
  left out of the batches.

### Sticker sheets (Unfinity)

Decided 2026-10-01.

- **Scope:** the 50 sticker sheets (type line "Stickers"): sheets as objects a player has
  outside the game, and the {TK} ticket costs for putting their stickers on permanents.
- **Already implemented:** the rules for stickers on permanents (CR 123) are in
  `stickers.rs`. Cards that put stickers on permanents are supported only as far as they
  work without modeling sheets; anything that needs sheets is deferred with them.
- **Status:** sheets are excluded from `card-coverage` (which reports their count
  separately) and from `unsupported` (unless asked for with `--card`). Rulings that apply only to
  sticker sheets are DEFERRED in `scripts/rulings_batches.py`.
- **Cards deferred with the sheets** (noted by `cards-dice-un`, 2026-10-14): every
  sticker a player puts on a permanent comes from one of their sheets (CR 123.2c, 123.3),
  so in a game without sheets nothing ever has a sticker. These cards' sticker abilities
  stay unsupported (the cards count as not fully supported) until sheets are modeled:
  putting stickers ("you may put a name sticker on it", Wizards of the \_\_\_\_\_,
  \_\_\_\_\_ Goblin, Wolf in \_\_\_\_\_ Clothing, Last Voyage of the \_\_\_\_\_,
  \_\_\_\_\_ \_\_\_\_\_ Rocketship, Pin Collection, Roxi, Publicist to the Stars, Done
  for the Day), triggers on placing them (Goblin Airbrusher, Wee Champion, A Good Day to
  Pie, \_\_\_\_\_ Balls of Fire, Make a \_\_\_\_\_ Splash, Tusk and Whiskers, Unlawful
  Entry), and abilities that read the stickers on objects (Fight the \_\_\_\_\_ Fight,
  Ambassador Blorpityblorpboop, Clandestine Chameleon, Scampire, Lineprancers, Proficient
  Pyrodancer, Baaallerina, \_\_\_\_\_ \_\_\_\_\_ \_\_\_\_\_ Trespasser, Sword-Swallowing
  Seraph). Rulings that apply only to these cards are DEFERRED in
  `scripts/rulings_batches.py` (`STICKER_DEFERRED_CARDS`, kept in step with this list).

### Digital-only cards

Decided 2026-10-01.

- **Scope:** cards with no paper printing and digital versions of cards, and their
  mechanics: perpetual, seek, conjure, spellbooks, draft from a spellbook, intensity,
  specialize, boons and similar. That is 855 cards on the 2026-09-24 data: MTG Arena and
  Alchemy cards (hbg, the y-sets, j21, Arena starter and box cards; 657 of them are legal in
  some format, all of them Arena formats: alchemy, historic, timeless, brawl,
  standardbrawl, gladiator, competitivebrawl), MTGO-only cards (the 74 MTGO vanguards and one
  promo) and the old Astral and Sega digital cards (18);
  and digital versions of paper cards: the "A-" rebalanced Alchemy versions (the oracle
  data in `data/` has none as separate cards) and "Name Sticker" Goblin, the MTGO version of
  the Unfinity card _____ Goblin (an MTGO-only record that rolls a d20 instead of using a
  name sticker, yet Scryfall lists it legal in Commander; excluded Oct 28 under the user's
  "ignore ... digital versions of cards", card-support-plan-4.md, "Paper-scope texts",
  decision 1). Its text is in `docs/agents/round4/deferred.tsv`.
- **What counts as paper** (fixed Oct 28, item `paper-scope-1`): every tool uses one rule,
  `ScryfallCard::is_paper()` (`crates/mtg-data/src/scryfall.rs`; `deferred()` in
  `scripts/rulings_batches.py` mirrors it): a card is paper if its oracle record's `games`
  is empty or includes "paper", or it is legal, restricted or banned in a format played
  with paper cards (standard, pioneer, modern, legacy, vintage, pauper, commander,
  oathbreaker, paupercommander, duel, oldschool, premodern, predh), except the digital
  versions in `DIGITAL_VERSIONS` ("Name Sticker" Goblin). The oracle record is
  one printing, and for 1,061 paper cards (City of Traitors, Fastbond, Memnite, Raging
  Goblin, ...) it is an online-only reprint (Masters Edition I-IV, Tempest Remastered,
  Vintage Masters, MTGO duel decks, Arena starter cards, promos). Until then the tools
  looked at `games` only and counted 1,916 digital-only cards, leaving those 1,061 out (and "Name Sticker" Goblin, which the legality rule alone would add).
- **Paper cards with these mechanics** (round-2 card plan, Oct 14): Pearl Collector
  (perpetual) and Tenacious Pup (a boon) are paper cards whose abilities use these
  digital-only mechanics; they stay unsupported with the mechanics
  (`docs/agents/round2/deferred.tsv`). Sigardian Evangel (conjure; legal in Historic,
  Timeless and Brawl) joined them in round 3 (`docs/agents/round3/deferred.tsv`).
- **Status:** `card-coverage` counts paper cards only. The plan for them is
  `docs/agents/items/cards-digital-alchemy.md` (939 unsupported texts), which is not
  scheduled. Rulings that apply only to digital-only cards are DEFERRED in
  `scripts/rulings_batches.py`.

### Cards not legal in any format

Decided 2026-10-02.

- **Scope:** about 2,050 paper cards with no format where they are legal or restricted:
  playtest cards (737), Un-set cards that aren't legal anywhere (474, including the acorn
  cards), planes and phenomena (212), cards from sets that weren't out yet when the data
  was fetched on 2026-09-24 (189, from Star Trek and three Commander products), schemes
  (102), cards banned everywhere (34: the 9 ante cards and 25 conspiracies), vanguards and
  memorabilia. All the dexterity cards but Chaos Orb and Falling Star are in this group
  (those two are legal or restricted in Old School; see "Dexterity cards" below).
- **Already implemented:** the variant and ante rules themselves (Planechase, Archenemy,
  Vanguard, Conspiracy draft, ante; CR 407 and 900-905) are in the engine and tested, and
  stay supported. Only covering these individual cards is deferred.
- **Status:** `card-coverage` and `unsupported` already skip them. Rulings that apply only
  to such cards are DEFERRED in `scripts/rulings_batches.py`. Together with digital-only
  cards, sticker sheets and the dexterity cards (below), that is 1,817 ruling texts on
  Oct 28, after the paper-scope fix counted the 1,061 paper cards with an online-only
  oracle printing as paper (2,520 on Oct 21; 2,636 on Oct 2: a text that a test has cited
  or an exemption has listed since then counts as CITED or EXEMPT instead).

### Dexterity cards

Decided 2026-10-21.

- **Scope:** Chaos Orb and Falling Star, the two dexterity cards that are legal or
  restricted in some format. Their abilities flip the physical card onto the table "from
  a height of at least one foot" and affect what it touches when it lands: Chaos Orb
  destroys the nontoken permanents it touches, Falling Star deals 3 damage to each
  creature it lands on. No rule in the Comprehensive Rules models a physical flip (CR 705
  covers coin flips only). Scryfall's Old School format (`oldschool`) is the only one
  where they are legal or restricted: Chaos Orb is restricted there and Falling Star is
  legal (both are banned in Vintage, Legacy and Commander). Every other card with
  Scryfall's `dexterity` tag (60 names: 54 Un-set cards such as Orcish Paratroopers and
  Landfill, three playtest cards including two planes, two holiday and convention promos,
  and one Secret Lair card) isn't legal in any format and is deferred under "Cards not
  legal in any format" above.
- **Status:** they stay unsupported, and `card-coverage` counts them as unsupported paper
  cards, like Pearl Collector and Tenacious Pup; their texts are in
  `docs/agents/round3/deferred.tsv`. Their rulings about the physical procedure (and Chaos
  Orb's Old School wording) are already exempt in
  `docs/rulings-exemptions/rulings-P136.tsv` and `rulings-P045.tsv`; the others are
  DEFERRED in `scripts/rulings_batches.py`, which finds these cards by the words "from a
  height of at least one foot".

### Conspiracy face-down draft cards

Decided by the user, 2026-10-28.

- **Scope:** five Conspiracy cards whose draft abilities need a simulation of the draft
  table itself, in which each player sees different booster packs: Illusionary Informant
  and Whispergear Sneak (turn the card face down during the draft to look at another
  player's next pick or at an unopened or unwatched booster pack), Archdemon of Paliano
  (draft cards at random without looking at booster packs, then turn it face down), Agent
  of Acquisitions (draft a whole booster pack one card at a time, turn it face down and
  draft nothing more that round) and Deal Broker (immediately after the draft, offer and
  accept card-pool exchanges with the other players). Only a Conspiracy draft (CR 905)
  uses these abilities, and they do nothing once a game starts (CR 905.2). The engine's
  draft model (`draft.rs`) keeps its face-up drafting and "reveal as you draft it"
  support; Conspiracy cards whose draft abilities leave notes that matter in the game
  (Cogwork Tracker, Noble Banneret, Paliano, the High City, ...) stay in scope.
- **Status:** they stay unsupported, and `card-coverage` counts them as unsupported paper
  cards; their draft texts are in `docs/agents/round3/deferred.tsv` and
  `docs/agents/round4/deferred.tsv`. Their other abilities (flying; Deal Broker's
  "{T}: Draw a card, then discard a card.") compile. Rulings that apply only to these
  cards (looking at packs, drafting at random, the exchange procedure) are DEFERRED in
  `scripts/rulings_batches.py`, which finds the cards by name; Deal Broker's ruling on
  drafts that break up into several games was already exempt in
  `docs/rulings-exemptions/rulings-P145.tsv`.

## Deferred to later

### Agents that play well

Decided 2026-10-01: simulation quality comes at the end.

- **Now:** `RandomAgent` (for fuzzing), plus the external decision interface (item
  `agent-api`): what each player can see and every legal option, as JSON, so an outside
  program or model can play any seat.
- **Later:** heuristic, search-based or learned agents for meaningful game results.
  `Game` is `Clone` and `Game::set_agents` swaps agents on a copy, so search agents can
  play ahead.
