# Rulings exemptions

One `.tsv` file per batch or area (e.g. `mechanics-adamant.tsv`), each line
`<Card Name><TAB><distinctive substring of the ruling><TAB><reason>`.

Exempt only rulings with no engine-testable content: release history ("This is a change
from previous rules"), card presentation and art, tournament or organized-play policy,
Oracle wording explanations that change no behavior, and similar. Everything else needs a
test citing it with `ruling!("Card Name", "substring")`.

Two kinds of ruling are exempt in part even though they speak about how the game plays: an
outdated one (older than a rules or Oracle change, named and dated in the reason), and one
whose words contradict the card's own Oracle text with no dated change to explain it
(register `ruling-contradicting-oracle-gives-way`; the reason names the policy, the date
of the decision and the ruling's likely origin). The parts of either that still hold stay
cited. A ruling inconsistent with the other rulings on a question the CR leaves open is
exempt only by a register decision about that question (Gnoll War Band's, by
`history-counts-players-who-left`). A ruling the data attaches to the wrong card is never
exempt: it is tested on the card it describes and cited under the card it is attached to.
Every exemption whose reason calls the ruling outdated, superseded, overridden,
inconsistent, misattached or a template error, or cites
`ruling-contradicting-oracle-gives-way`, needs an entry quoting that ruling in
`docs/RULING_DISCREPANCIES.md` (`cargo test -p mtg-data` and `rulings-coverage --check`
fail otherwise).

`mtg-tools rulings-coverage` reads every file here and reports exemptions that match no
ruling. `--card "Name"` and `--text "substring"` print the status of individual rulings
(CITED, CITED* = the same text is cited on another card, EXEMPT, UNCOVERED).
