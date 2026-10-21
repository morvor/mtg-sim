# cards4-manual-defs: Hand-written card definitions IV: one-off abilities with no shared construct — 2 texts / 2 cards (2 unlocked alone)

SCOPE: 2 unsupported ability texts on paper cards, blocking 2 paper cards; both become fully supported when this item alone is done. Both texts were in cards3-costs-mana-replacements, which has landed, and are still unsupported after it. The plan's one-off check searched all in-scope paper cards' Oracle text for each construct (and the review repeated it, see card-support-plan-4.md, "Adversarial review"); every other leftover text of the landed round-3 items has a construct that another in-scope card shares, so it is in a cards4 item. The full list ships as items/cards4-manual-defs.texts.tsv (columns: cards blocked, normalized text, example card, piece kind, failing piece, family, previous item, note). Normalized text = reminder text removed, the card's own name as ~, numbers as N.

GOAL: Write hand-written definitions (src/cards/, `ManualAbility`, a test per card in tests/cards/m_<card>.rs, `mtg-tools manual-check`) for the two texts below. Rules the AST can't express go in a `KeywordRules` impl with `kinds() -> &[]` using the registry-wide hooks, finding their sources by the custom ability, never by name (CLAUDE.md).

READ FIRST:
- crates/mtg-engine/src/cards/mod.rs (policy), an existing manual card for the shape (`ls crates/mtg-engine/src/cards/`), custom.rs (named custom behaviours), keyword_impls.rs (registry-wide hooks for rules the AST can't express), replacement.rs and stack.rs (countering, CR 701.6), prevention.rs.
- items/cards3-manual-defs.md (round 3, in flight while this was planned: reuse whatever hooks it adds; merge origin first).

PHRASE FAMILIES (texts in family; the construct and why it is a one-off; the engine need):
- one-off constructs (family `one-off`): 2.
  - Guile: "If a spell or ability you control would counter a spell, instead exile that spell and you may play that card without paying its mana cost." — a replacement of the counter event itself (CR 701.6, 614.1a). Probe: "If a spell or ability you control would counter a spell, you gain 1 life instead." fails. No other in-scope paper card replaces countering (searched "would counter", "would be countered"; "can't be countered" is a different rule). Rulings: a spell that can't be countered isn't affected; exiling is mandatory, casting optional and only right then; a spell exiled this way was never countered. Engine need: a replacement hook where a spell or ability you control counters a spell (stack.rs countering goes through `replace()` with the countering object's controller), then the exile and a one-time free-cast permission.
  - Torrent of Lava: "As long as ~ is on the stack, each creature has \"{T}: Prevent the next 1 damage that would be dealt to ~ by ~ this turn.\"" — a static ability of a sorcery that works while it is on the stack (CR 113.6 exception) and grants each creature an ability whose prevention names both the creature and the spell. The only other on-the-stack static in scope is split second, a keyword (searched "is on the stack", "as long as ... on the stack"). Probe: "Each creature has \"{T}: You gain 1 life.\"" fails on the sorcery too. Ruling: the ability can be activated and resolved before Torrent itself resolves. Engine need: a static ability that functions on the stack for the spell's source, a granted activated ability with a prevention shield from that source to that creature for the turn.

ALSO BLOCKED BY: none.

RULES NOTES:
- CR 701.6 (counter), 614.1a (replacement effects with "instead"), 113.6 (abilities that function in other zones; a spell's static abilities work on the stack only when they say so), 615.1, 615.7 (prevention shields).

BASELINE (paper texts containing the filter in round4/baseline.tsv, case-insensitive; counts include other items' texts): `would counter` → 1 texts / 1 cards; `on the stack` → 1 texts / 1 cards

FILES THIS ITEM OWNS: new src/cards/guile.rs, src/cards/torrent_of_lava.rs and tests/cards/m_guile.rs, tests/cards/m_torrent_of_lava.rs; hooks in custom.rs/keyword_impls.rs only where a manual card needs one.

COMMON RULES FOR CARD ITEMS (round 4):
- Before writing a definition, re-run the one-off check: search all in-scope paper cards' Oracle text for the construct (a card that shares it means it must be compiled instead, CLAUDE.md "Compile first"). `mtg-tools manual-check` (also run by `cargo test`) fails on stale, now-compilable, duplicate or untested entries.
- The exact normalized block for `ManualAbility` comes from `mtg-tools unsupported --card "Name" --raw`.
- Faithfulness first. Each card's test asserts the in-game outcome with TestGame (citing cr!() for rules exercised and ruling!() for the card's Scryfall rulings exercised). Judgment calls go in the interpretation register (docs/interpretations/, `interp!`).
- Build hygiene (CLAUDE.md): `export CARGO_BUILD_JOBS=2`, one cargo build at a time per worktree, no second checkout for before/after numbers.
- DONE when both cards are fully supported with tested definitions, or a remaining text is listed with a reason.
