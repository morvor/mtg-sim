# mtg-sim — Magic: The Gathering rules engine

A Rust rules engine for simulating Magic games, built on the Comprehensive Rules
(`data/comprehensive-rules.txt`, effective 2026-09-25) and Scryfall bulk data
(`data/oracle_cards.jsonl.gz`, `data/rulings.jsonl.gz`, `data/oracle_tags.jsonl.gz`).
No UI. The data files are the source of truth — the CR text in this repo is newer than
most training data, so **read the actual rule text** (`grep -n "^702.19" data/comprehensive-rules.txt`)
instead of relying on memory. Notably: combat damage assignment order no longer exists
(510.1c/d: divide freely), "tribal" is now "kindred", and there are many new mechanics.

## Crates

- `crates/mtg-data` — loaders: Scryfall cards/rulings/tags, CR parser. Defines the
  `cr!` and `ruling!` citation macros.
- `crates/mtg-engine` — the engine.
- `crates/mtg-api` — external decision interface: per-player observations, every legal
  option for every decision, the JSON agent protocol (`docs/AGENT_PROTOCOL.md`),
  `ExternalAgent` (child process) and `Session` (pull-style embedding).
- `crates/mtg-sim` — CLI simulator (`cargo run --release -p mtg-sim -- --games 100`);
  `--random-decks` fuzzes the engine with random decks of fully supported cards.
- `crates/mtg-tools` — coverage reports (`cr-coverage`, `card-coverage`,
  `rulings-coverage`, `unsupported`).

## Engine map (`crates/mtg-engine/src`)

| File | What |
|---|---|
| `game.rs` | `Game` state, players, effect registries, `ask()` for decisions |
| `object.rs` | `GameObject`, `Characteristics`, zones, stack info |
| `ability.rs` | The ability language (Effect, Sel, Filter, Value, Condition, Cost, TriggerCond, StaticEffect, Modification, ReplacementDef, Restriction, ...) |
| `eval.rs` | Evaluating Filter/Value/Condition/Sel/PlayerRef against the game (`Ctx`) |
| `resolve.rs` | Effect interpreter (`Game::exec`) |
| `layers.rs` | Characteristic computation, layers 1–7 with timestamps/dependencies (CR 613); collects active statics |
| `replacement.rs` | Replacement/prevention pipeline (CR 614–616) |
| `actions.rs` | Primitive actions: zone moves, draw, damage, life, counters, tokens, destroy, sacrifice, tap, attach |
| `triggers.rs` | Trigger detection from events, look-back (CR 603.10), APNAP stacking, delayed & state triggers |
| `sba.rs` | State-based actions (CR 704) and the settle loop (CR 117.5) |
| `stack.rs` | Targets (CR 115), modes, resolution (CR 608), countering |
| `casting.rs` | Casting (CR 601), activation (CR 602/605/606), costs (CR 118), legal actions, land play |
| `mana.rs`, `mana_abilities.rs` | Mana, costs, pools, payment solver, auto-tapping |
| `combat.rs` | Combat (CR 506–511), evasion, requirements/restrictions, damage assignment |
| `turn.rs` | Turn structure, steps, priority loop, main game loop |
| `keyword_impls.rs` | Hook points for keywords + a few built-in keyword expansions |
| `kw/` | **Keyword registry**: one file per keyword (family), `impl KeywordRules`, `inventory::submit!` |
| `keyword_actions.rs`, `keyword_actions_impl.rs` | Keyword actions (CR 701) |
| `oracle/` | Oracle text compiler (normalize → split → parse). `oracle/patterns/` is the **pattern registry** |
| `card.rs` | `CardDef` from Scryfall, `CardDb`, `card("Name")` |
| `testing.rs` | `TestGame` harness + `ScriptedAgent` |
| `agents.rs` | `RandomAgent` |
| `cards/` | **Hand-written card abilities** for genuine one-offs (`ManualAbility`, one file per card; policy in `cards/mod.rs`) |
| `kwa/` | **Keyword action registry** (CR 701): one file per action; see also `keyword_actions*.rs` |
| `decision.rs`, `events.rs`, `types.rs`, `keywords.rs` | `Agent` trait and `Decision`/`Answer`; game events; basic vocabulary (ids, colors, types); the `Keyword` enum |
| Rule topics | `zones.rs` (CR 400–408), `target_rules.rs` (115), `special_actions.rs` (116), `cost_rules.rs`, `spell_costs.rs`, `cost_choices.rs`, `next_spell.rs` (118, 601.2), `life_totals.rs` (119), `excess_damage.rs` (120), `draw_rules.rs` (121), `counter_rules.rs` (122), `stickers.rs` (123), `names.rs` (201), `mana_value.rs` (202.3), `game_terms.rs` (700), `piles.rs` (700.3), `apnap.rs` (101.4), `as_though.rs` (609.4), `prevention.rs` (609.7, 615), `until.rs` (610.3), `skip.rs` (614.10), `copy.rs`, `copy_rules.rs` (707), `shortcuts.rs` (732), `modal_history.rs` |
| Keyword-action rules (CR 701) | `create_rules.rs`, `discard_rules.rs`, `exchange.rs`, `mill_rules.rs`, `reveal.rs`, `scry_rules.rs`, `search_rules.rs`, `shuffle_rules.rs`, `transform_rules.rs`, `behold.rs`; `dice.rs` (coins and dice, 705–706); `splice.rs` (702.47) |
| Card and permanent kinds | `attach.rs`, `battle.rs` (310), `dfc.rs`, `flip.rs` (710), `adventure.rs` (715, 720), `classes.rs` (716), `attraction_cards.rs` (717), `cases.rs` (719), `rooms.rs` (709.5), `merge.rs` (730), `saga.rs` (714), `dungeons.rs` (309), `facedown.rs` (708), `tokens.rs`, `tokens_predefined.rs` (111, 114), `radiation.rs` (728), `monarch_initiative.rs` (725–726), `designations.rs`, `text_change.rs` |
| Game flow | `start.rs`, `opening_hand.rs`, `mulligan.rs` (103), `game_end.rs` (104), `restart.rs` (727), `turn_structure.rs` (500–505), `untap_choice.rs`, `untap_limits.rs` (502.3), `end_turn.rs` (724), `player_control.rs` (723), `subgame.rs` (729), `library.rs`, `choices.rs` |
| Formats and variants | `deck.rs` (100.2), `match_play.rs` (100.6), `ante.rs` (407), `multiplayer/` (800–811), `teams.rs` (805), `casual.rs` (900–905), `planechase.rs` (901), `commander_rules.rs` (903), `draft.rs` (905), `variants.rs` |
| Extension points | `custom.rs` (named custom behaviors), `oracle_ext.rs` (pattern registry dispatch) |
| Tooling | `structure.rs`: structural fingerprints of abilities (AST with literal parameters abstracted) and the structure log written when `MTG_STRUCTURE_LOG=<dir>` is set (abilities exercised in games), read by `mtg-tools structure-coverage` |

