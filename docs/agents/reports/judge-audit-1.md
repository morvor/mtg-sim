# Judge audit 1: do our tests assert what the rules say?

A blind audit, the user's pick E from the Oct 14 review, run once on a random sample of tests.
For each sampled test, an agent that never saw the engine or its tests predicted the outcome
from the Comprehensive Rules, Oracle text and Scryfall rulings alone. Its predictions were
then compared with what the test asserts. Since every test passes, what a test asserts is also
what the engine does.

**Result: the judge agreed with every assertion: 225 of 225 questions from 58 tests.**
With 0 wrong tests in 58, the share of tests that assert a wrong outcome is below about 5%
(one-sided 95% bound), for errors the judge can detect (see Limits).

## Method

- **Sample:** 60 tests drawn at random (seed 20261014) at commit 9b0af15a, by test directory:
  14 cr, 18 rulings, 10 keywords, 12 cards, 6 actions.
- **Describe:** an agent read the test and wrote the game situation: players, zones, the Oracle
  text of every card, and the actions and choices in order. It turned each assertion about the
  game's outcome into a question with an exact answer format, at most 6 per test, and kept the
  asserted values apart. Two tests were set aside because they play no game: cr-02 checks how
  card data is loaded, and cards-05 checks the Oracle compiler's output.
- **Judge:** a separate agent answered the questions from the situation, the CR text
  (data/comprehensive-rules.txt), Oracle text and rulings. It was forbidden to read the engine,
  its tests, docs or git history. Its transcripts show it complied: all 79 of its shell commands
  read only the frozen checkout's data/ files.
- **Compare, adjudicate, verify:** answers were compared with the assertions after normalizing
  the format. A disagreement would have gone to an adjudicator with full access, and a "the test
  is wrong" verdict to two agents trying to refute it. There were no disagreements, so neither
  step ran.
- **Size:** 118 agent runs in about 18 minutes.
- **Files:** the full record (situations, questions, asserted values, the judge's answers and
  reasoning) is in `judge-audit-1.json`. The workflow script lives in the coordinator's session.

| Tests judged | Questions | Agreed | Judge confidence |
|---|---:|---:|---|
| cr 13, rulings 18, keywords 10, cards 11, actions 6 (58) | 225 | 225 | 223 high, 2 medium |

## Cases the judge flagged as not fully settled by the rules

All three still matched the test. They are candidates for the interpretation register (pick D):

- **cr-04** (`a_conspiracy_draft_game_is_multiplayer_free_for_all`): is a Conspiracy Draft game
  for exactly two players valid? The engine says no. The judge agreed with medium confidence:
  905.3 calls the game multiplayer, and 100.1b defines multiplayer as more than two players, but
  the CR never forbids two players outright.
- **rulings-18** (`gigantoplasm_copying_another_gigantoplasm`): when a copy effect's exception
  grants an ability the copied values already contain, does the copy have it twice? The engine
  says two instances. The judge agreed with medium confidence, from 707.9a and the cited
  ruling, but notes that the rules don't say it outright.
- **rulings-15** (backup on a Clone): this one is about the question's wording, not the rules.
  "Immediately after Clone resolves" comes before the trigger is put on the stack (117.5,
  603.3). The test means the point when a player next receives priority.

## Limits

- **Same kind of agent.** The judge is the same kind of agent that wrote the engine and the
  tests, so a misunderstanding they share would not show up. Only an independent source can
  catch that, e.g. comparing against Forge or XMage (pick C, not chosen yet), or a human judge
  reviewing a sample.
- **The describer saw the test.** Sometimes it had to spell out the test's scripted choices, and
  those can contain the rule being tested. For example, in keywords-07 (trample over
  planeswalkers) the fallback damage assignment the script uses is described in the rule's own
  terms. The judge still had to decide that the first proposed assignment was illegal.
- **Rulings tests hand the judge the ruling.** The ruling often states the outcome, so for
  those 18 tests agreement mostly confirms that the test sets up the situation the ruling
  describes.
- **Sample size.** 58 tests can only rule out error rates above about 5%. Ruling out 1% would
  take about 300 tests: roughly 600 agent runs, or 2 hours.

## What next (after the weekly usage reset; nothing issued yet)

- Record the cr-04 and rulings-18 calls in the interpretation register when it exists.
- If more confidence is wanted, either audit a larger sample (for example 300 tests, aimed at
  layers, replacement effects, copies, triggers and multiplayer), or add the independent source
  above, which addresses the shared-misunderstanding limit that a larger sample of the same kind
  can't.
