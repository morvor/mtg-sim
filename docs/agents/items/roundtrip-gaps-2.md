# roundtrip-gaps-2: renderer gaps in effects, values, modifications and remembered groups

Part of the `roundtrip-followups.md` stream. Baseline (Oct 14): 25,278 of 28,702 fully
supported cards pass the round trip; 3,394 mismatch.

CLUSTERS TAKEN (from `roundtrip --clusters 40`):
- `gap: Effect` (38) — Karn, Scion of Urza; Plunge into Darkness; Karn, Living Legacy
- `gap: Value` (34) — Gorex, the Tombshell; Windfall; Extus, Oriq Overlord
- `gap: Modification` (17) — Quickening Licid; Tempting Licid; Transmogrifying Licid
- `gap: a remembered group no instruction names` (8) — Green Slime; Twist Allegiance
- `gap: remembering Union` (7) — Cauldron's Gift; Distended Mindbender
- `gap: terms of a static permission to play cards` (6) — Lobelia, Defender of Bag End; Chainer, Nightmare Adept
- `gap: Sel` (4), `gap: StaticEffect` (4), `gap: qualities of an alternative list` (4),
  `gap: several remembered numbers` (4) — Korvold, Gleeful Glutton; Lilah, Undefeated Slickshot; Combo Attack; Primal Adversary

The parallel item roundtrip-gaps-1 takes the Filter/Condition/replacement/TriggerCond and
target-slot gaps; don't take those.

For each card: teach the renderer (`crates/mtg-engine/src/oracle/render/`) to word the AST
node from its meaning, independently of the parser. Once it renders, triage any remaining
mismatch: parser bug (fix + in-game test in `tests/cards/roundtrip_gaps_2.rs` + row in the
docs/ROUNDTRIP.md bug table), renderer wording, or a CR-justified equivalence. Gaps that
expose an AST that can't express the text's meaning are usually parser bugs. Regenerate
docs/ROUNDTRIP.md and docs/roundtrip-passing.txt; `roundtrip --check` must pass.

## Parser bugs found (review)

- Devouring Rage: "For each Spirit sacrificed this way, that creature gets an additional
  +3/+0": "that creature" was each sacrificed Spirit, so the target got only +3/+0. Fixed in
  `iteration_grammar::for_each_object` ("that [noun]" with a noun the iterated objects
  aren't called by is the target named before).
- Ixhel, Scion of Atraxa: "each opponent who has three or more poison counters exiles the
  top card of their library face down. You may look at and play those cards": "those
  cards" were the last opponent's only (none when that opponent exiled nothing). Fixed in
  `simultaneous.rs` (`Frames::finish`: "it" after an "each player" instruction is what all
  the players' instructions acted on).

Tests: `crates/mtg-engine/tests/cards/roundtrip_gaps_2.rs` (also Dargo's two reductions and
Phabine's revealed-card counts, checked in play because their renderings looked odd).

## Remaining

Mismatches left in the taken clusters, each checked against the AST (and in play where
noted) and judged to play as the text says; what's left is wording:

- Combo Attack: "each deal damage equal to their power" rendered as "for each of two
  target creatures, it deals X damage"; one damage instruction per creature, dealt
  together (CR 608.2f via `simultaneous.rs`).
- Karn, Scion of Urza: −1 word order ("you own" after "with a silver counter on it").
- Lilah, Undefeated Slickshot: the trigger gives the spell the plot replacement rather
  than being the replacement; the trigger resolves before the spell, same result.
- Plunge into Darkness: "put one of those cards into your hand" as "put a card from among
  them"; "exile the rest" as "put the rest into exile".
- Indoraptor, the Perfect Hybrid: "where X is" placed before "unless".
- Cauldron's Gift: adds "under its owner's control"; the card is in your graveyard, so
  you own it (CR 404.2) and it's your control either way. "additional" counter wording.
- Devouring Rage: after the fix, iteration over the sacrificed Spirits is rendered as such.
- Patriarch's Bidding, Harsh Mercy: placement of "of a type chosen this way"; "isn't a
  permanent of" for "aren't of".
- Nameless Race: "the total number ... in their graveyards" vs "the number ... your
  opponents' graveyard"; the AST counts cards opponents own in graveyards (checked).
- Distended Mindbender, Dreams of Steel and Oil: the renderer adds "that player owns" (a
  card in a player's hand or graveyard is owned by them, CR 404.2, 402.1).
- Doc Aurlock, Grizzled Genius: "from your graveyard or from exile" as two noun phrases.
- Twist Allegiance: the card says "each gain control" (not "exchange", CR 701.12); the AST
  stores both groups first, then two gain-control effects: same result.
- Natural Balance: still an `Effect::Custom(look: opt in)` gap; "each player who" compiled as
  a per-player condition (checked: `AsPlayer` + `If` about that player).
- Diligent Zookeeper: "to a maximum of 10" vs "the lesser of ... and 10".
- Phabine, Boss's Confidant: "revealed this way" rendered as the union of the variables the
  parley instruction fills (checked in play: one Citizen, +1/+1).
- Runo Stromkirk // Krothuss: "create two of those tokens instead" wording.
- Doomsday: "you lose half your life, rounded up" compiled as a loop over you (same).
- Wolverine, Fierce Fighter: the healing replacement as "heal X, where X is the damage marked on it minus that much" (the earlier damage), wording.
- Alpine Moon, Ultima, Origin of Oblivion: "gain"/"has" vs "have"/"gains" for a granted ability (layer 6
  either way).
- Revealing Wind: word order ("attacking or blocking face-down creature").
- Interdict: "source" after the type list, possessive order.
- Blitzwing, Adaptive Assailant: "choose flying or indestructible at random" compiled as a
  random choice of 1 or 2 and conditionals (same distribution). The front face's "If no
  life is lost this way" checks the life actually lost (`Prev`), not the amount asked.
- Deliver Unto Evil: the Bolas branch as an "instead" after the opponent's choice (tested
  in `tests/cards/choice_grammar_rest.rs`).
- Ixhel: after the fix, "each opponent if there are ..." wording only.
- Necrodominance, Dauthi Voidwalker, Mission Briefing: "your graveyard"/"an opponent's
  graveyard" as "a graveyard" with an owner (cards go to their owner's graveyard, CR
  404.2); quality order.
- Dargo, the Shipwrecker: the two reductions as two statics (checked in play: {4} for two
  sacrificed this way plus {2} for one other this turn).
- Leeching Licid: "the upkeep of enchanted creature's controller" as "each upkeep if
  enchanted creature's controller is the active player".
- Trap the Trespassers: "that many stun counters" as "X, where X is the number of votes
  for it".