Key invariants:
- Every zone change creates a new `ObjectId` (CR 400.7); the old object keeps its last
  known information. Use `game.current(id)` to follow an object; `is_live(id)` to check.
- Characteristics are recomputed by `recompute()` whenever `dirty`; mutate objects via
  primitives in `actions.rs` (they emit events and set `dirty`).
- Anything replaceable goes through `replace()`; anything that can trigger emits an
  `Event`; `flush_events()` detects triggers; `settle()` runs SBAs + puts triggers on the stack.
- Decisions go through `game.ask(player, Decision)`; always validate answers and fall back
  to a sensible default on `Answer::Default` or invalid answers.

Out of scope for now: `docs/DEFERRED.md` lists what the project has decided not to support
yet (Contraptions, host/augment, sticker sheets, digital-only cards, cards not legal in
any format). Don't implement those;
note anything new you run into there.

## Extending — prefer adding files over editing shared code

- **Keywords (CR 702)**: add `src/kw/<keyword>.rs` implementing `KeywordRules` and
  `inventory::submit! { KeywordRegistration(&X) }` (see `src/kw/bushido.rs`). Files in
  `src/kw/` are auto-included by `build.rs`.
- **Oracle patterns**: add `src/oracle/patterns/<topic>.rs` registering `EffectPattern`,
  `TriggerPattern`, `StaticPattern`, `ConditionPattern`, or `AbilityPattern` via
  `inventory::submit!` (see `src/oracle/patterns/mod.rs`). Auto-included.
- **Hand-written card abilities** (genuine one-offs only): add `src/cards/<card>.rs`
  registering a `ManualAbility` (card, face, the exact normalized block from
  `mtg-tools unsupported --card "Name" --raw`, a `build` fn returning AST abilities, and a
  reason); rules the AST can't express go in a `KeywordRules` impl with `kinds() -> &[]`
  using the registry-wide hooks (`custom_*`, `cast_prohibited`, `target_forbidden`, ...),
  finding their sources by the custom ability, never by name. **Compile first**: any
  construct two or more cards share must be compiled. Every manual card needs a test in
  `tests/cards/m_<card>.rs`; `mtg-tools manual-check` (also run by `cargo test`) fails on
  stale, now-compilable, duplicate or untested entries. See `src/cards/mod.rs`. The Oracle
  round-trip renderer reports these abilities as "manual" (verified by their tests).
