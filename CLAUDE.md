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
- `crates/mtg-sim` — CLI simulator (`cargo run --release -p mtg-sim -- --games 100`).
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
| Others | `attach.rs`, `battle.rs`, `copy.rs`, `dfc.rs`, `facedown.rs`, `saga.rs`, `tokens*.rs`, `library.rs`, `choices.rs`, `designations.rs`, `multiplayer.rs`, `mulligan.rs`, `variants.rs`, `text_change.rs`, `custom.rs` |

Key invariants:
- Every zone change creates a new `ObjectId` (CR 400.7); the old object keeps its last
  known information. Use `game.current(id)` to follow an object; `is_live(id)` to check.
- Characteristics are recomputed by `recompute()` whenever `dirty`; mutate objects via
  primitives in `actions.rs` (they emit events and set `dirty`).
- Anything replaceable goes through `replace()`; anything that can trigger emits an
  `Event`; `flush_events()` detects triggers; `settle()` runs SBAs + puts triggers on the stack.
- Decisions go through `game.ask(player, Decision)`; always validate answers and fall back
  to a sensible default on `Answer::Default` or invalid answers.

## Extending — prefer adding files over editing shared code

- **Keywords (CR 702)**: add `src/kw/<keyword>.rs` implementing `KeywordRules` and
  `inventory::submit! { KeywordRegistration(&X) }` (see `src/kw/bushido.rs`). Files in
  `src/kw/` are auto-included by `build.rs`.
- **Oracle patterns**: add `src/oracle/patterns/<topic>.rs` registering `EffectPattern`,
  `TriggerPattern`, `StaticPattern`, `ConditionPattern`, or `AbilityPattern` via
  `inventory::submit!` (see `src/oracle/patterns/mod.rs`). Auto-included.
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
- Use `mtg_engine::testing::TestGame` (see its docs and `tests/smoke.rs`). Use real cards
  via `t.battlefield(P0, "Card Name")` etc. For rules that need a custom object, build a
  `CardDef::custom(...)`.
- Rules that have genuinely no engine-testable behavior (purely informational text,
  physical-card presentation, tournament/draft procedures) may be listed in
  `docs/cr-exemptions/<area>.tsv` as `rule<TAB>reason` (one file per area, to avoid merge conflicts). Be conservative: deck-construction rules,
  for example, are testable via deck validation code.

## Commands

```sh
cargo build -p mtg-engine
cargo test -p mtg-engine                       # all engine tests
cargo test -p mtg-engine --test cr             # one test binary
cargo run --release -p mtg-tools -- cr-coverage
cargo run --release -p mtg-tools -- card-coverage
cargo run --release -p mtg-tools -- unsupported --limit 50 --filter "enters"
cargo run --release -p mtg-sim -- --games 200
```

Before committing: `cargo build --workspace --all-targets` must be warning-free enough to
read, and `cargo test --workspace` must pass. Run `cargo fmt` on files you touch.
