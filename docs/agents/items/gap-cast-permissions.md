# gap-cast-permissions: Casting permissions: richer play grants, choosing which permission is used, foretell faces

GOAL: Permissions to play or cast cards from other zones lack expressiveness: grants from resolved effects always allow land plays and can't carry an alternative/extra cost or flash timing, the engine silently picks which permission a cast uses, and foretold cards can only be cast as their front face.

GAPS:

[G18] Play/cast grants from resolved effects are too coarse
- Rules: CR 601.3, 305.9 / 305.1 ("you may cast that card" doesn't let you play a land), 118.9 (an
  alternative cost attached to a permission), 601.3e, 702.8 (flash timing given by the permission).
- Engine now: `PlayGrant { player, object, duration, free, source, turn }` (casting.rs:53-60) is created
  by `Effect::GrantPlayPermission { who, what, duration, free }` (ability.rs:3216-3221) and always permits
  both playing a land and casting (`permission_allows_with` ignores `land` for grants, casting.rs:384-390).
  A grant can't be cast-only, can't carry an alternative cost ("pay life equal to its mana value rather than
  pay its mana cost") or an extra cost ("A spell cast this way costs {2} more"), and can't give flash timing
  ("you may cast them as though they had flash").
- Cards (UNS; 126 unsupported lines contain "you may cast that card/it/those cards", part of them for
  this reason): Ragavan, Nimble Pilferer ("exile the top card of that player's library. Until end of turn, you
  may cast that card"); Hurl Through Hell; Xander's Pact, Inside Information ("If you cast a spell this way,
  pay life equal to its mana value rather than pay its mana cost": an alternative cost on the grant); Invasion of Gobakhan ("its owner may play it. A spell cast this way costs {2}
  more"); Azula, Cunning Usurper ("cast them as though they had flash"). Patterns currently accept only
  "you may play" (or texts whose card can't be a land) because a grant worded "you may cast" would let a
  land be played (oracle-card_flow implementer note).
- Approach: extend `PlayGrant`/`GrantPlayPermission` with `spells_only`, `alt_cost: Option<Cost>`,
  `extra_cost`, `flash` (mirroring `PlayPermission`, ability.rs:2268-2280) and honour them in
  `permission_allows_with` and `cast_options`.
- Size: S-M

[G31] Choosing which permission a cast or land play uses; per-type permissions
- Rules: CR 601.2/601.3 (a player announces how a card is cast, including which permission allows casting
  it from a zone), 305.1/305.9; Muldrotha rulings.
- Engine now: kw/once_each_turn_cast.rs (header, lines 9-14): a spell cast from the graveyard "uses the
  permission of the first such object that allows it"; there is no decision for which permission is used,
  and once-each-turn usage is recorded per object (`TurnHistory::once_permissions_used: Vec<ObjectId>`,
  game.rs:446), so "a land and a permanent spell of each permanent type" can't be tracked.
- Cards (UNS): Muldrotha, the Gravetide ("During each of your turns, you may play a land and cast a
  permanent spell of each permanent type from your graveyard"); Karador + Yawgmoth's Will / Lurrus combos
  consume the wrong permission.
- Rulings: Muldrotha "If multiple effects allow you to play a card from your graveyard, you must announce
  which permission you're using as you begin to play the card", "you may cast an artifact creature spell as
  your artifact spell and cast another artifact creature spell as your creature spell", "Use the type of the
  card as it's played or cast to determine which permanent type to count it as".
- Approach: make the permission part of the cast/land-play action (a `CastOption` field listing the
  permission source and, for typed permissions, the type slot), consume exactly that one, and key usage by
  (source, slot).
- Size: M

[G21] Foretell from exile only offers the front face
- Rules: CR 702.143 (foretell), 712.11b (modal DFC cast as either face), 601.3e.
- Engine now: kw/foretell.rs:225-233 builds every foretell option with `FaceState::Front`; foretell costs
  "equal to its mana cost reduced by {2}" use the front face's cost.
- Cards: Ethereal Valkyrie / any MDFC foretold (Kolvori, God of Kinship // The Ringhart Crest example).
- Rulings: Ethereal Valkyrie "If you foretell a modal double-faced card, the foretell cost will be based on
  the mana cost of the face you cast from exile ... you can't play that card as a land".
- Approach: iterate `castable_faces` as escape/flashback already do (kw/escape.rs:54).
- Size: S

ESTIMATED SIZE: G18 S-M, G31 M, G21 S.

OVERLAPS: gap-spell-costs extends `cast_options` with external alternative costs; agree on one `CastOption` shape (permission source, alternative cost, flash) before both land.

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