- **Tests**: add files to `crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/`.
  Every `.rs` file there is auto-included into that directory's test binary. Name files by
  rule: `tests/cr/r704_state_based_actions.rs`, `tests/keywords/k702_019_trample.rs`,
  `tests/actions/a701_034_proliferate.rs`, `tests/rulings/<card_or_mechanic>.rs`.
- Edit core files only when the core is wrong or lacks a needed hook; keep such edits
  minimal and focused, and add a hook rather than special-casing.
- If you add an AST variant (`ability.rs`), handle it everywhere it's matched (the compiler
  will tell you) — never leave `todo!()`/`unimplemented!()` in engine code paths.

## Tests: citing rules and rulings

- Every rules test must cite the rule(s) it verifies: `cr!("704.5a", "704.5b");` at the
  top of the test. The macro fails the test if the rule doesn't exist, and
  `mtg-tools cr-coverage` counts citations. Only cite a rule if the test actually
  exercises that rule's behavior.
- Tests encoding a Scryfall ruling cite it: `ruling!("Card Name", "distinctive substring");`
  — fails if the card has no such ruling (so citations can't be invented). Copy the
  substring from the real ruling text (`zcat data/rulings.jsonl.gz | grep ...`).
- Rulings coverage counts ruling texts: a text shared by several cards is covered by a
  test on any card it applies to. Pick a card with typical wording, and when the cards
  word the relevant ability differently, test each wording (cards worded slightly
  differently can compile and behave differently). The `ruling!` macro needs literals,
  so a test looping over cards has one `ruling!("Card", "...")` line per card.
  `python3 scripts/rulings_batches.py summary|list|show P042|card "Name"` reports every
  ruling text as CITED, EXEMPT or OPEN and splits them into batches of similar rulings.
- Use `mtg_engine::testing::TestGame` (see its docs and `tests/smoke.rs`). Use real cards
  via `t.battlefield(P0, "Card Name")` etc. For rules that need a custom object, build a
  `CardDef::custom(...)`.
- Rules that have genuinely no engine-testable behavior (purely informational text,
  physical-card presentation, tournament/draft procedures) may be listed in
  `docs/cr-exemptions/<area>.tsv` as `rule<TAB>reason` (one file per area, to avoid merge conflicts). Be conservative: deck-construction rules,
  for example, are testable via deck validation code.
- Rulings with no engine-testable content (release notes, presentation, tournament policy)
  may be listed in `docs/rulings-exemptions/<batch>.tsv` as `Card<TAB>substring<TAB>reason`.
  `mtg-tools rulings-coverage --card "Name"` / `--text "substring"` shows per-ruling status.

## Commands

```sh
cargo build -p mtg-engine
cargo test -p mtg-engine                       # all engine tests
cargo test -p mtg-engine --test cr             # one test binary
cargo run --release -p mtg-tools -- cr-coverage
cargo run --release -p mtg-tools -- card-coverage
cargo run --release -p mtg-tools -- manual-check   # hand-written abilities: current, compiled-first, tested
cargo run --release -p mtg-tools -- rulings-coverage --card "Tarmogoyf"
cargo run --release -p mtg-tools -- unsupported --limit 50 --filter "enters"
rm -rf target/structlog && MTG_STRUCTURE_LOG=$PWD/target/structlog cargo test --workspace
cargo run --release -p mtg-tools -- structure-coverage --write docs/STRUCTURE_COVERAGE.md   # structures no test exercises
cargo run --release -p mtg-tools -- structure-coverage --card "Fetid Heath" --show-fingerprints
cargo run --release -p mtg-sim -- --games 200
cargo run --release -p mtg-sim -- --random-decks --games 1000   # fuzz: random decks; panics/hangs print a repro command
```

Build machines are small (4 cores, ~15 GB RAM) and running out of memory restarts the whole
machine, killing every agent on it: `export CARGO_BUILD_JOBS=2`, never run two cargo builds
at the same time in one worktree, and never build a second checkout just to compare
before/after numbers (report the after number and the base from an earlier measurement).

Before committing: `cargo build --workspace --all-targets` must be warning-free enough to
read, and `cargo test --workspace` must pass. Run `cargo fmt` on files you touch.
