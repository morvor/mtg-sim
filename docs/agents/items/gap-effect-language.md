# gap-effect-language: Effect language: open-ended repetition and tokens with value-defined power/toughness

GOAL: Two missing pieces of the effect AST block about 120 unsupported Oracle lines between them: loops that repeat while a condition holds or while a player chooses to continue, and tokens whose P/T is a value fixed when they're created.

GAPS:

[G29] Conditional and open-ended repetition ("repeat this process")
- Rules: CR 608.2c (instructions in order), the cards' own loop text; rulings below.
- Engine now: `Effect::Repeat { times: Value, effect }` (ability.rs:2661-2664) repeats a number of times
  fixed before the first iteration (resolve.rs:152-157); "Repeat this process once" has a follow-up
  pattern (oracle/patterns/repeat_process.rs). Nothing repeats while a condition holds, until a player
  stops, or "any number of times" with a choice each time. 40 unsupported lines contain "repeat this
  process".
- Cards (UNS): Ad Nauseam, Primal Surge, Grindstone, Sphinx's Tutelage, Tainted Pact, Cultivator Colossus, Eureka, Hypergenesis, Countryside Crusher, Hoarder's Greed,
  Insatiable Frugivore, Tyrant of Discord, Grist, the Hunger Tide (+1), Kindle the Carnage, Trade Secrets,
  Helm of Obedience ("repeats this process until ...").
- Rulings: Primal Surge "Repeating the process includes the instruction to repeat the process", "If putting
  any of those permanent cards onto the battlefield causes abilities to trigger, those abilities will wait to
  go on the stack until Primal Surge has finished resolving"; Ad Nauseam "Each time you put the revealed card
  into your hand and lose the appropriate amount of life, you decide whether to continue", "even if your
  life total has been reduced to 0 or less".
- Approach: `Effect::RepeatWhile { body, cond: Condition, may: Option<PlayerRef>, max_iterations }`
  (condition re-evaluated after each pass, using the pass's "this way" results; a safety cap for loops the
  rules leave to the players, CR 104.4b/732).
- Size: S-M

[G22] Tokens whose power/toughness is a value ("create an X/X ... token, where X is ...")
- Rules: CR 111.3/111.4 (the effect that creates a token defines its characteristics), 107.3 (X fixed
  as the effect is applied, 608.2h).
- Engine now: `TokenSpec { power: Option<i32>, toughness: Option<i32>, .. }` (ability.rs:1645-1656) and
  `Effect::CreateToken { spec: TokenSpec, .. }` (ability.rs:2806-2812) can only describe fixed P/T;
  there is no oracle pattern or engine path for X/X tokens (no hits for "X/X" under src/oracle). 83
  unsupported lines contain "X/X".
- Cards (UNS): Gelatinous Genesis ("Create X X/X green Ooze creature tokens"), Spoils of Blood, Mystic
  Genesis, Promise of Power, Corpse Cobble, Kin-Tree Invocation, Tumbleweed Rising, Formless Genesis,
  Sylvan Offering, Nissa, Ascended Animist, Saint Elenda, Dance of the Tumbleweeds (spree), Pest
  Infestation; token copies with value exceptions (Soul Separator, S27 #67).
- Approach: let `CreateToken`/`TokenSpec` carry `Value` P/T (or a `pt: Option<(Value, Value)>` override
  evaluated once at creation into the token's copiable values), keeping `TokenSpec` usable in
  replacements (`PlusTokens`) and predefined tokens.
- Size: M (core AST change touching every `TokenSpec` construction)

ESTIMATED SIZE: G29 S-M, G22 M (core AST change touching every `TokenSpec` construction).

OVERLAPS: G22 changes `TokenSpec`, which gap-zone-moves (fabricate Servos) and gap-variants-misc don't touch; coordinate with any item editing token creation (actions.rs create_tokens).

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests, and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 85f020d and may have shifted; search for the named functions and types.
- Each gap above was verified against the code. "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). Ruling quotes above may be shortened with "..."; copy the exact substring from data/rulings.jsonl.gz (`zcat data/rulings.jsonl.gz | grep -F "..."`) and check its status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
