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
- **Status:** sheets are to be excluded from `card-coverage` and `unsupported` and
  reported separately; the `cards-dice-un` item does this. Rulings that apply only to
  sticker sheets are DEFERRED in `scripts/rulings_batches.py`.

### Digital-only cards

Decided 2026-10-01.

- **Scope:** the 1,916 cards with no paper printing (MTG Arena/Alchemy and other
  digital-only cards) and their mechanics: perpetual, seek, conjure, spellbooks, draft from
  a spellbook, intensity, specialize, boons and similar.
- **Status:** `card-coverage` counts paper cards only. The plan for them is
  `docs/agents/items/cards-digital-alchemy.md` (939 unsupported texts), which is not
  scheduled. Rulings that apply only to digital-only cards are DEFERRED in
  `scripts/rulings_batches.py`.

### Cards not legal in any format

Decided 2026-10-02.

- **Scope:** about 2,050 paper cards with no format where they are legal or restricted:
  playtest cards (737), Un-set cards that aren't legal anywhere (474, including the acorn
  cards), planes and phenomena (212), schemes (102), cards banned everywhere such as the
  ante cards and dexterity cards (77), vanguards, conspiracies and memorabilia.
- **Already implemented:** the variant and ante rules themselves (Planechase, Archenemy,
  Vanguard, Conspiracy draft, ante; CR 407 and 900-905) are in the engine and tested, and
  stay supported. Only covering these individual cards is deferred.
- **Status:** `card-coverage` and `unsupported` already skip them. Rulings that apply only
  to such cards are DEFERRED in `scripts/rulings_batches.py`. Together with digital-only
  cards and sticker sheets, that is 2,636 ruling texts.

## Deferred to later

### Agents that play well

Decided 2026-10-01: simulation quality comes at the end.

- **Now:** `RandomAgent` (for fuzzing), plus the external decision interface (item
  `agent-api`): what each player can see and every legal option, as JSON, so an outside
  program or model can play any seat.
- **Later:** heuristic, search-based or learned agents for meaningful game results.
  `Game` is `Clone` and `Game::set_agents` swaps agents on a copy, so search agents can
  play ahead.
