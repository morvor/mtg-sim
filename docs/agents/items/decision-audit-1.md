# decision-audit-1: check every provisional and non-user-decided register entry against all rulings of its cards

Audit item, effort max, with an independent auditor subagent and a separate adversarial checker subagent. Why: the user found that the coordinator's relay of `copy-of-a-card-that-left-is-made-where-it-was` said no ruling covered a case that a God-Eternal Kefnet ruling (2019-05-03) does cover ("If the card leaves your hand before Kefnet's triggered ability resolves, you'll copy it using its last known information."), and asked how it was missed. The user is concerned about the integrity of the project; this audit checks the other entries for the same kind of miss.

For every register entry (docs/interpretations/*.md) that is (a) provisional, or (b) decided but NOT "Decided by the user" (decided by the coordinator or a worker; about 57):
1. Take every card in its Affects (and the cards its Question and tests name). Pull EVERY ruling of those cards from data/rulings.jsonl.gz (all dates), not only the ones the entry quotes.
2. Decide, per entry: does any ruling (or CR rule) answer the entry's question directly? Does any contradict its Decision? Is any ruling the entry quotes misquoted, misdated or quoted out of context? Does the entry claim "no ruling/rule addresses this" when one does?
3. Also check the entry's Status claim: a provisional entry that a ruling settles should be decided "by the ruling"; a decided entry that rests on a ruling that doesn't actually say it should be flagged.
Fix plain errors (misquotes, dates, a settled entry wrongly provisional) in place, with a note in the entry. Anything that would change engine behavior, or a Decision a ruling contradicts, goes in the report for the coordinator and the user with every source quoted verbatim with dates, and is not changed without the user.

Also check the coordinator's presentation path: list any entry whose Rules field quotes a ruling that answers its own Question (as Kefnet's did) — these are the cases a summary can most easily misstate.

Report: per entry, OK / fixed (what) / for the user (what, with quotes). `cargo test -p mtg-data` passes; yield row; follow worker.md's one-item-per-session rule and notify the coordinator.
