# interpretation-register: one reviewable register of every rules judgment call

WHY: the user wants the simulation to follow the rules precisely, and to be able to see and overrule our judgment calls. Today those calls are scattered: in code comments, commit messages, reports (the changeling decision is in docs/agents/reports/structure-tests.md) and tests. The user chose (Oct 14) to keep them in one register. Each entry gives the rule text, the decision, the alternatives and the tests that hold the decision in place.

1. Format. Use `docs/interpretations/<area>.md`, one file per area so that merge conflicts stay rare. Example areas: characteristics-layers, copy, replacement-prevention, triggers, stack-casting-costs, combat, zones-objects, players-multiplayer, turn-structure, decisions-policies, variants. Each entry:
   ```
   ## <id>: <one-line title>
   - Kind: interpretation | deviation | policy
   - Status: decided | provisional
   - Rules: rule numbers with the exact text quoted (and Scryfall rulings, quoted, when they bear on it)
   - Question: what the rules leave open
   - Decision: what the engine does
   - Alternatives: the other readings, and what each would change
   - Why: the reasoning (CR and rulings first; tournament documents such as the MTR only as supporting evidence)
   - Affects: example cards (and how many, if known)
   - Code: file and function
   - Tests: the tests that pin it
   ```
   Kinds:
   - **interpretation**: the rules are ambiguous or silent, and we chose a reading.
   - **deviation**: the engine knowingly does something other than what the CR says, for example a simplification or an unsupported corner case. Each one needs a reason and a plan: fix it, or accept it and say why.
   - **policy**: the rules leave something to a player or to tournament procedure, and the engine decides it. Examples: defaults when an agent gives no answer or an invalid one; shortcuts (CR 732); loops and draws (CR 104.4b); ordering choices the engine makes for a player; what hidden information agents see; mtg-sim's turn and action limits.

   Ids are short, stable slugs, such as `changeling-cda-only`. Mark an entry **provisional** when a reasonable judge could rule the other way; the user reviews those.
2. Tooling.
   - Add `interp!("<id>")` to mtg-data, next to `cr!` and `ruling!`. It fails the test unless an entry with that id exists under docs/interpretations/.
   - Add a check that `cargo test` runs (as it runs `manual-check`, or as part of it). It verifies that ids are unique, that every `interp!` names an existing entry, and that every entry is cited by at least one test.
   - Add `mtg-tools interpretations --write`, which writes `docs/INTERPRETATIONS.md`: the provisional entries first, then an index of all entries (id, title, kind, status, area).
3. Seed the register from a sweep. Different searches find different calls, so run all of them, with subagents in parallel, then merge the duplicates.
   - **Code comments in crates/*/src that mention:** interpret, ambigu, "the rules don't say" or "doesn't say", unclear, not specified, treat as, assume, approximat, simplif, heuristic, "for now", TODO. Also look for tie-breaks that pick the "first" or "lowest id".
   - **Tests** and their comments that state a choice.
   - **Commit messages:** `git log` subjects and bodies on the branch with treat, interpret, choose, "rather than", "instead of", ambigu, or "review fixes". Reviewers often record a call there.
   - **docs/agents/reports/\*.md:** the changeling decision becomes entry `changeling-cda-only`. It is option B, made by the coordinator on Oct 14, with status decided.
   - **docs/MUTATION.md:** survivors triaged "equivalent with today's cards", "for in-scope cards" or "in practice" are latent judgment calls.
   - **docs/DEFERRED.md** lists scope decisions. Refer to it; don't copy it.
   - **Exemption reasons** in docs/cr-exemptions/ and docs/rulings-exemptions/ that encode a decision.
   - **Decision defaults:** the fallbacks for `Answer::Default` and invalid answers in `game.ask` callers. Write one policy entry per kind of decision, not one per call site.
   - **CR rules that leave choices to players or procedure:** ordering (101.4, 603.3b), loops (104.4b), shortcuts (732), hidden information (mtg-api visibility), concession, and mtg-sim's limits.

   Check every entry against the code, meaning what the engine really does, not what anyone remembers. Pin every entry with a test that cites `interp!`, and write the test if none exists.
4. Point to the register from:
   - CLAUDE.md, with one line under "Tests: citing rules and rulings": when you make a judgment call, add an entry, or cite an existing one with `interp!`.
   - The reviewer brief in docs/agents/worker.md: the reviewer checks that new judgment calls are registered.
5. Report: the number of entries by kind and status, and each provisional entry in one line, so the coordinator can put them to the user.

DONE when the register, the macro, the check and docs/INTERPRETATIONS.md exist, the sweep's findings are registered with tests, and CLAUDE.md and worker.md point to the register.
