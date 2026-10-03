# Rulings exemptions

One `.tsv` file per batch or area (e.g. `mechanics-adamant.tsv`), each line
`<Card Name><TAB><distinctive substring of the ruling><TAB><reason>`.

Exempt only rulings with no engine-testable content: release history ("This is a change
from previous rules"), card presentation and art, tournament or organized-play policy,
Oracle wording explanations that change no behavior, and similar. Everything else needs a
test citing it with `ruling!("Card Name", "substring")`.

`mtg-tools rulings-coverage` reads every file here and reports exemptions that match no
ruling. `--card "Name"` and `--text "substring"` print the status of individual rulings
(CITED, CITED* = the same text is cited on another card, EXEMPT, UNCOVERED).
