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
  Seraph).

### Digital-only cards

Decided 2026-10-01.

- **Scope:** the 1,916 cards with no paper printing (MTG Arena/Alchemy and other
  digital-only cards) and their mechanics: perpetual, seek, conjure, spellbooks, draft from
  a spellbook, intensity, specialize, boons and similar.
- **Paper cards with these mechanics** (round-2 card plan, Oct 14): Pearl Collector
  (perpetual) and Tenacious Pup (a boon) are paper cards whose abilities use these
  digital-only mechanics; they stay unsupported with the mechanics
  (`docs/agents/round2/deferred.tsv`).
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
  cards, sticker sheets and the dexterity cards (below), that is 2,520 ruling texts on
  Oct 21 (2,636 on Oct 2: a text that a test has cited or an exemption has listed since
  then counts as CITED or EXEMPT instead).

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

## Deferred to later

### Agents that play well

Decided 2026-10-01: simulation quality comes at the end.

- **Now:** `RandomAgent` (for fuzzing), plus the external decision interface (item
  `agent-api`): what each player can see and every legal option, as JSON, so an outside
  program or model can play any seat.
- **Later:** heuristic, search-based or learned agents for meaningful game results.
  `Game` is `Clone` and `Game::set_agents` swaps agents on a copy, so search agents can
  play ahead.
